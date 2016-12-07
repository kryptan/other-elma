//! Crate for parsing ElastoMania LGR files.
//!
//! LGR files contain pcx images.

extern crate byteorder;

use std::io;
use std::path::Path;
use std::fs::File;
use std::collections::BTreeMap;
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
    Texture,

    /// Masks are used to draw textures.
    Mask,
}

#[derive(Debug, Copy, Clone)]
pub enum ClippingMode {
    U,
    G,
    S,
}

/// Picture description.
#[derive(Debug, Clone)]
pub struct PictureInfo {
    pub kind : PictureKind,
    pub distance : u32,
    pub clipping : ClippingMode,

    /// Value with unknown purpose. Usually equals to 12.
    pub unknown : u32,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub unknown_a : u32,
    pub unknown_b : u32,

    pub pcx : Vec<u8>,

    /// Optional information describing image.
    pub info : Option<PictureInfo>,
}

/// Structure representing content of an LGR file.
#[derive(Debug, Clone)]
pub struct Lgr {
    pub images : BTreeMap<String, Image>,

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
   /// Read LGR from file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LgrError> {
        let file = File::open(path)?;
        Lgr::load(&mut io::BufReader::new(file))
    }

    /// Read LGR from stream.
    pub fn load<R : io::Read>(stream : &mut R) -> Result<Self, LgrError> {
        let mut magic  = [0; 5];
        stream.read_exact(&mut magic)?;

        if &magic != b"LGR12" {
            return Err(LgrError::NotAnLgr)
        }

        let total_images = stream.read_u32::<LittleEndian>()? as usize;
        let unknown = stream.read_u32::<LittleEndian>()?;
        let listed_images = stream.read_u32::<LittleEndian>()? as usize;

        let mut infos = Self::read_infos(stream, listed_images)?;

        let mut images = BTreeMap::new();
        for _ in 0..total_images {
            let name = read_string(stream, 12)?;
            let unknown_a = stream.read_u32::<LittleEndian>()?;
            let unknown_b = stream.read_u32::<LittleEndian>()?;
            let length = stream.read_u32::<LittleEndian>()? as usize;

            let mut pcx : Vec<u8> = std::iter::repeat(0).take(length).collect();
            stream.read_exact(&mut pcx)?;

            let info = infos.remove(name.trim_right_matches(".pcx"));

            images.insert(name, Image {
                unknown_a : unknown_a,
                unknown_b : unknown_b,
                pcx : Vec::new(),
                info : info,
            });
        }

        Ok(Lgr {
            images : images,
            unknown : unknown,
        })
    }

    fn read_infos<R : io::Read>(stream : &mut R, listed_images : usize) -> Result<BTreeMap<String, PictureInfo>, LgrError> {
        // need tp init array to something, these values will be overwritten
        let initial_info = PictureInfo {
            kind : PictureKind::Picture,
            distance : 0,
            clipping : ClippingMode::U,
            unknown : 0,
        };
        let mut infos : Vec<(String, PictureInfo)> = std::iter::repeat(("".to_string(), initial_info)).take(listed_images).collect();

        for i in 0..listed_images {
            infos[i].0 = read_string(stream, 10)?;
        }

        for i in 0..listed_images {
            let kind = stream.read_u32::<LittleEndian>()?;
            infos[i].1.kind = match kind {
                100 => PictureKind::Picture,
                101 => PictureKind::Texture,
                102 => PictureKind::Mask,
                _ => return Err(LgrError::UnknownPictureKind)
            };
        }

        for i in 0..listed_images {
            infos[i].1.distance = stream.read_u32::<LittleEndian>()?;
        }

        for i in 0..listed_images {
            let clipping = stream.read_u32::<LittleEndian>()?;
            infos[i].1.clipping = match clipping {
                0 => ClippingMode::U,
                1 => ClippingMode::G,
                2 => ClippingMode::S,
                _ => return Err(LgrError::UnknownClippingMode)
            };
        }

        for i in 0..listed_images {
            infos[i].1.unknown = stream.read_u32::<LittleEndian>()?;
        }

        Ok(infos.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use ::Lgr;

    #[test]
    // FIXME: add test with some free LGR file.
    fn it_works() {
     /*   let test = Lgr::load_from_file("E:/d/games/ElastoMania/Lgr/Default.lgr").unwrap();

        println!("{:#?}", test);
        assert!(false);*/
    }
}
