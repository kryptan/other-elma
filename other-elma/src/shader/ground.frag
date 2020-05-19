#version 150 core

in vec4 color;
in vec2 tex_coord;
in vec4 tex_bounds;
out vec4 pixel;

void main() {
    pixel = color;
}
