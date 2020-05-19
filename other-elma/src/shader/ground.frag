#version 150 core

uniform sampler2D tex;
in vec4 color;
in vec2 tex_coord;
in vec4 tex_bounds;
out vec4 pixel;

void main() {
    pixel = color + texture2D(tex, tex_coord);
}
