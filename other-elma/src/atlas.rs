use cgmath::{vec2, Vector2};
use elma::lgr::{Picture, PictureType, Transparency, LGR};
use image::RgbaImage;
use rect_packer::{Config, Packer, Rect};
use std::collections::BTreeMap;

pub struct Atlas {
    pics: BTreeMap<String, Pic>,
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

pub struct Pic {
    pub info: Option<Picture>,
    pub bounds: [f32; 4],
    pub size: Vector2<f64>,
}

impl Atlas {
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
                    texture[index(row, x, tex_width, rect)] = buffer[x];
                }
            }

            let mut palette = [0; 256 * 3];
            let _palette_len = reader.read_palette(&mut palette).unwrap();

            let info = info.remove(&image.name);
            let (transparency, kind) = info
                .as_ref()
                .map(|info| (info.transparency, info.picture_type))
                .unwrap_or((Transparency::TopLeft, PictureType::Normal));
            let transparent = match transparency {
                _ if kind == PictureType::Texture => None,
                Transparency::Solid => None,
                Transparency::Palette => Some(0),
                Transparency::TopLeft => Some(texture[index(0, 0, tex_width, rect)]),
                Transparency::TopRight => {
                    Some(texture[index(0, width as usize - 1, tex_width, rect)])
                }
                Transparency::BottomLeft => {
                    Some(texture[index(height as usize - 1, 0, tex_width, rect)])
                }
                Transparency::BottomRight => {
                    Some(texture[index(height as usize - 1, width as usize - 1, tex_width, rect)])
                }
            };

            dbg!(transparent);

            for row in 0..height as usize {
                for x in 0..width as usize {
                    let i = index(row, x, tex_width, rect);
                    let index = texture[i];
                    if Some(index) == transparent {
                        texture[i + 0] = 0;
                        texture[i + 1] = 0;
                        texture[i + 2] = 0;
                        texture[i + 3] = 0;
                    } else {
                        for c in 0..3 {
                            texture[i + c] = palette[index as usize * 3 + c];
                        }
                        texture[i + 3] = 255;
                    }
                }
            }

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
                    size: vec2(width as f64, height as f64),
                },
            );
        }

        RgbaImage::from_raw(tex_width as _, tex_height as _, texture.clone())
            .unwrap()
            .save("texture.png")
            .unwrap();

        Atlas {
            pics,
            data: texture,
            width: tex_width,
            height: tex_height,
        }
    }

    pub fn get(&self, name: &str) -> &Pic {
        //   dbg!(name);
        self.pics.get(name).unwrap()
    }
}

fn index(row: usize, column: usize, tex_width: i32, rect: Rect) -> usize {
    ((rect.y as usize + row) * tex_width as usize + rect.x as usize + column) * 4
}
