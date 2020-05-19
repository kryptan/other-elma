use crate::render::Vertex;
use crate::texture::{Pic, Texture};
use cgmath::{vec2, Vector2};
use elma::lev::Level;
use elma::rec::EventType;
use elma_physics::{Control, Events, Moto, Object, Segments};
use gl::types::*;
use glutin::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use lyon_tessellation::VertexBuffers;
use std::time::{Duration, Instant};

mod render;
mod texture;
mod triangulation;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

/*
mod gles {
    include!(concat!(env!("OUT_DIR"), "/gles_bindings.rs"));
}*/

struct GameState {
    moto: Moto,
    level: Level,
    segments: Segments,
}

impl GameState {
    fn new(path: &str) -> GameState {
        let level = Level::load(path).unwrap();

        let player = level
            .objects
            .iter()
            .find(|object| object.is_player())
            .unwrap();

        let moto = Moto::new(vec2(player.position.x, player.position.y));
        let segments = Segments::new(&level.polygons);

        GameState {
            moto,
            level,
            segments,
        }
    }
}

struct E;
impl Events for E {
    fn event(&mut self, _kind: EventType) {
        //  dbg!(kind);
    }
}

struct Scene {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Scene {
    fn add_image(&mut self, pic: &Pic, position: Vector2<f64>) -> usize {
        let v = self.vertices.len() as u32;

        for i in 0..4 {
            let v = match i {
                0 => vec2(0.0, 0.0),
                1 => vec2(1.0, 0.0),
                2 => vec2(1.0, 1.0),
                3 => vec2(0.0, 1.0),
                _ => unreachable!(),
            };
            let p = position + 0.021 * vec2(v.x * pic.size.x, -v.y * pic.size.y); // FIXME: the exact coefficient isn't known

            self.vertices.push(Vertex {
                position: [p.x as f32, p.y as f32],
                color: [0.0, 0.0, 0.0, 0.0],
                tex_coord: [v.x as f32, v.y as f32],
                tex_bounds: pic.bounds,
            });
        }

        self.indices
            .extend_from_slice(&[v, v + 1, v + 2, v, v + 2, v + 3]);

        v as usize
    }
}

fn main() {
    let mut game_state = GameState::new("E:/d/games/ElastoMania/Lev/Olliz055.lev");

    let mut texture = Texture::new("E:/d/games/ElastoMania/lgr/default.lgr");
    let ground_texture = texture.get(&(game_state.level.ground.clone() + ".pcx"));

    let VertexBuffers { vertices, indices } = triangulation::triangulate(&game_state.level);
    let mut scene = Scene { vertices, indices };

    for pic in &game_state.level.pictures {
        if pic.name.is_empty() {
            continue;
        }

        println!("picture = {}", pic.name);
        let pic2 = texture.get(&(pic.name.clone() + ".pcx"));
        scene.add_image(pic2, vec2(pic.position.x, pic.position.y));
    }

    let wheel_pic = texture.get("Q1WHEEL.pcx");
    let bike = scene.add_image(wheel_pic, vec2(0.0, 0.0));
    let wheels = [
        scene.add_image(wheel_pic, vec2(0.0, 0.0)),
        scene.add_image(wheel_pic, vec2(0.0, 0.0)),
    ];

    let events_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Elastomania")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        // .with_multisampling(0)
        .build_windowed(window_builder, &events_loop)
        .unwrap();

    let mut size = windowed_context.window().inner_size();
    let mut scale_factor = windowed_context.window().scale_factor();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|name| windowed_context.get_proc_address(name) as *const _);
    //  let _gles = gles::Gles2::load_with(|name| self.window.context().get_proc_address(name) as *const _);

    let mut renderer = unsafe { render::Renderer::new(&gl, &mut texture) };
    let time = Instant::now();
    let mut control = Control::default();
    let mut next_frame_time = Instant::now();

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        let now = Instant::now();
        if now > next_frame_time {
            windowed_context.window().request_redraw();
            next_frame_time = now + Duration::from_millis(20);
        }

        let mut close = false;
        let mut resize = false;

        let time = time.elapsed().as_secs_f64();
        game_state
            .moto
            .advance(control, time * 0.4368, &game_state.segments, &mut E);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                close = true;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;
                resize = true;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: new_scale_factor,
                        new_inner_size,
                    },
                ..
            } => {
                scale_factor = new_scale_factor;
                size = *new_inner_size;
                resize = true;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(key) = input.virtual_keycode {
                    let state = input.state == ElementState::Pressed;
                    match key {
                        VirtualKeyCode::Left => control.rotate_left = state,
                        VirtualKeyCode::Right => control.rotate_right = state,
                        VirtualKeyCode::Up => control.throttle = state,
                        VirtualKeyCode::Down => control.brake = state,
                        VirtualKeyCode::Space if state => {
                            game_state.moto.direction = !game_state.moto.direction
                        }
                        _ => {}
                    }
                }
            }
            Event::WindowEvent { event: _event, .. } => {
                //    dbg!(event);
            }
            Event::RedrawRequested(_) => {
                // let width = size.width as f64 / scale_factor;
                // let height = size.height as f64 / scale_factor;

                unsafe {
                    gl.ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl.Clear(gl::COLOR_BUFFER_BIT);
                }

                let viewport = render::Viewport::from_center_and_scale(
                    game_state.moto.bike.position,
                    15.0,
                    size,
                );

                for i in 0..2 {
                    object_to_vertices(
                        &game_state.moto.wheels[i],
                        &mut scene.vertices[wheels[i]..],
                    );
                }
                object_to_vertices(&game_state.moto.bike, &mut scene.vertices[bike..]);

                unsafe { renderer.draw_batch(&gl, &scene.vertices, &scene.indices, viewport) };

                windowed_context.swap_buffers().unwrap(); // FIXME: handle error
            }
            _ => {}
        };

        if resize {
            windowed_context.resize(size);
            unsafe { gl.Viewport(0, 0, size.width as GLsizei, size.height as GLsizei) };
        }

        if close {
            *control_flow = ControlFlow::Exit;
            unsafe {
                renderer.cleanup(&gl);
            }
            return;
        }
    });

    //  unsafe { renderer.cleanup(&gl) };
}

fn object_to_vertices(object: &Object, vertices: &mut [Vertex]) {
    let (sin, cos) = object.angular_position.sin_cos();
    let v = 0.4 * 2.0f64.sqrt() * vec2(cos, sin);
    let pos = [
        object.position - v,
        object.position + vec2(v.y, -v.x),
        object.position + v,
        object.position + vec2(-v.y, v.x),
    ];

    for i in 0..4 {
        vertices[i].position = [pos[i].x as f32, pos[i].y as f32];
    }
}

/*

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "in_pos",
        color: [f32; 3] = "in_color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "pixel",
        displacement: gfx::Global<[f32; 2]> = "displacement",
        scale: gfx::Global<[f32; 2]> = "scale",
    }
}

fn main() {
    use gfx::Factory;

    println!("Hello, world!");

    let w = 1024;
    let h = 768;

    let builder = glutin::WindowBuilder::new()
        .with_title("Elastomania".to_string())
        .with_dimensions(w as u32, h as u32)
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) = gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        include_bytes!("shader/ground.glslv"),
        include_bytes!("shader/ground.glslf"),
        pipe::new()
    ).unwrap();

    let mut size_in_pixels = (w, h);
    let scale = 30.0;

    let level = Level::load("E:/d/games/ElastoMania/Lev/0lp25.lev").unwrap();

    let vertices = triangulation::triangulate(&level);

    println!("{:?}", vertices.vertices);

    let vertex_buffer = factory.create_vertex_buffer(&vertices.vertices);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
        displacement: [0.0, 0.0],
        scale: [scale/(size_in_pixels.0 as f32), scale/(size_in_pixels.1 as f32)],
    };

    let index_buffer = factory.create_buffer_const(&vertices.indices, gfx::BufferRole::Index, gfx::Bind::empty()).unwrap();

    let slice = gfx::Slice {
        start: 0,
        end: vertices.indices.len() as u32,
        base_vertex: 0,
        instances: None,
        buffer: gfx::IndexBuffer::Index16(index_buffer),
    };

    'main: loop {
        // loop over events
        for event in window.poll_events() {
            match event {
                //   glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                glutin::Event::Resized(w, h) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    size_in_pixels = (w as i32, h as i32);
                },
                _ => {},
            }
        }

        data.displacement = [0.0, 0.0];
        data.scale = [scale/(size_in_pixels.0 as f32), scale/(size_in_pixels.1 as f32)];

        // draw a frame
        encoder.clear(&data.out, [0.2, 0.3, 0.4, 0.7]);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}*/
