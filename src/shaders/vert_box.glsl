#version 440 

uniform vec2 translation;
uniform vec2 scale;

in vec2 position;
in float hue;

out float f_hue;

void main() {
    f_hue = hue;
    gl_Position = vec4(position * scale + translation, 0.0, 1.0);
}