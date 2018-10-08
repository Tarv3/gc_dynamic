#version 440 

in float f_hue;
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
    else if (h >= 100){
        colour = vec3(1, 0, 1);
    }
    else {
        colour = vec3(1, 0, 0);
    }
    return(vec4(colour, 1.0));
}

void main() {
    colour = hsv_to_rgb(1.0 - f_hue);
}