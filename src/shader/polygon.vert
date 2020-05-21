#version 150 core

uniform vec2 displacement;
uniform vec2 scale;

in vec2 in_position;
in float in_clip;

void main() {
    gl_Position = vec4(displacement + in_position*scale, in_clip, 1.0);
}
