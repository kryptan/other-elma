use elma::lev::Level;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod render;
mod triangulation;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use cgmath::Vector2;
use gl::types::*;

/*
mod gles {
    include!(concat!(env!("OUT_DIR"), "/gles_bindings.rs"));
}*/

fn main() {
    let level = Level::load("E:/d/games/ElastoMania/Lev/0lp25.lev").unwrap();
    let vertices = triangulation::triangulate(&level);
    let indices: Vec<u32> = vertices.indices.iter().map(|&i| i as u32).collect();

    let events_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Elastomania")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop)
        .unwrap();

    let mut size = windowed_context.window().inner_size();
    let mut scale_factor = windowed_context.window().scale_factor();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|name| windowed_context.get_proc_address(name) as *const _);
    //  let _gles = gles::Gles2::load_with(|name| self.window.context().get_proc_address(name) as *const _);

    let mut renderer = unsafe { render::Renderer::new(&gl) };

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut close = false;
        let mut resize = false;

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
                println!("new size = {:?}", new_size);
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
                println!("new hidpi_factor = {}", scale_factor);
                resize = true;
            }
            Event::WindowEvent { event, .. } => {
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
                    Vector2 { x: 0.0, y: 0.0 },
                    100.0,
                    size,
                );

                // Render batches.
                unsafe { renderer.draw_batch(&gl, &vertices.vertices, &indices, viewport) };

                windowed_context.swap_buffers().unwrap(); // FIXME: handle error
            }
            _ => {}
        };

        if resize {
            windowed_context.resize(size);
            unsafe { gl.Viewport(0, 0, size.width as GLsizei, size.height as GLsizei) };
        }

        if close {
            println!("close");
            *control_flow = ControlFlow::Exit;
            return;
        }
    });

    //  unsafe { renderer.cleanup(&gl) };
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
