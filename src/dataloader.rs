use crate::Pos::Position;
use bloomfilter::Bloom;
use byteorder::{LittleEndian, ReadBytesExt};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rip_shuffle::RipShuffleParallel;
use std::cell::RefCell;
use std::cmp;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::{Duration, Instant};
#[derive(Debug)]
pub enum Result {
    UNKNOWN,
    WIN,
    LOSS,
    DRAW,
}

impl From<i8> for Result {
    fn from(item: i8) -> Self {
        match item {
            1 => Result::LOSS,
            2 => Result::WIN,
            3 => Result::DRAW,
            _ => Result::UNKNOWN,
        }
    }
}

pub fn create_unique_fens(input: String, output: String) -> std::io::Result<()> {
    //to be implemented
    let reader = BufReader::new(File::open(input)?);
    let mut writer = File::create(output)?;
    let counter = 1000000000;
    let mut filter = Bloom::new_for_fp_rate(counter, 0.1);
    let mut capture_count = 0;
    for line in reader.lines() {
        let fen_string = line?;
        let pos = Position::try_from(fen_string.as_str())?;
        //checking if we have a capture
        if !filter.check(&pos) {
            writer.write_all((fen_string + "\n").as_str().as_bytes())?;
            filter.set(&pos)
        }
    }

    println!("Found a total of {} captures", capture_count);

    Ok(())
}

pub struct SampleData {
    reader: std::io::BufReader<std::fs::File>,
    num_samples: u64,
}

struct SampleIterator {
    data: RefCell<SampleData>,
}

pub enum SampleType {
    Fen(String),   //a not yet converted FenString
    Pos(Position), //already converted to a position
    None,
}

pub struct Sample {
    pub position: SampleType,
    pub eval: i16,
    pub result: Result,
}

pub struct DataLoader {
    reader: std::io::BufReader<std::fs::File>,
    pub path: String,
    shuff_buf: Vec<Sample>,
    shuffle: bool,
    pub num_samples: u64,
    capa: usize,
    rng: StdRng,
}

impl DataLoader {
    pub fn new(path: String, capacity: usize, shuffle: bool) -> std::io::Result<DataLoader> {
        let mut data_loader = DataLoader {
            reader: BufReader::with_capacity(10000000, File::open(path.clone())?),
            path: path.clone(),
            shuff_buf: Vec::with_capacity(capacity),
            num_samples: 0,
            shuffle,
            capa: capacity,
            rng: StdRng::from_rng(thread_rng()).unwrap(),
        };
        data_loader.num_samples = data_loader.reader.read_u64::<LittleEndian>()?;
        data_loader.capa = cmp::min(data_loader.capa, data_loader.num_samples as usize);
        println!("Got {} available samples", data_loader.num_samples);
        Ok(data_loader)
    }

    pub fn read(&mut self) -> std::io::Result<Sample> {
        let has_data_left = self.reader.has_data_left()?;
        if !has_data_left {
            println!("Reached the end of the file and buffer is empty");
            self.reader.rewind()?;
            //reading the file size once more since we rewind the stream
            self.reader.read_u64::<LittleEndian>()?;
        }
        let val = self.reader.read_u16::<LittleEndian>()?;
        let mut buffer = vec![0; val as usize];
        self.reader.read_exact(&mut buffer)?;
        let s = String::from_utf8(buffer).unwrap();
        let eval = self.reader.read_i16::<LittleEndian>()?;
        let res = self.reader.read_i8()?;

        Ok(Sample {
            position: SampleType::Fen(s),
            eval,
            result: Result::from(res),
        })
    }

    pub fn get_next(&mut self) -> std::io::Result<Sample> {
        if self.shuff_buf.is_empty() {
            let now = Instant::now();
            for _ in 0..self.capa {
                let mut result;
                loop {
                    result = self.read()?;
                    match result.position {
                        SampleType::Pos(position) => {
                            let piece_count = position.wp.count_ones() + position.bp.count_ones();
                            if piece_count > 10 {
                                continue;
                            } else {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                self.shuff_buf.push(result);
            }
            if self.shuffle {
                let shuff_time = Instant::now();
                self.shuff_buf.par_shuffle(&mut self.rng);
                println!("Shuffled the buffer");
                println!("ShuffleTime {}", shuff_time.elapsed().as_millis());
            }
            //Need to convert all the samples
            let transform = Instant::now();
            self.shuff_buf.par_iter_mut().for_each(|sample| {
                if let SampleType::Fen(ref fen_string) = sample.position {
                    sample.position = SampleType::Pos(Position::try_from(&fen_string[..]).unwrap());
                }
            });

            //and now with rayon
            let elapsed = now.elapsed().as_millis();
            println!("Elapsed time {}", elapsed);
            println!("Transformation time {}", transform.elapsed().as_millis());
        }

        let sample = self.shuff_buf.pop().unwrap();
        Ok(sample)
    }

    pub fn dump_pos_to_file(&mut self, output: String) -> std::io::Result<()> {
        self.shuffle = false;
        println!("We have {} positions", self.num_samples);
        let mut writer = File::create(output)?;
        while let Ok(sample) = self.get_next() {
            match sample.position {
                SampleType::Fen(fen_string) => {
                    writer.write_all((fen_string + "\n").as_bytes())?;
                }
                SampleType::Pos(_) => (),
                SampleType::None => (),
            }
        }

        Ok(())
    }
}
