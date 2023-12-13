use crate::Pos;
use crate::Pos::Position;
use crate::Sample;
use bloomfilter::reexports::bit_vec::BitBlock;
use bloomfilter::Bloom;
use indicatif::{ProgressBar, ProgressStyle};
use mktemp::Temp;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{BufRead, Seek, Write};
use std::option;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use Sample::SampleType;

//Rescorer takes fen/pdn strings and scores them
#[derive(Debug, Default)]
pub struct Rescorer {
    path: String,
    output: String,
    num_workers: usize,
    pub max_rescores: Option<usize>,
}

fn prepend_file<P: AsRef<Path>>(data: &[u8], file_path: &P) -> std::io::Result<()> {
    let tmp_path = Temp::new_file()?;
    let mut tmp = File::create(&tmp_path)?;
    let mut src = File::open(&file_path)?;
    tmp.write_all(&data)?;
    std::io::copy(&mut src, &mut tmp)?;
    std::fs::remove_file(file_path)?;
    std::fs::rename(&tmp_path, file_path)?;

    Ok(())
}

pub fn create_unique_fens<P: AsRef<Path>>(input: &P, output: &P) -> std::io::Result<()> {
    //to be implemented
    let reader = BufReader::with_capacity(10000000, File::open(&input)?);
    let mut writer = File::create(&output)?;
    let mut filter = Bloom::new_for_fp_rate(1000000000, 0.1);
    let mut line_count: usize = 0;
    for line in reader.lines() {
        let fen_string = line?;
        let pos = Position::try_from(fen_string.as_str())?;
        if !filter.check(&pos) {
            writer.write_all((fen_string + "\n").as_str().as_bytes())?;
            filter.set(&pos);
            line_count += 1;
        }
    }
    prepend_file(format!("{line_count}\n").as_bytes(), output)?;
    Ok(())
}

pub fn count_positions<F: Fn(Position) -> bool>(
    path: String,
    predicate: F,
) -> std::io::Result<usize> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    let bar = ProgressBar::new(buffer.replace("\n", "").parse::<u64>().unwrap());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise},{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut counter: usize = 0;
    for line in reader.lines() {
        let pos = Position::try_from(line.unwrap().as_str())?;
        if predicate(pos) {
            counter += 1;
        }
        bar.inc(1);
    }

    Ok(counter)
}

pub fn count_material_less_than(path: String, count: usize) -> std::io::Result<usize> {
    count_positions(path, |pos| {
        (pos.bp.count_ones() + pos.wp.count_ones()) as usize <= count
    })
}

pub fn get_material_distrib(path: String) -> std::io::Result<HashMap<u32, usize>> {
    let mut my_map = HashMap::new();

    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    let bar = ProgressBar::new(buffer.replace("\n", "").parse::<u64>().unwrap());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise},{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    for line in reader.lines() {
        let pos = Position::try_from(line.unwrap().as_str())?;
        let piece_count = pos.bp.count_ones() + pos.wp.count_ones();
        *my_map.entry(piece_count).or_insert(0) += 1;
        bar.inc(1);
    }
    Ok(my_map)
}

impl Rescorer {
    pub fn new(path: String, output: String, num_workers: usize) -> Rescorer {
        Rescorer {
            path,
            output,
            num_workers,
            ..Rescorer::default()
        }
    }

    pub fn start_rescoring(self) -> std::io::Result<()> {
        let mut reader = Arc::new(Mutex::new(BufReader::with_capacity(
            1000000,
            File::open(self.path)?,
        )));
        let mut output = File::create(self.output.clone())?;
        let mut buffer = String::new();
        let mut counter: usize = 0; // count how many elements have  beenprocessed by our main
                                    // thread
        let mut thread_counter = Arc::new(AtomicUsize::new(0));
        {
            let mut guard = reader.lock().unwrap();
            guard.read_line(&mut buffer).unwrap();
        }
        let num_samples: u64 = buffer.replace("\n", "").trim().parse().unwrap();

        let progress_count = match self.max_rescores {
            Some(value) => value as u64,
            None => num_samples,
        };

        println!("NumSamples {}", num_samples);
        let bar = ProgressBar::new(progress_count);
        bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise},{eta_precise},{per_sec}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

        let (tx, rx): (Sender<(String, i32)>, Receiver<(String, i32)>) = mpsc::channel();

        println!("Starting threads");
        let mut handles = Vec::new();
        for thread_id in 0..self.num_workers {
            let reader_local = Arc::clone(&reader);

            let sender = tx.clone();
            let a_counter = Arc::clone(&thread_counter);
            let handle = std::thread::spawn(move || {
                let mut command = Command::new("./generator2")
                    .args(["--eval_loop"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to start process");
                let mut stdin = command.stdin.take().unwrap();
                let stdout = command.stdout.take().unwrap();
                let mut f = BufReader::new(stdout);

                //creating a local buffer for each thread to hold some positions
                'outer: loop {
                    let mut buffer = Vec::with_capacity(10000);
                    {
                        let mut guard = reader_local.lock().unwrap();
                        if !guard.has_data_left().unwrap() {
                            break 'outer;
                        }
                        for _ in 0..10000 {
                            let mut b = String::new();
                            match guard.read_line(&mut b) {
                                Ok(0) => break,
                                _ => {}
                            }
                            buffer.push(b);
                        }
                        println!("Filled buffer for thread{thread_id}");
                    }

                    for (counter, value) in buffer.iter().enumerate() {
                        let trimmed = value.trim().replace("\n", "");
                        stdin
                            .write_all((String::from(trimmed) + "\n").as_bytes())
                            .unwrap();
                        let mut buffer = String::new();
                        match f.read_line(&mut buffer) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{:?}", e)
                            }
                        }
                        let splits: Vec<&str> = buffer.split("!").collect();

                        let eval: i32 = splits[1].trim().replace("\n", "").parse().unwrap();
                        let is_send =
                            sender.send((String::from(splits[0].trim().replace("\n", "")), eval));

                        if let Err(_) = is_send {
                            break 'outer;
                        }

                        //Now we can count up
                        a_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if a_counter.load(std::sync::atomic::Ordering::SeqCst)
                            >= self.max_rescores.unwrap_or(num_samples as usize)
                        {
                            break 'outer;
                        }
                    }
                }
                stdin
                    .write_all((String::from("terminate\n")).as_bytes())
                    .unwrap();

                command.kill().unwrap();
            });
            handles.push(handle);
        }

        for value in rx {
            let (pos, eval) = value;
            let mut sample = Sample::Sample::default();
            sample.eval = eval as i16;
            sample.position = SampleType::Fen(pos);
            sample.write_fen(&mut output)?;
            bar.inc(1);
            counter += 1;
            //to be save code below
            if thread_counter.load(std::sync::atomic::Ordering::SeqCst)
                >= self.max_rescores.unwrap_or(num_samples as usize)
            {
                break;
            }
        }
        for handle in handles {
            handle.join().unwrap();
        }
        drop(output);
        println!("We wrote {} samples to output", counter);

        let bytes = counter.to_le_bytes();
        let path = Path::new(self.output.as_str());
        prepend_file(bytes.as_slice(), &path)?;
        Ok(())
    }
}
