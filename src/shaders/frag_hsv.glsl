#version 440

uniform sampler2D overlay;

uniform sampler2D colour_map1;
uniform sampler2D colour_map2;
uniform float interpolation;

in vec3 f_normal;
in vec2 f_tex;

out vec4 colour;

vec4 hsv_to_rgb(in float hue) {
    float h = hue * 100;
    float x  = 1.0 - abs(mod(h / 20.0, 2.0) - 1.0);
    vec3 colour;
    if (h >= 0 && h < 20) {
        colour = vec3(1, x, 0);
    }
    else if (h >= 20 && h < 40) {
        colour = vec3(x, 1, 0);
    }
    else if (h >= 40 && h < 60) {
        colour = vec3(0, 1, x);
    }
    else if (h >= 60 && h < 80) {
        colour = vec3(0, x, 1);
    }
    else if (h >= 80 && h < 100) {
        colour = vec3(x, 0, 1);
    }
    else if (h > 100){
        colour = vec3(1, 0, 1);
    }
    else {
        colour = vec3(1, 0, 0);
    }
    return(vec4(colour, 1.0));
}

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

    image_colour1 = overlay_colour.x * hsv_to_rgb(1.0 - image_colour1.x);
    image_colour2 = overlay_colour.x * hsv_to_rgb(1.0 - image_colour2.x);

    colour = mix(image_colour1, image_colour2, interpolation);
}