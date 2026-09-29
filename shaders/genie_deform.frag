#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_window_texture; // Textura capturada de la ventana (FBO)
uniform float u_progress;           // 0.0 a 1.0

void main() {
    vec4 tex = texture(u_window_texture, v_tex_coords);
    // Atenuación sutil del brillo durante el estiramiento de absorción
    float brightness = 1.0 - (u_progress * 0.15);
    frag_color = vec4(tex.rgb * brightness, tex.a);
}
