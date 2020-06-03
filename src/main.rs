#![feature(bool_to_option)]
use crate::atlas::Atlas;
use crate::physics::{Control, Events, Moto, Segments};
use crate::scene::Scene;
use cgmath::vec2;
use elma::lev::Level;
use elma::rec::EventType;
use gl::types::*;
use glutin::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::time::{Duration, Instant};

mod atlas;
mod bike;
mod physics;
mod render;
mod scene;
mod transform;
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

fn main() {
    let mut game_state = GameState::new("D:/games/Elma Online/Lev/0LP05.lev");
    // dbg!(&game_state.level.objects);
    // dbg!(&game_state.level.ground);

    let mut atlas = Atlas::new("D:/games/ElastoMania/lgr/default.lgr");
    let mut scene = Scene::new(&mut game_state.level, &atlas);

    let moto = scene.add_moto(&atlas, false);

    let events_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Elastomania")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        // .with_multisampling(0)
        .with_depth_buffer(8)
        .build_windowed(window_builder, &events_loop)
        .unwrap();

    let mut size = windowed_context.window().inner_size();
    let mut scale_factor = windowed_context.window().scale_factor();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|name| windowed_context.get_proc_address(name) as *const _);
    //  let _gles = gles::Gles2::load_with(|name| self.window.context().get_proc_address(name) as *const _);

    let mut renderer = unsafe { render::Renderer::new(&gl, &mut atlas) };
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
                    gl.DepthMask(true as _);
                    gl.ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl.ClearDepth(1.0);
                    gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                let viewport = render::Viewport::from_center_and_scale(
                    game_state.moto.bike.position,
                    10.0,
                    size,
                );

                scene.animate(time);
                scene.update(viewport);

                bike::render_moto(&mut scene, &moto, &game_state.moto);

                unsafe {
                    renderer.draw_polygons(
                        &gl,
                        &scene.polygons.vertices,
                        &scene.polygons.indices,
                        viewport,
                    );
                    renderer.draw_pictures(&gl, &scene.vertices, &scene.indices, viewport);
                };

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

            // FIXME: move?
            unsafe {
                renderer.cleanup(&gl);
            }
            return;
        }
    });
}
