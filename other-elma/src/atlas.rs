use cgmath::{vec2, Vector2};
use elma::lgr::{Picture, PictureType, Transparency, LGR};
use rect_packer::{Config, Packer, Rect};
use std::collections::BTreeMap;

pub struct Atlas {
    pub sprites: BTreeMap<String, Sprite>,
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

pub struct Sprite {
    pub bounds: [f32; 4],
    pub size: Vector2<f64>,
}

impl Atlas {
    pub fn new(path: &str) -> Self {
        let lgr = LGR::load(path).unwrap();

        let atlas_width = 2048;
        let atlas_height = 2048;
        let mut rect_packer = Packer::new(Config {
            width: atlas_width,
            height: atlas_height,
            border_padding: 1,
            rectangle_padding: 1,
        });

        let mut data = vec![0; 4 * atlas_width as usize * atlas_height as usize];

        let mut info: BTreeMap<String, Picture> = lgr
            .picture_list
            .into_iter()
            .map(|pic| (pic.name.clone(), pic))
            .collect();

        let mut sprites = BTreeMap::new();

        let mut buffer = Vec::new();
        for image in lgr.picture_data.into_iter() {
            let name = &image.name[..image.name.len() - 4];

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
                    data[index(row, x, atlas_width, rect)] = buffer[x];
                }
            }

            let mut palette = [0; 256 * 3];
            let _palette_len = reader.read_palette(&mut palette).unwrap();

            let info = info.remove(name);
            let (transparency, mut kind) = info
                .as_ref()
                .map(|info| (info.transparency, info.picture_type))
                .unwrap_or((Transparency::TopLeft, PictureType::Normal));
            if name == "QGRASS" {
                kind = PictureType::Texture;
            }

            let transparent = match transparency {
                _ if kind == PictureType::Texture => None,
                Transparency::Solid => None,
                Transparency::Palette => Some(0),
                Transparency::TopLeft => Some(data[index(0, 0, atlas_width, rect)]),
                Transparency::TopRight => {
                    Some(data[index(0, width as usize - 1, atlas_width, rect)])
                }
                Transparency::BottomLeft => {
                    Some(data[index(height as usize - 1, 0, atlas_width, rect)])
                }
                Transparency::BottomRight => {
                    Some(data[index(height as usize - 1, width as usize - 1, atlas_width, rect)])
                }
            };

            for row in 0..height as usize {
                for x in 0..width as usize {
                    let i = index(row, x, atlas_width, rect);
                    let index = data[i];
                    if Some(index) == transparent {
                        for c in 0..4 {
                            data[i + c] = 0;
                        }
                    } else {
                        for c in 0..3 {
                            data[i + c] = palette[index as usize * 3 + c];
                        }
                        data[i + 3] = 255;
                    }
                }
            }

            let mut left = rect.x as f32;
            let mut top = rect.y as f32;
            let mut right = left + rect.width as f32;
            let mut bottom = top + rect.height as f32;
            if kind == PictureType::Texture {
                left += 0.5;
                top += 0.5;
                right -= 0.5;
                bottom -= 0.5;
            }
            sprites.insert(
                name.to_owned(),
                Sprite {
                    bounds: [
                        left / atlas_width as f32,
                        top / atlas_height as f32,
                        right / atlas_width as f32,
                        bottom / atlas_height as f32,
                    ],
                    size: vec2(width as f64, height as f64),
                },
            );
        }

        /*    RgbaImage::from_raw(atlas_width as _, atlas_height as _, texture.clone())
        .unwrap()
        .save("texture.png")
        .unwrap();*/

        Atlas {
            sprites,
            data,
            width: atlas_width,
            height: atlas_height,
        }
    }

    pub fn get(&self, name: &str) -> &Sprite {
        //  dbg!(name);
        self.sprites.get(name).unwrap()
    }
}

fn index(row: usize, column: usize, atlas_width: i32, rect: Rect) -> usize {
    ((rect.y as usize + row) * atlas_width as usize + rect.x as usize + column) * 4
}
