//! Crate for parsing ElastoMania LGR files.
//!
//! LGR files contain PCX images.
extern crate byteorder;
extern crate pcx;

use std::{io, iter};
use std::path::Path;
use std::fs::File;
use std::collections::BTreeMap;
use byteorder::{LittleEndian, ReadBytesExt};

/// Kind of a picture.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PictureKind {
    /// A simple picture with the top-left pixel signing the transparent color.
    Picture,

    /// Texture.
    Texture,

    /// Masks are used to draw textures.
    Mask,
}

/// Clipping property.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClippingMode {
    /// `Unclipped` means that the picture or texture will be seen above the whole picture (unless other pictures hide it).
    Unclipped,

    /// `Ground` means the picture will be seen only above the ground.
    Ground,

    /// `Sky` means the picture will be seen only above the sky.
    Sky,
}

/// Picture description.
#[derive(Debug, Clone, Copy)]
pub struct Picture {
    /// Whether this is picture, texture or mask.
    pub kind : PictureKind,

    /// The distance must be in the 1-999 range. If a picture has less distance than an other, it will hide the other picture.
    /// The motorbiker and the food-exit-killer objects have a distance of 500.
    pub distance : u32,

    /// The clipping property determines whether a picture is seen above the sky, the ground, or both (this is independent of the distances).
    pub clipping : ClippingMode,
}

/// Image from LGR.
pub struct Image {
    /// Optional information describing image.
    pub info : Option<Picture>,

    /// Width of the image.
    pub width : u16,

    /// Height of the image.
    pub height : u16,

    /// Image pixels. Each pixel is an index into LGR palette.
    pub pixels : Vec<u8>,

    /// Raw content of the PCX file.
    pub pcx : Vec<u8>,
}

/// Content of an LGR file.
pub struct Lgr {
    /// All images contained in this LGR. Names include .pcx extension.
    pub images : BTreeMap <String, Image>,

    /// Palette contains 256-colors, format is R0, G0, B0, R1, G1, B1, ...
    pub palette : Vec<u8>,
}

impl Image {
    #[inline]
    /// Get image pixel. Returned value is an index into LGR palette.
    pub fn get_pixel(&self, x : u16, y : u16) -> u8 {
        self.pixels[(y as usize)*(self.width as usize) + (x as usize)]
    }
}

impl Lgr {
    /// Get color from LGR palette. There are 256 colors in the palette.
    ///
    /// Returns triple (R, G, B).
    #[inline]
    pub fn get_palette_color(&self, i : u8) -> (u8, u8, u8) {
        (
            self.palette[(i as usize)*3],
            self.palette[(i as usize)*3 + 1],
            self.palette[(i as usize)*3 + 2]
        )
    }
}

fn trim_at_zero(string : &mut Vec<u8>) {
    for i in 0..string.len() {
        if string[i] == 0 {
            string.truncate(i);
            break;
        }
    }
}

fn error<T>(msg : &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}

fn read_string<R : io::Read>(stream : &mut R, len : usize) -> io::Result<String> {
    let mut string : Vec<u8> = std::iter::repeat(0).take(len).collect();
    stream.read_exact(&mut string)?;

    trim_at_zero(&mut string);

    match String::from_utf8(string) {
        Ok(s) => Ok(s),
        Err(_) => error("LGR: invalid ASCII"),
    }
}

/// Read LGR from file.
pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Lgr> {
    let file = File::open(path)?;
    load(&mut io::BufReader::new(file))
}

/// Read LGR from stream.
pub fn load<R : io::Read>(stream : &mut R) -> io::Result<Lgr> {
    let mut magic  = [0; 5];
    stream.read_exact(&mut magic)?;

    if &magic != b"LGR12" {
        return error("Not an LGR");
    }

    let total_images = stream.read_u32::<LittleEndian>()? as usize;
    let unknown = stream.read_u32::<LittleEndian>()?;
    if unknown != 1002 { // Some kind of version or something like that
        return error("LGR: invalid unknown value != 1002");
    }

    let listed_images = stream.read_u32::<LittleEndian>()? as usize;

    let mut infos = read_pictures(stream, listed_images)?;

    let mut images = BTreeMap::new();
    let mut palette = Vec::new();
    for _ in 0..total_images {
        let name = read_string(stream, 12)?;
        let _unknown_a = stream.read_i32::<LittleEndian>()?;
        let _unknown_b = stream.read_i32::<LittleEndian>()?;
        let length = stream.read_u32::<LittleEndian>()? as usize;

        let mut pcx : Vec<u8> = std::iter::repeat(0).take(length).collect();
        stream.read_exact(&mut pcx)?;

        let info = infos.remove(name.trim_right_matches(".pcx"));

        let (pixels, width, height) = {
            let read_stream = &pcx[..];
            let mut pcx_reader = pcx::Reader::new(read_stream)?;
            let (width, height) = (pcx_reader.width() as usize, pcx_reader.height() as usize);

            let mut pixels: Vec<u8> = iter::repeat(0).take(width*height).collect();

            for i in 0..height {
                pcx_reader.next_row_paletted(&mut pixels[i*width..(i + 1)*width])?;
            }

            // Masks contain invalid palettes, we can take palette from the first image that is not a mask.
            let valid_palette = if let Some(info) = info {
                info.kind != PictureKind::Mask
            } else {
                true
            };

            if valid_palette && palette.is_empty() {
                palette = iter::repeat(0).take(256*3).collect();
                pcx_reader.read_palette(&mut palette)?;
            }

            (pixels, width as u16, height as u16)
        };

        images.insert(name, Image {
            info : info,
            width : width,
            height : height,
            pixels : pixels,
            pcx : pcx,
        });
    }

    Ok(Lgr {
        images : images,
        palette : palette,
    })
}

fn read_pictures<R : io::Read>(stream : &mut R, listed_images : usize) -> io::Result<BTreeMap<String, Picture>> {
    // need tp init array to something, these values will be overwritten
    let initial_info = Picture {
        kind : PictureKind::Picture,
        distance : 0,
        clipping : ClippingMode::Unclipped,
    };
    let mut pictures : Vec<(String, Picture)> = std::iter::repeat(("".to_string(), initial_info)).take(listed_images).collect();

    for i in 0..listed_images {
        pictures[i].0 = read_string(stream, 10)?;
    }

    for i in 0..listed_images {
        let kind = stream.read_u32::<LittleEndian>()?;
        pictures[i].1.kind = match kind {
            100 => PictureKind::Picture,
            101 => PictureKind::Texture,
            102 => PictureKind::Mask,
            _ => return error("LGR: unknown picture kind"),
        };
    }

    for i in 0..listed_images {
        pictures[i].1.distance = stream.read_u32::<LittleEndian>()?;
    }

    for i in 0..listed_images {
        let clipping = stream.read_u32::<LittleEndian>()?;
        pictures[i].1.clipping = match clipping {
            0 => ClippingMode::Unclipped,
            1 => ClippingMode::Ground,
            2 => ClippingMode::Sky,
            _ => return error("LGR: unknown clipping mode"),
        };
    }

    for _ in 0..listed_images {
        let unknown = stream.read_u32::<LittleEndian>()?;
        if unknown != 12 {
            return error("LGR: invalid unknown value != 12");
        }
    }

    Ok(pictures.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use load_from_file;

    #[test]
    // FIXME: add test with some free LGR file.
    fn it_works() {
     //   let test = Lgr::load_from_file("E:/d/games/ElastoMania/Lgr/Default.lgr").unwrap();
        let test = load_from_file("E:/d/games/ElastoMania/Lgr/Carma.lgr").unwrap();

     //   println!("{:#?}", test);
        assert!(false);
    }
}
