#version 150 core

uniform sampler2D atlas;
in vec2 tex_coord;
in vec4 tex_bounds;
in vec2 mask;
out vec4 pixel;

void main() {
    vec2 coord = mod(tex_coord, 1.0);
    coord = tex_bounds.xy*(1.0 - coord.xy) + tex_bounds.zw*coord.xy;
    vec4 result = texture2D(atlas, coord);

    if (mask.x > 0) {
        result.a *= texture2D(atlas, mask).a;
    }

    pixel = result;
}
