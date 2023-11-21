use crate::Pos::Position;
use bloomfilter::Bloom;
use byteorder::WriteBytesExt;
use byteorder::{LittleEndian, ReadBytesExt};
use mktemp::Temp;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rip_shuffle::RipShuffleParallel;
use std::cell::RefCell;
use std::cmp;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
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

fn prepend_file<P: AsRef<Path>>(data: &[u8], file_path: &P) -> io::Result<()> {
    let mut tmp_path = Temp::new_file()?;
    let mut tmp = File::create(&tmp_path)?;
    let mut src = File::open(&file_path)?;
    tmp.write_all(&data)?;
    io::copy(&mut src, &mut tmp)?;
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

impl Sample {
    //only writes if SampleType is a fenstring
    //or we need a get_fen_string function as well to do the conversion
    pub fn write_fen<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if let SampleType::Fen(ref fen_string) = self.position {
            let length: u16 = fen_string.len() as u16;
            writer.write_u16::<LittleEndian>(length)?;
            writer.write_all(fen_string.as_bytes())?;
            writer.write_i16::<LittleEndian>(self.eval)?;
            let conv;
            match self.result {
                Result::LOSS => conv = 1,
                Result::WIN => conv = 2,
                Result::DRAW => conv = 3,
                Result::UNKNOWN => conv = 0,
            }
            writer.write_i8(conv)?;
        }

        Ok(())
    }
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
                let result = self.read()?;
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
