use crate::atlas::{Atlas, Sprite};
use crate::render::{PictureVertex, PolygonVertex, Viewport};
use crate::triangulation::triangulate;
use cgmath::{vec2, Vector2};
use elma::constants::OBJECT_RADIUS;
use elma::lev::{Level, ObjectType};
use elma::Clip;
use lyon_tessellation::VertexBuffers;
use std::cmp::{max, min};

/*
1st pass - render polygons with depth
2ns pass - render sorted pictures with depth test but no depth writing
*/

pub struct Scene {
    pub vertices: Vec<PictureVertex>,
    pub indices: Vec<u32>,
    pub polygons: VertexBuffers<PolygonVertex, u32>,
    sky: usize,
    sky_size: Vector2<f64>,
    ground: usize,
    ground_size: Vector2<f64>,
    objects: Vec<Object>,
}

struct Object {
    index: usize,
    position_y: Option<f64>,
    bounds: [f32; 4],
    num_frames: i32,
}

fn vec_dir(i: i32) -> Vector2<f64> {
    match i {
        0 => vec2(0.0, 0.0),
        1 => vec2(1.0, 0.0),
        2 => vec2(1.0, 1.0),
        3 => vec2(0.0, 1.0),
        _ => unreachable!(),
    }
}

const PIXELS_PER_UNIT: f64 = 95.0 / 2.0; // FIXME: the exact coefficient isn't known

impl Scene {
    pub fn new(level: &mut Level, atlas: &Atlas) -> Scene {
        let sky_texture = atlas.get(&level.sky);
        let ground_texture = atlas.get(&level.ground);
        let sky_size = sky_texture.size;
        let ground_size = ground_texture.size;
        let grass_texture = atlas.get("QGRASS");

        let polygons = triangulate(&level, false, |position| PolygonVertex {
            position,
            clip: 0.0,
        });

        let vertices = Vec::new();
        let indices = Vec::new();

        let mut scene = Scene {
            vertices,
            indices,
            polygons,
            sky: 0,
            sky_size,
            ground: 0,
            ground_size,
            objects: Vec::new(),
        };

        scene.sky = scene.add_image(sky_texture, vec2(0.0, 0.0), Clip::Sky, false);
        scene.ground = scene.add_image(ground_texture, vec2(0.0, 0.0), Clip::Ground, false);

        let grass = triangulate(&level, true, |position| PictureVertex {
            position,
            tex_coord: position,
            tex_bounds: grass_texture.bounds,
            mask: [-1.0, -1.0],
            clip: 0.0,
        });
        let num_vertices = scene.vertices.len();
        scene.vertices.extend_from_slice(&grass.vertices);
        scene
            .indices
            .extend(grass.indices.into_iter().map(|i| i + num_vertices as u32));

        level.pictures.sort_by_key(|picture| picture.distance);
        for pic in level.pictures.iter().rev() {
            if !pic.name.is_empty() {
                let sprite = atlas.get(&pic.name);
                scene.add_image(
                    sprite,
                    vec2(pic.position.x, pic.position.y),
                    pic.clip,
                    false,
                );
            } else if !pic.texture.is_empty() && !pic.mask.is_empty() {
                //    dbg!(pic.texture);
                //   dbg!(pic.mask);
                let texture = atlas.get(&pic.texture);
                let mask = atlas.get(&pic.mask);

                //  scene.add_image(mask, vec2(pic.position.x, pic.position.y), pic.clip, false);

                let mask_pos = vec2(mask.bounds[0] as f64, mask.bounds[1] as f64);
                let mask_size = vec2(mask.bounds[2] as f64, mask.bounds[3] as f64) - mask_pos;

                let v = scene.vertices.len() as u32;
                for i in 0..4 {
                    let v = vec_dir(i);
                    let p = vec2(pic.position.x, pic.position.y)
                        + (1.0 / PIXELS_PER_UNIT) * vec2(v.x * mask.size.x, -v.y * mask.size.y);
                    let mask = mask_pos + vec2(v.x * mask_size.x, v.y * mask_size.y);

                    scene.vertices.push(PictureVertex {
                        position: [p.x as f32, p.y as f32],
                        tex_coord: position_to_tex_coord(p, texture.size),
                        tex_bounds: texture.bounds,
                        mask: [mask.x as f32, mask.y as f32],
                        clip: match pic.clip {
                            Clip::Unclipped => 0.5,
                            Clip::Ground => 0.0,
                            Clip::Sky => 1.0,
                        },
                    });
                }

                scene
                    .indices
                    .extend_from_slice(&[v, v + 1, v + 2, v, v + 2, v + 3]);
            }
        }

        for object in &level.objects {
            let name;
            let name = match object.object_type {
                ObjectType::Apple { animation, .. } => {
                    name = format!("qfood{}", min(1, max(2, animation)));
                    &name
                }
                ObjectType::Exit => "QEXIT",
                ObjectType::Killer => "QKILLER",
                ObjectType::Player => continue,
            };

            let sprite = atlas.get(name);
            let index = scene.add_image(
                sprite,
                vec2(object.position.x, object.position.y) - vec2(OBJECT_RADIUS, -OBJECT_RADIUS),
                Clip::Unclipped,
                true,
            );
            scene.objects.push(Object {
                index,
                bounds: sprite.bounds,
                position_y: (object.object_type != ObjectType::Killer).then_some(object.position.y),
                num_frames: (sprite.size.x / sprite.size.y).round() as i32,
            });
        }

        scene
    }

    pub fn add_image(
        &mut self,
        sprite: &Sprite,
        position: Vector2<f64>,
        clip: Clip,
        animated: bool,
    ) -> usize {
        let v = self.vertices.len() as u32;
        let size_x = if animated {
            sprite.size.y
        } else {
            sprite.size.x
        };

        for i in 0..4 {
            let v = vec_dir(i);
            let p = position + (1.0 / PIXELS_PER_UNIT) * vec2(v.x * size_x, -v.y * sprite.size.y);

            self.vertices.push(PictureVertex {
                position: [p.x as f32, p.y as f32],
                tex_coord: [v.x as f32, v.y as f32],
                tex_bounds: sprite.bounds,
                mask: [-1.0, -1.0],
                clip: match clip {
                    Clip::Unclipped => 0.5,
                    Clip::Ground => 0.0,
                    Clip::Sky => 1.0,
                },
            });
        }

        self.indices
            .extend_from_slice(&[v, v + 1, v + 2, v, v + 2, v + 3]);

        v as usize
    }

    pub fn animate(&mut self, time: f64) {
        let frame = (time * 30.0) as i32; // FIXME: exact framerate is unknown
        for object in &self.objects {
            let vertices = &mut self.vertices[object.index..object.index + 4];
            let frame = (frame % object.num_frames) as f32;

            if let Some(position_y) = object.position_y {
                let shift = (time * 4.0).sin() * 0.1; // FIXME: exact parameters are unknown
                let top = (position_y + OBJECT_RADIUS + shift) as f32;
                let bottom = (position_y - OBJECT_RADIUS + shift) as f32;
                vertices[0].position[1] = top;
                vertices[1].position[1] = top;
                vertices[2].position[1] = bottom;
                vertices[3].position[1] = bottom;
            }

            for vertex in vertices {
                vertex.tex_bounds = [
                    object.bounds[0] + (object.bounds[3] - object.bounds[1]) * frame,
                    object.bounds[1],
                    object.bounds[0] + (object.bounds[3] - object.bounds[1]) * (frame + 1.0),
                    object.bounds[3],
                ];
            }
        }
    }

    pub fn update(&mut self, viewport: Viewport) {
        let sky = &mut self.vertices[self.sky..];
        //  let sky_width =
        //      sky_size.y as f64 / sky_size.x as f64 * viewport.size.x / viewport.size.y;
        //   let sky_offset = viewport.position.x;
        // FIXME: vertical inversion
        for i in 0..4 {
            /*    let v = vec_dir(i);
            let p = viewport.position + vec2(viewport.size.x * v.x, viewport.size.y * v.y);
            sky[i as usize].position = [p.x as f32, p.y as f32];
            if v.x > 0.5 {
                sky[i as usize].tex_coord[0] = sky_width as f32;
            } else {
                sky[i as usize].tex_coord[0] = 0.0;
            }

            let tex_coord_a = PIXELS_PER_UNIT * viewport.position.x / sky_size.x;
            //    let tex_coord = PIXELS_PER_UNIT * viewport.size.x / sky_size.x;
            let tex_coord = 0.5 * tex_coord_a; // + tex_coord * v.x;
            sky[i as usize].tex_coord[0] += tex_coord as f32;*/

            let v = vec_dir(i);
            let p = viewport.position + vec2(viewport.size.x * v.x, viewport.size.y * v.y);
            sky[i as usize].position = [p.x as f32, p.y as f32];

            let tex_coord_a = PIXELS_PER_UNIT * vec2(viewport.position.x / self.sky_size.x, 0.0);

            let tex_coord = PIXELS_PER_UNIT
                * vec2(
                    viewport.size.x / self.sky_size.x,
                    -viewport.size.y / self.sky_size.y,
                );
            let tex_coord = 0.5 * tex_coord_a + vec2(tex_coord.x * v.x, tex_coord.y * v.y);
            sky[i as usize].tex_coord = [tex_coord.x as f32, tex_coord.y as f32];
        }

        // FIXME: vertical inversion
        let ground = &mut self.vertices[self.ground..];
        for i in 0..4 {
            let v = vec_dir(i);
            let p = viewport.position + vec2(viewport.size.x * v.x, viewport.size.y * v.y);
            ground[i as usize].position = [p.x as f32, p.y as f32];
            ground[i as usize].tex_coord = position_to_tex_coord(p, self.ground_size);
        }
    }
}

fn position_to_tex_coord(position: Vector2<f64>, size: Vector2<f64>) -> [f32; 2] {
    let tex_coord = PIXELS_PER_UNIT * vec2(position.x / size.x, -position.y / size.y);
    [tex_coord.x as f32, tex_coord.y as f32]
}
