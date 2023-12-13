//move sample definition from dataloader here
//should make stuff a little easier to handle :)
use crate::Pos::Position;
use byteorder::WriteBytesExt;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::prelude::*;
#[derive(Debug)]
pub enum SampleType {
    Fen(String),   //a not yet converted FenString
    Pos(Position), //already converted to a position
    None,
}

#[derive(Debug)]
pub enum Result {
    UNKNOWN,
    WIN,
    LOSS,
    DRAW,
}

impl Default for Result {
    fn default() -> Self {
        Result::UNKNOWN
    }
}

impl Default for SampleType {
    fn default() -> Self {
        SampleType::None
    }
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

#[derive(Default, Debug)]
pub struct Sample {
    pub position: SampleType,
    pub eval: i16,
    pub result: Result,
}

impl Sample {
    //only writes if SampleType is a fenstring
    //or we need a get_fen_string function as well to do the conversion
    pub fn write_fen<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        //should implement a function that gets me the fenstring !!!

        if let SampleType::Fen(ref fen_string) = self.position {
            //need to change this after makign eval an option
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

    pub fn read_into<R: Read>(&self, reader: &R) -> std::io::Result<()> {
        // to be added
        Ok(())
    }

    //converts the evaluation to a win_percentage
    //positions are always white to move first
}
