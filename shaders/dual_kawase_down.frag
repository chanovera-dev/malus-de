#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_texture;
uniform vec2 u_halfpixel; // 0.5 / resolución del buffer de origen
uniform float u_offset;   // Separación de muestreo

void main() {
    vec4 sum = texture(u_texture, v_tex_coords) * 4.0;
    sum += texture(u_texture, v_tex_coords - u_halfpixel.xy * u_offset);
    sum += texture(u_texture, v_tex_coords + u_halfpixel.xy * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(u_halfpixel.x, -u_halfpixel.y) * u_offset);
    sum += texture(u_texture, v_tex_coords - vec2(u_halfpixel.x, -u_halfpixel.y) * u_offset);
    frag_color = sum / 8.0;
}
