extern crate byteorder;

use std::io;
use std::path::Path;
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum LgrError {
    NotAnLgr,
    UnknownPictureKind,
    UnknownClippingMode,
    InvalidAscii,
    Io(io::Error),
}

impl From<io::Error> for LgrError {
    fn from(err: io::Error) -> LgrError {
        LgrError::Io(err)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PictureKind {
    Picture,
    Text,
    Mask,
}

#[derive(Debug, Copy, Clone)]
pub enum ClippingMode {
    U,
    G,
    S,
}

#[derive(Debug, Clone)]
pub struct PictureInfo {
    pub name : String,
    pub kind : PictureKind,
    pub distance : u32,
    pub clipping : ClippingMode,

    /// Value with unknown purpose. Usually equals to 12.
    pub unknown : u32,
}

#[derive(Debug, Clone)]
pub struct Picture {
    pub name : String,

    pub unknown_a : u32,
    pub unknown_b : u32,

    pub pcx : Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Lgr {
    pub infos : Vec<PictureInfo>,
    pub pictures : Vec<Picture>,

    /// Value with unknown purpose. Usually equals to 1002.
    pub unknown : u32,
}

fn trim_at_zero(string : &mut Vec<u8>) {
    for i in 0..string.len() {
        if string[i] == 0 {
            string.truncate(i);
            break;
        }
    }
}

fn read_string<R : io::Read>(stream : &mut R, len : usize) -> Result<String, LgrError> {
    let mut string : Vec<u8> = std::iter::repeat(0).take(len).collect();
    stream.read_exact(&mut string)?;

    trim_at_zero(&mut string);

    match String::from_utf8(string) {
        Ok(s) => Ok(s),
        Err(_) => Err(LgrError::InvalidAscii),
    }
}

impl Lgr {
    pub fn load<R : io::Read>(stream : &mut R) -> Result<Self, LgrError> {
        let mut magic  = [0; 5];
        stream.read_exact(&mut magic)?;

        if &magic != b"LGR12" {
            return Err(LgrError::NotAnLgr)
        }

        let total_images = stream.read_u32::<LittleEndian>()? as usize;
        let unknown = stream.read_u32::<LittleEndian>()?;
        let listed_images = stream.read_u32::<LittleEndian>()? as usize;

        // need tp init array to something, these values will be overwritten
        let initial_info = PictureInfo {
            name : "".to_string(),
            kind : PictureKind::Picture,
            distance : 0,
            clipping : ClippingMode::U,
            unknown : 0,
        };
        let mut infos : Vec<PictureInfo> = std::iter::repeat(initial_info).take(listed_images).collect();

        for i in 0..listed_images {
            infos[i].name = read_string(stream, 10)?;
        }

        for i in 0..listed_images {
            let kind = stream.read_u32::<LittleEndian>()?;
            infos[i].kind = match kind {
                100 => PictureKind::Picture,
                101 => PictureKind::Text,
                102 => PictureKind::Text,
                _ => return Err(LgrError::UnknownPictureKind)
            };
        }

        for i in 0..listed_images {
            infos[i].distance = stream.read_u32::<LittleEndian>()?;
        }

        for i in 0..listed_images {
            let clipping = stream.read_u32::<LittleEndian>()?;
            infos[i].clipping = match clipping {
                0 => ClippingMode::U,
                1 => ClippingMode::G,
                2 => ClippingMode::S,
                _ => return Err(LgrError::UnknownClippingMode)
            };
        }

        for i in 0..listed_images {
            infos[i].unknown = stream.read_u32::<LittleEndian>()?;
        }

        let mut pictures = Vec::with_capacity(total_images);
        for _ in 0..total_images {
            let name = read_string(stream, 12)?;
            let unknown_a = stream.read_u32::<LittleEndian>()?;
            let unknown_b = stream.read_u32::<LittleEndian>()?;
            let length = stream.read_u32::<LittleEndian>()? as usize;

            let mut pcx : Vec<u8> = std::iter::repeat(0).take(length).collect();
            stream.read_exact(&mut pcx)?;

            pictures.push(Picture {
                name : name,
                unknown_a : unknown_a,
                unknown_b : unknown_b,
                pcx : Vec::new(),
            })
        }

        Ok(Lgr {
            infos : infos,
            pictures : pictures,
            unknown : unknown,
        })
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LgrError> {
        let file = File::open(path)?;
        Lgr::load(&mut io::BufReader::new(file))
    }
}

#[cfg(test)]
mod tests {
    use ::Lgr;

    #[test]
    fn it_works() {

        let test = Lgr::load_from_file("E:/d/games/ElastoMania/Lgr/Default.lgr").unwrap();

        println!("{:#?}", test);
        assert!(false);
    }
}
