use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use std;
use std::mem::size_of;
use std::ptr;

pub struct Renderer {
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

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub struct Viewport {
    pub position: [f32; 2],
    pub size: [f32; 2],
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
    pub unsafe fn new(gl: &Gl) -> Self {
        glsl_version(gl);

        let mut vertex_buffer = 0;
        gl.GenBuffers(1, &mut vertex_buffer);
        gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

        let mut index_buffer = 0;
        gl.GenBuffers(1, &mut index_buffer);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

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
        let hidpi_factor_uniform =
            gl.GetUniformLocation(program, "hidpi_factor\0".as_ptr() as *const GLchar);

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
        width: f64,
        height: f64,
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

        gl.Uniform2f(self.displacement_uniform, -1.0, 1.0);
        gl.Uniform2f(
            self.scale_uniform,
            2.0 / (width as f32),
            -2.0 / (height as f32),
        );

        gl.DrawElements(
            gl::TRIANGLES,
            indices.len() as GLsizei,
            gl::UNSIGNED_INT,
            ptr::null(),
        );
    }

    /*   pub unsafe fn cleanup(self, gl: &Gl) {
        gl.DeleteProgram(self.program);
        gl.DeleteShader(self.fragment_shader);
        gl.DeleteShader(self.vertex_shader);
        gl.DeleteBuffers(2, [self.vertex_buffer, self.index_buffer].as_ptr());
    }*/
}
