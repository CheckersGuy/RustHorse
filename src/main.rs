#![feature(buf_read_has_data_left)]
pub mod Pos;
pub mod dataloader;
use crate::dataloader::DataLoader;
use dataloader::{create_unique_fens, SampleData, SampleType};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use Pos::Position;
use Pos::Square;

fn start_listener(sender: Sender<String>, receiver: Receiver<String>) {
    let child = Command::new("ping")
        .arg("google.com")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Could not start process");
    println!("Started process {}", child.id());

    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.unwrap());
        let mut stdin = child.stdin.unwrap();

        for line in receiver {
            stdin.write_all(line.as_bytes()).unwrap();
        }
    });
}

fn main() -> std::io::Result<()> {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    start_listener(tx1, rx2);

    //let mut loader = DataLoader::new(String::from("shuffled.train.raw.rescored"), 5000000, true)?;
    //loader.dump_pos_to_file(String::from("training.pos"))?;
    /*create_unique_fens(
        String::from("training.pos"),
        String::from("trainingunqiue.pos"),
    );
    */
    Ok(())
}
