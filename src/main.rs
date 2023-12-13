#![feature(buf_read_has_data_left)]
pub mod Data;
pub mod Pos;
pub mod Sample;
pub mod dataloader;
use byteorder::{LittleEndian, ReadBytesExt};
use dataloader::DataLoader;
use indicatif::{ProgressBar, ProgressStyle};
use mktemp::Temp;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
use std::thread;
use std::{io, path::Path};
use Data::Rescorer;
use Pos::Position;
use Sample::SampleType;
fn main() -> std::io::Result<()> {
    //let mut sample = Sample::Sample::default();

    //println!("Sample {:?}", sample);
    //
    //
    //

    /*let mut dataloader = DataLoader::new(String::from("test.data"), 1000000, false)?;

    for _ in 0..100 {
        let sample = dataloader.get_next()?;
        if let SampleType::Fen(ref position) = sample.position {
            let pos = Position::try_from(position.as_str())?;
            pos.print_position(); e
            println!("Evaluation is: {}", sample.eval);
        }
        println!();
        println!();
    }
    */

    let mut rescorer = Rescorer::new(
        String::from("trainingunique2.pos"),
        String::from("test.data"),
        14,
    );
    rescorer.max_rescores = Some(1000000);

    rescorer.start_rescoring().unwrap();

    /*let mut reader = BufReader::with_capacity(1000000, File::open("trainingunique2.pos")?);
        let mut buffer = String::new();
        let capacity = 1000000;
        reader.read_line(&mut buffer).unwrap();
        let num_samples: u64 = buffer.replace("\n", "").trim().parse().unwrap();
        let num_workers = 2;

        let bar = ProgressBar::new(num_samples);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise},{eta_precise},{per_sec}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        loop {
            let (tx, rx): (Sender<(String, i32)>, Receiver<(String, i32)>) = channel();
            if !reader.has_data_left().unwrap() {
                break;
            }
            let mut lines = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer) {
                    Ok(0) => break,
                    _ => {}
                }
                lines.push(buffer);
            }
            println!("Starting threads");
            let mut handles = Vec::new();
            for chunk_it in lines.chunks(capacity / num_workers) {
                let chunk: Vec<String> = chunk_it.iter().cloned().collect();
                let sender = tx.clone();
                let handle = thread::spawn(move || {
                    let mut command = Command::new("./generator2")
                        .args(["--eval_loop"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to start process");
                    let mut stdin = command.stdin.take().unwrap();
                    let stdout = command.stdout.take().unwrap();
                    let mut f = BufReader::new(stdout);

                    for (counter, value) in chunk.iter().enumerate() {
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
                        /*     println!(
                            "ThreadID: {},  #{counter}   Eval {}  {}",
                            rayon::current_thread_index().unwrap_or(1),
                            splits[1].replace("\n", ""),
                            splits[0].replace("\n", "")
                        );
                        */
                        let eval: i32 = splits[1].trim().replace("\n", "").parse().unwrap();
                        sender
                            .send((String::from(splits[0].trim().replace("\n", "")), eval))
                            .unwrap();
                    }
                    stdin
                        .write_all((String::from("terminate") + "\n").as_bytes())
                        .unwrap();

                    command.kill().unwrap();
                });
                handles.push(handle);
            }

            for value in rx {
                //println!("Got back {:?}", value);
                bar.inc(1);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
        bar.finish();
    */
    Ok(())
}
