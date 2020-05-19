#version 150 core

uniform sampler2D tex;
in vec4 color;
in vec2 tex_coord;
in vec4 tex_bounds;
in float depth;
out vec4 pixel;

void main() {
    vec2 a = mod(tex_coord, 1.0);
    vec2 c = vec2(
        tex_bounds.x*(1.0 - a.x) + tex_bounds.z*a.x,
        tex_bounds.y*(1.0 - a.y) + tex_bounds.w*a.y
    );

    vec4 res = color + texture2D(tex, c);
    pixel = res;
    if (res.a > 0.5) {
        gl_FragDepth = depth;
    }
}
