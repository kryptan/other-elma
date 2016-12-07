extern crate elma;
#[macro_use] extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;
extern crate lyon_tessellation;
extern crate lyon_core;
extern crate lyon_path;
extern crate lyon_path_builder;
extern crate lyon_path_iterator;

use gfx::traits::FactoryExt;
use gfx::Device;
use elma::lev::Level;

mod triangulation;

type Interval = u32;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

enum Action {
    ThrottleSwitch,
    Flip,
    CounterClockwiseVolt,
    ClockwiseVolt,
    ClockwiseAloVolt,
}

struct TimedAction {
    delta : Interval,
    action : Action,
}

type ActionSequence = Vec<TimedAction>;

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
        include_bytes!("shader/triangle_150.glslv"),
        include_bytes!("shader/triangle_150.glslf"),
        pipe::new()
    ).unwrap();

    let mut size_in_pixels = (w, h);

    let level = Level::load("E:/d/games/ElastoMania/Lev/0lp25.lev").unwrap();

    let vertices = triangulation::triangulate(&level);

    println!("{:?}", vertices.vertices);

    let vertex_buffer = factory.create_vertex_buffer(&vertices.vertices);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color
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

        // draw a frame
        encoder.clear(&data.out, [0.2, 0.3, 0.4, 0.7]);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
