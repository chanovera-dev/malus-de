#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;

void main() {
    vec4 sum = vec4(0.0);
    sum += texture(u_texture, v_tex_coords + vec2(-u_halfpixel.x * 2.0, 0.0) * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(-u_halfpixel.x, u_halfpixel.y) * u_offset) * 2.0;
    sum += texture(u_texture, v_tex_coords + vec2(0.0, u_halfpixel.y * 2.0) * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(u_halfpixel.x, u_halfpixel.y) * u_offset) * 2.0;
    sum += texture(u_texture, v_tex_coords + vec2(u_halfpixel.x * 2.0, 0.0) * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(u_halfpixel.x, -u_halfpixel.y) * u_offset) * 2.0;
    sum += texture(u_texture, v_tex_coords + vec2(0.0, -u_halfpixel.y * 2.0) * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(-u_halfpixel.x, -u_halfpixel.y) * u_offset) * 2.0;
    frag_color = sum / 12.0;
}
