use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
use std::collections::BTreeSet;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Avoid bloating executable by only including function names we need.
    let functions: BTreeSet<&'static str> = [
        "AttachShader",
        "BindBuffer",
        "BindTexture",
        "BlendFunc",
        "BufferData",
        "BufferSubData",
        "Clear",
        "ClearColor",
        "CompileShader",
        "CreateProgram",
        "CreateShader",
        "DeleteBuffers",
        "DeleteProgram",
        "DeleteShader",
        "DrawElements",
        "Enable",
        "EnableVertexAttribArray",
        "GenBuffers",
        "GetAttribLocation",
        "GenTextures",
        "GetProgramInfoLog",
        "GetProgramiv",
        "GetShaderInfoLog",
        "GetShaderiv",
        "GetString",
        "GetUniformLocation",
        "LineWidth",
        "LinkProgram",
        "PolygonMode",
        "TexParameteri",
        "TexImage2D",
        "ShaderSource",
        "Uniform1f",
        "Uniform2f",
        "UseProgram",
        "VertexAttribPointer",
        "Viewport",
    ]
    .iter()
    .cloned()
    .collect();

    let mut file = File::create(&Path::new(&out_dir).join("gl_bindings.rs")).unwrap();
    let mut registry = Registry::new(Api::Gl, (3, 0), Profile::Core, Fallbacks::All, []);
    registry.cmds = registry
        .cmds
        .into_iter()
        .filter(|cmd| functions.contains(cmd.proto.ident.as_str()))
        .collect();
    registry.write_bindings(StructGenerator, &mut file).unwrap();

    /*  let functions: BTreeSet<&'static str> = [
        "Clear",
        "Enable",
        "BlendFunc",
        "Uniform1f",
        "Uniform2f",
        "BufferData",
        "UseProgram",
        "BindBuffer",
        "GenBuffers",
        "ClearColor",
        "LinkProgram",
        "GetShaderiv",
        "AttachShader",
        "CreateProgram",
        "CreateShader",
        "ShaderSource",
        "GetProgramiv",
        "DeleteShader",
        "DrawElements",
        "CompileShader",
        "DeleteBuffers",
        "DeleteProgram",
        "BufferSubData",
        "GetShaderInfoLog",
        "GetProgramInfoLog",
        "GetAttribLocation",
        "GetUniformLocation",
        "VertexAttribPointer",
        "EnableVertexAttribArray",
    ].into_iter().cloned().collect();

    let mut file = File::create(&Path::new(&out_dir).join("gles_bindings.rs")).unwrap();
    Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, []);
    registry.cmds = registry.cmds.into_iter().filter(|cmd| functions.contains(cmd.proto.ident.as_str())).collect();
    registry.write_bindings(StructGenerator, &mut file).unwrap();*/
}
