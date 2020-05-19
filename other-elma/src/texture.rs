use cgmath::Vector2;
use elma::lgr::{Picture, PictureData, LGR};
use image::RgbaImage;
use rect_packer::{Config, Packer};
use std::collections::BTreeMap;

pub struct Texture {
    pics: BTreeMap<String, Pic>,
    pub texture: Vec<u8>,
    pub tex_width: i32,
    pub tex_height: i32,
}

pub struct Pic {
    pub info: Option<Picture>,
    pub bounds: [f32; 4],
}

impl Texture {
    pub fn new(path: &str) -> Self {
        let lgr = LGR::load(path).unwrap();

        let tex_width = 2048;
        let tex_height = 2048;
        let mut rect_packer = Packer::new(Config {
            width: tex_width,
            height: tex_height,
            border_padding: 1,
            rectangle_padding: 1,
        });

        let mut texture = vec![0; 4 * tex_width as usize * tex_height as usize];

        let mut info: BTreeMap<String, Picture> = lgr
            .picture_list
            .into_iter()
            .map(|pic| (pic.name.clone(), pic))
            .collect();

        let mut pics = BTreeMap::new();

        let mut buffer = Vec::new();
        for image in lgr.picture_data.into_iter() {
            dbg!(&image.name);
            let mut reader = pcx::Reader::new(&image.data[..]).unwrap();
            let width = reader.width();
            let height = reader.height();

            let rect = rect_packer
                .pack(width as i32, height as i32, false)
                .unwrap();
            buffer.resize(width as usize, 0);

            for row in 0..height as usize {
                reader.next_row_paletted(&mut buffer).unwrap();
                for x in 0..width as usize {
                    texture[((rect.y as usize + row) * tex_width as usize + rect.x as usize + x)
                        * 4] = buffer[x];
                }
            }

            let mut palette = [0; 256 * 3];
            let palette_len = reader.read_palette(&mut palette).unwrap();
            dbg!(palette.iter().collect::<Vec<&u8>>());
            for row in 0..height as usize {
                for x in 0..width as usize {
                    let i =
                        ((rect.y as usize + row) * tex_width as usize + rect.x as usize + x) * 4;
                    let index = texture[i] as usize;
                    for c in 0..3 {
                        texture[i + c] = palette[index * 3 + c];
                    }
                    texture[i + 3] = 255; // FIXME: set transparency
                }
            }

            let info = info.remove(&image.name);
            pics.insert(
                image.name,
                Pic {
                    info,
                    bounds: [
                        rect.x as f32 / tex_width as f32,
                        rect.y as f32 / tex_height as f32,
                        (rect.x + rect.width) as f32 / tex_width as f32,
                        (rect.y + rect.height) as f32 / tex_height as f32,
                    ],
                },
            );
        }

        RgbaImage::from_raw(tex_width as _, tex_height as _, texture.clone())
            .unwrap()
            .save("texture.png")
            .unwrap();

        Texture {
            pics,
            texture,
            tex_width,
            tex_height,
        }
    }

    pub fn get(&self, name: &str) -> &Pic {
        self.pics.get(name).unwrap()
    }
}
