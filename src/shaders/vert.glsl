#version 440

uniform sampler2D height_map;
uniform mat4 view;
uniform vec3 eye;
uniform mat4 rotation;
uniform float height_scale;

in vec3 position;
in vec3 normal;
in vec2 tex_coord;

out vec3 f_normal;
out vec2 f_tex;

void main() {
    float height = texture(height_map, tex_coord).x;
    f_normal = mat3(rotation) * normal;
    f_tex = tex_coord;
    gl_Position = view * vec4((1.0 + height * height_scale) * position, 1.0);
}