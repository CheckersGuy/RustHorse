#![feature(buf_read_has_data_left)]
pub mod Pos;
pub mod dataloader;
use crate::dataloader::DataLoader;
use dataloader::{create_unique_fens, SampleData, SampleType};
use signal_hook::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use Pos::Position;
use Pos::Square;

//Check why some returned values showed 15000
//when only a capture was possible

fn start_process_thread(child: &mut Child, sender: Sender<String>, receiver: Receiver<String>) {
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    thread::spawn(move || {
        let mut f = BufReader::new(stdout);
        loop {
            match receiver.try_recv() {
                Ok(line) => {
                    stdin.write_all(line.as_bytes()).unwrap();
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => {
                    println!("Error {:?}", e);
                }
            }

            let mut buf = String::new();
            match f.read_line(&mut buf) {
                Ok(_) => {
                    sender.send(buf).unwrap();
                    continue;
                }
                Err(e) => {
                    println!("Error {:?}", e);
                    break;
                }
            }
        }
    });
}

fn start_command_thread(sender: Sender<String>) {
    thread::spawn(move || {
        let reader = BufReader::new(File::open("trainingunique.pos").unwrap());
        for line in reader.lines() {
            sender.send(line.unwrap() + "\n").unwrap();
        }
    });
}

fn start_process(sender: Sender<String>, receiver: Receiver<String>) -> Child {
    let mut child = Command::new("./generator2")
        .args(["--eval_loop"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");
    start_process_thread(&mut child, sender, receiver);
    println!("Started process: {}", child.id());
    child
}
//to be continued

fn main() -> std::io::Result<()> {
    /* let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let mut child = start_process(tx1.clone(), rx2);

    start_command_thread(tx2.clone());
    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&should_terminate))?;

    while !should_terminate.load(Ordering::Relaxed) {
        match rx1.try_recv() {
            Ok(line) => {
                //println!("Got this back {line}");
                let splits: Vec<&str> = line.split("!").collect();
                let pos = Pos::Position::try_from(splits[0]).unwrap();
                pos.print_position();
                println!("Value {}", splits[1]);
                println!(".............................................\n");
            }
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_millis(1));
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
    child.kill()?;
    */

    //let mut loader = DataLoader::new(String::from("shuffled.train.raw.rescored"), 5000000, true)?;
    //loader.dump_pos_to_file(String::from("training.pos"))?;
    create_unique_fens(
        String::from("trainingunique.pos"),
        String::from("traininguniquefixed.pos"),
    )?;

    Ok(())
}
