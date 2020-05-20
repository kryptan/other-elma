#version 150 core

uniform sampler2D atlas;
in vec2 tex_coord;
in vec4 tex_bounds;
out vec4 pixel;

void main() {
    vec2 coord = mod(tex_coord, 1.0);
    coord = tex_bounds.xy*(1.0 - coord.xy) + tex_bounds.zw*coord.xy;

    pixel = texture2D(atlas, coord);
}
