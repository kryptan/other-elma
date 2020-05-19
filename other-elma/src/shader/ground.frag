#version 150 core

uniform sampler2D tex;
in vec4 color;
in vec2 tex_coord;
in vec4 tex_bounds;
out vec4 pixel;

void main() {
    vec2 a = mod(tex_coord, 1.0);
    vec2 c = vec2(
        tex_bounds.x*(1.0 - a.x) + tex_bounds.z*a.x,
        tex_bounds.y*(1.0 - a.y) + tex_bounds.w*a.y
    );

    pixel = color + texture2D(tex, c);
}
