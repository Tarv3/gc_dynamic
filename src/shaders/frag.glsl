#version 440

uniform sampler2D overlay;

uniform sampler2D colour_map1;
uniform sampler2D colour_map2;
uniform float interpolation;

in vec3 f_normal;
in vec2 f_tex;

out vec4 colour;

void main() {
    vec3 light = normalize(vec3(1, 1, 1));
    float brightness = dot(light, normalize(f_normal));
    brightness = max(0, brightness);

    ivec2 tex_size1 = textureSize(colour_map1, 0);
    ivec2 coords1 = ivec2(f_tex * tex_size1);

    ivec2 tex_size2 = textureSize(colour_map2, 0);
    ivec2 coords2 = ivec2(f_tex * tex_size2);

    ivec2 overlay_size = textureSize(overlay, 0);
    ivec2 overlay_coords = ivec2(f_tex * overlay_size);

    vec4 image_colour1 = texelFetch(colour_map1, coords1, 0);
    vec4 image_colour2 = texelFetch(colour_map2, coords2, 0);
    vec4 overlay_colour = texelFetch(overlay, overlay_coords, 0);

    image_colour1 = image_colour1;
    image_colour2 = image_colour2;

    colour = mix(image_colour1, image_colour2, interpolation);
}