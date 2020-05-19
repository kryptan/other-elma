use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use crate::texture::Texture;
use cgmath::{vec2, Vector2};
use glutin::dpi::PhysicalSize;
use std;
use std::mem::size_of;
use std::ptr;

pub struct Renderer {
    vertex_buffer: GLuint,
    vertex_buffer_capacity: usize,

    index_buffer: GLuint,
    index_buffer_capacity: usize,

    texture: GLuint,

    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,

    displacement_uniform: GLint,
    scale_uniform: GLint,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
    pub tex_bounds: [f32; 4],
}

#[derive(Debug)]
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

unsafe fn glsl_version(gl: &Gl) /*-> (u32, u32, u32) */
{
    use std::ffi::CStr;
    use std::os::raw::c_char;

    let version = CStr::from_ptr(gl.GetString(gl::SHADING_LANGUAGE_VERSION) as *const c_char)
        .to_string_lossy();
    println!("{}", version);
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    };
}

impl Renderer {
    pub unsafe fn new(gl: &Gl, tex: &mut Texture) -> Self {
        glsl_version(gl);

        let mut vertex_buffer = 0;
        gl.GenBuffers(1, &mut vertex_buffer);
        gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

        let mut index_buffer = 0;
        gl.GenBuffers(1, &mut index_buffer);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

        let mut texture = 0;
        gl.GenTextures(1, &mut texture);
        gl.BindTexture(gl::TEXTURE_2D, texture);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::SRGB_ALPHA as _, // FIXME: use sgrb
            tex.tex_width as _,
            tex.tex_height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            tex.texture.as_ptr() as *const _,
        );

        tex.texture.clear();

        let vertex_shader =
            compile_shader(gl, include_str!("shader/ground.vert"), gl::VERTEX_SHADER);
        let fragment_shader =
            compile_shader(gl, include_str!("shader/ground.frag"), gl::FRAGMENT_SHADER);
        // let vertex_shader = compile_shader(gl, lithium::shader::GLES_100_VERTEX, gl::VERTEX_SHADER);
        // let fragment_shader = compile_shader(gl, lithium::shader::GLES_100_FRAGMENT, gl::FRAGMENT_SHADER);
        let program = link_program(gl, vertex_shader, fragment_shader);

        gl.UseProgram(program);

        // Specify the layout of the vertex data
        let attributes = [
            ("in_pos\0", 2, offset_of!(Vertex, position)),
            ("in_color\0", 4, offset_of!(Vertex, color)),
            ("in_tex_coord\0", 2, offset_of!(Vertex, tex_coord)),
            ("in_tex_bounds\0", 4, offset_of!(Vertex, tex_bounds)),
        ];

        for &(name, size, offset) in &attributes {
            let attribute = gl.GetAttribLocation(program, name.as_ptr() as *const GLchar);
            gl.EnableVertexAttribArray(attribute as GLuint);
            gl.VertexAttribPointer(
                attribute as GLuint,
                size,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                size_of::<Vertex>() as GLsizei,
                offset as *const _,
            );
        }

        let displacement_uniform =
            gl.GetUniformLocation(program, "displacement\0".as_ptr() as *const GLchar);
        let scale_uniform = gl.GetUniformLocation(program, "scale\0".as_ptr() as *const GLchar);

        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        //    gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        //   gl.LineWidth(7.0);

        // Linear blending.
        gl.Enable(gl::FRAMEBUFFER_SRGB);

        Renderer {
            vertex_buffer,
            vertex_buffer_capacity: 0,

            index_buffer,
            index_buffer_capacity: 0,

            texture,

            vertex_shader,
            fragment_shader,
            program,

            displacement_uniform,
            scale_uniform,
        }
    }

    pub unsafe fn draw_batch(
        &mut self,
        gl: &Gl,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>,
        viewport: Viewport,
    ) {
        if vertices.len() == 0 || indices.len() == 0 {
            return;
        }

        if vertices.len() > self.vertex_buffer_capacity {
            self.vertex_buffer_capacity = vertices.capacity();
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_buffer_capacity * size_of::<Vertex>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
        } else {
            gl.BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * size_of::<Vertex>()) as GLsizeiptr,
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

        /*
                gl_Position = displacement + in_pos*scale;

                gl_Position(top_right) = (1, -1)
                gl_Position(bottom_left) = (-1, 1)

                in_pos(top_right) = viewport.position + viewport.size
                in_pos(bottom_left) = viewport.position

                (1, -1) = displacement + (viewport.position + viewport.size)*scale
                (-1, 1) = displacement + (viewport.position)*scale
        ------------------------
                1 = displacement.x + (viewport.position.x + viewport.size.x)*scale.x
                -1 = displacement.x + (viewport.position.x)*scale.x

                -1 = displacement.y + (viewport.position.y + viewport.size.y)*scale.y
                1 = displacement.y + (viewport.position.y)*scale.y
                -----------------
                1 = displacement.x + viewport.position.x*scale.x + viewport.size.x*scale.x
                -1 = displacement.x + viewport.position.x*scale.x

                -1 = displacement.y + viewport.position.y*scale.y + viewport.size.y*scale.y
                1 = displacement.y + viewport.position.y*scale.y
                -----------------
                2 = viewport.size.x*scale.x
                -2 = viewport.size.y*scale.y
                -----------------
                scale.x = 2.0/viewport.size.x
                scale.y = -2.0/viewport.size.y
                -----------------
                -1 = displacement.x + viewport.position.x*2.0/viewport.size.x
                1 = displacement.y + viewport.position.y*-2.0/viewport.size.y
                -----------------
                displacement.x = -1 - 2.0*viewport.position.x/viewport.size.x
                displacement.y = 1 + 2.0*viewport.position.y/viewport.size.y

                */

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
        gl.DeleteTextures(1, [self.texture].as_ptr());
    }
}

/*
impl Drop for Renderer {
}
*/
