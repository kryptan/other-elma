#version 150 core

uniform vec2 displacement;
uniform vec2 scale;

in vec2 in_pos;
in vec4 in_color;
in vec2 in_tex_coord;
in vec4 in_tex_bounds;
in float in_depth;

out vec4 color;
out vec2 tex_coord;
out vec4 tex_bounds;
out float depth;

void main() {
    color = in_color;
    tex_coord = in_tex_coord;
    tex_bounds = in_tex_bounds;
    depth = in_depth*0.001*0.95 + 0.002;
    gl_Position = vec4(displacement + in_pos*scale, 0.0, 1.0);
}
