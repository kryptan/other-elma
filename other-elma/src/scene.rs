use crate::atlas::{Atlas, Sprite};
use crate::render::{PictureVertex, PolygonVertex, Viewport};
use crate::triangulation::triangulate;
use cgmath::{vec2, Vector2};
use elma::lev::Level;
use elma::Clip;
use lyon_tessellation::VertexBuffers;

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

        let polygons = triangulate(&level);

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
        };

        scene.sky = scene.add_image(sky_texture, vec2(0.0, 0.0), Clip::Sky);
        scene.ground = scene.add_image(ground_texture, vec2(0.0, 0.0), Clip::Ground);

        level.pictures.sort_by_key(|picture| picture.distance);
        for pic in level.pictures.iter().rev() {
            if pic.name.is_empty() {
                continue;
            }

            let pic2 = atlas.get(&pic.name);
            scene.add_image(pic2, vec2(pic.position.x, pic.position.y), pic.clip);
        }

        scene
    }

    pub fn add_image(&mut self, sprite: &Sprite, position: Vector2<f64>, clip: Clip) -> usize {
        let v = self.vertices.len() as u32;

        for i in 0..4 {
            let v = vec_dir(i);
            let p = position
                + (1.0 / PIXELS_PER_UNIT) * vec2(v.x * sprite.size.x, -v.y * sprite.size.y);

            self.vertices.push(PictureVertex {
                position: [p.x as f32, p.y as f32],
                tex_coord: [v.x as f32, v.y as f32],
                tex_bounds: sprite.bounds,
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
                    viewport.size.y / self.sky_size.y,
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

            let tex_coord_a = PIXELS_PER_UNIT
                * vec2(
                    viewport.position.x / self.ground_size.x,
                    viewport.position.y / self.ground_size.y,
                );

            let tex_coord = PIXELS_PER_UNIT
                * vec2(
                    viewport.size.x / self.ground_size.x,
                    viewport.size.y / self.ground_size.y,
                );
            let tex_coord = tex_coord_a + vec2(tex_coord.x * v.x, tex_coord.y * v.y);
            ground[i as usize].tex_coord = [tex_coord.x as f32, tex_coord.y as f32];
        }
    }
}
