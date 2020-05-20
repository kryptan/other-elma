#version 150 core

uniform vec2 displacement;
uniform vec2 scale;

in vec2 in_position;
in vec2 in_tex_coord;
in vec4 in_tex_bounds;
in float in_clip;

out vec2 tex_coord;
out vec4 tex_bounds;

void main() {
    tex_coord = in_tex_coord;
    tex_bounds = in_tex_bounds;
    gl_Position = vec4(displacement + in_position*scale, in_clip, 1.0);
}
