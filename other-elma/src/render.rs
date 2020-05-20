use crate::atlas::Atlas;
use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use cgmath::{vec2, Vector2};
use glutin::dpi::PhysicalSize;
use std;
use std::mem::size_of;
use std::ptr;

pub struct Renderer {
    polygons: Pass,
    pictures: Pass,
    texture: GLuint,
}

struct Pass {
    vertex_buffer: GLuint,
    vertex_buffer_capacity: usize,

    index_buffer: GLuint,
    index_buffer_capacity: usize,

    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,

    displacement_uniform: GLint,
    scale_uniform: GLint,
}

/*
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    };
}*/

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct PolygonVertex {
    pub position: [f32; 2],
    pub clip: f32,
}

const POLYGON_ATTRIBUTES: &[(&str, GLint, usize)] =
    &[("in_position\0", 2, 0 * 4), ("in_clip\0", 1, 2 * 4)];

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct PictureVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub tex_bounds: [f32; 4],
    pub clip: f32,
}

const PICTURE_ATTRIBUTES: &[(&str, GLint, usize)] = &[
    ("in_position\0", 2, 0 * 4),
    ("in_tex_coord\0", 2, 2 * 4),
    ("in_tex_bounds\0", 4, 4 * 4),
    ("in_clip\0", 1, 8 * 4),
];

#[derive(Copy, Clone, Debug)]
pub struct Viewport {
    pub position: Vector2<f64>,
    pub size: Vector2<f64>,
}

impl Viewport {
    pub fn from_center_and_scale(
        center: Vector2<f64>,
        scale: f64,
        screen_size: PhysicalSize<u32>,
    ) -> Self {
        let screen_size = vec2(screen_size.width as f64, screen_size.height as f64);
        let size = screen_size / ((screen_size.x * screen_size.y).sqrt()) * scale;

        Viewport {
            position: center - size * 0.5,
            size,
        }
    }
}

unsafe fn glsl_version(gl: &Gl) /*-> (u32, u32, u32) */
{
    use std::ffi::CStr;
    use std::os::raw::c_char;

    let version = CStr::from_ptr(gl.GetString(gl::SHADING_LANGUAGE_VERSION) as *const c_char)
        .to_string_lossy();
    println!("{}", version);
}

impl Pass {
    pub unsafe fn new(gl: &Gl, vertex_shader: &str, fragment_shader: &str) -> Self {
        let mut vertex_buffer = 0;
        gl.GenBuffers(1, &mut vertex_buffer);

        let mut index_buffer = 0;
        gl.GenBuffers(1, &mut index_buffer);

        let vertex_shader = compile_shader(gl, vertex_shader, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(gl, fragment_shader, gl::FRAGMENT_SHADER);

        let program = link_program(gl, vertex_shader, fragment_shader);

        let displacement_uniform =
            gl.GetUniformLocation(program, "displacement\0".as_ptr() as *const GLchar);
        let scale_uniform = gl.GetUniformLocation(program, "scale\0".as_ptr() as *const GLchar);

        Pass {
            vertex_buffer,
            vertex_buffer_capacity: 0,
            index_buffer,
            index_buffer_capacity: 0,
            vertex_shader,
            fragment_shader,
            program,
            displacement_uniform,
            scale_uniform,
        }
    }

    unsafe fn draw<V>(
        &mut self,
        gl: &Gl,
        vertices: &Vec<V>,
        indices: &Vec<u32>,
        viewport: Viewport,
    ) {
        gl.UseProgram(self.program);
        gl.BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);

        if vertices.len() > self.vertex_buffer_capacity {
            self.vertex_buffer_capacity = vertices.capacity();
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_buffer_capacity * size_of::<V>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
        } else {
            gl.BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * size_of::<V>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
            );
        }

        if indices.len() > self.index_buffer_capacity {
            self.index_buffer_capacity = indices.capacity();
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.index_buffer_capacity * size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
        } else {
            gl.BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                (indices.len() * size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const _,
            );
        }

        gl.Uniform2f(
            self.displacement_uniform,
            (-1.0 - 2.0 * viewport.position.x / viewport.size.x) as f32,
            (-1.0 - 2.0 * viewport.position.y / viewport.size.y) as f32,
        );
        gl.Uniform2f(
            self.scale_uniform,
            (2.0 / viewport.size.x) as f32,
            (2.0 / viewport.size.y) as f32,
        );

        gl.DrawElements(
            gl::TRIANGLES,
            indices.len() as GLsizei,
            gl::UNSIGNED_INT,
            ptr::null(),
        );
    }

    pub unsafe fn cleanup(&mut self, gl: &Gl) {
        gl.DeleteProgram(self.program);
        gl.DeleteShader(self.fragment_shader);
        gl.DeleteShader(self.vertex_shader);
        gl.DeleteBuffers(2, [self.vertex_buffer, self.index_buffer].as_ptr());
    }
}

impl Renderer {
    pub unsafe fn new(gl: &Gl, atlas: &mut Atlas) -> Self {
        glsl_version(gl);

        let polygons = Pass::new(
            gl,
            include_str!("shader/polygon.vert"),
            include_str!("shader/polygon.frag"),
        );

        let pictures = Pass::new(
            gl,
            include_str!("shader/picture.vert"),
            include_str!("shader/picture.frag"),
        );

        let mut texture = 0;
        gl.GenTextures(1, &mut texture);
        gl.BindTexture(gl::TEXTURE_2D, texture);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::SRGB_ALPHA as _,
            atlas.width as _,
            atlas.height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            atlas.data.as_ptr() as *const _,
        );

        atlas.data = Vec::new();

        // Specify the layout of the vertex data
        for &(pass, attributes, stride) in &[
            (&polygons, POLYGON_ATTRIBUTES, size_of::<PolygonVertex>()),
            (&pictures, PICTURE_ATTRIBUTES, size_of::<PictureVertex>()),
        ] {
            gl.UseProgram(pass.program);
            gl.BindBuffer(gl::ARRAY_BUFFER, pass.vertex_buffer);

            for &(name, size, offset) in attributes {
                let attribute = gl.GetAttribLocation(pass.program, name.as_ptr() as *const GLchar);
                gl.EnableVertexAttribArray(attribute as GLuint);
                gl.VertexAttribPointer(
                    attribute as GLuint,
                    size,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    stride as GLsizei,
                    offset as *const _,
                );
            }
        }

        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        //    gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        //   gl.LineWidth(7.0);

        // Linear blending.
        gl.Enable(gl::FRAMEBUFFER_SRGB);

        Renderer {
            polygons,
            pictures,
            texture,
        }
    }

    pub unsafe fn draw_polygons(
        &mut self,
        gl: &Gl,
        vertices: &Vec<PolygonVertex>,
        indices: &Vec<u32>,
        viewport: Viewport,
    ) {
        if vertices.len() == 0 || indices.len() == 0 {
            return;
        }

        //  gl.DepthFunc(gl::ALWAYS);
        //     gl.DepthMask(true as _);
        self.polygons.draw(gl, vertices, indices, viewport);
    }

    pub unsafe fn draw_pictures(
        &mut self,
        gl: &Gl,
        vertices: &Vec<PictureVertex>,
        indices: &Vec<u32>,
        viewport: Viewport,
    ) {
        if vertices.len() == 0 || indices.len() == 0 {
            return;
        }

        gl.DepthFunc(gl::NOTEQUAL);
        gl.DepthMask(false as _);
        self.pictures.draw(gl, vertices, indices, viewport);
    }

    pub unsafe fn cleanup(&mut self, gl: &Gl) {
        self.polygons.cleanup(gl);
        self.pictures.cleanup(gl);
        gl.DeleteTextures(1, [self.texture].as_ptr());
    }
}

unsafe fn link_program(gl: &Gl, vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl.CreateProgram();

    gl.AttachShader(program, vertex_shader);
    gl.AttachShader(program, fragment_shader);
    gl.LinkProgram(program);

    // Get the link status.
    let mut status = gl::FALSE as GLint;
    gl.GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error.
    if status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl.GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len((log_length as usize) - 1); // subtract 1 to skip the trailing null character
        gl.GetProgramInfoLog(
            program,
            log_length,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );

        panic!(
            "Lithium OpenGL program linking failed: {}",
            std::str::from_utf8(&log)
                .ok()
                .expect("glProgramInfoLog returned invalid utf-8")
        );
    }

    program
}

unsafe fn compile_shader(gl: &Gl, source: &str, kind: GLenum) -> GLuint {
    let shader = gl.CreateShader(kind);

    // Attempt to compile the shader.
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr() as *const GLchar].as_ptr(),
        [source.len() as GLint].as_ptr(),
    );
    gl.CompileShader(shader);

    // Get the compile status.
    let mut status = gl::FALSE as GLint;
    gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error.
    if status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len((log_length as usize) - 1); // subtract 1 to skip the trailing null character
        gl.GetShaderInfoLog(
            shader,
            log_length,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );

        panic!(
            "Lithium OpenGL shader compilation failed: {}",
            std::str::from_utf8(&log)
                .ok()
                .expect("glGetShaderInfoLog returned invalid utf-8")
        );
    }

    shader
}
