#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_screen_texture; // Textura completa del framebuffer de salida
uniform vec2 u_resolution;          // Resolución física del monitor
uniform float u_screen_radius;      // Radio de curvatura físico (ej. 24.0 px)

// Máscara analítica para recortar las 4 esquinas de pantalla estilo squircle superelíptico (n=4.2)
float screen_corner_mask(vec2 p, vec2 half_res, float r) {
    vec2 d = abs(p) - half_res + vec2(r);
    if (d.x <= 0.0 || d.y <= 0.0) {
        return 1.0;
    }
    vec2 md = max(d, 0.0);
    float dist = pow(pow(md.x, 4.2) + pow(md.y, 4.2), 1.0 / 4.2) - r;
    float aa = fwidth(dist);
    return 1.0 - smoothstep(-aa, 0.0, dist);
}

void main() {
    vec4 base_color = texture(u_screen_texture, v_tex_coords);
    vec2 p = (v_tex_coords - 0.5) * u_resolution;
    vec2 half_res = u_resolution * 0.5;

    float mask = screen_corner_mask(p, half_res, u_screen_radius);
    // En las esquinas exteriores recorta a negro puro
    frag_color = mix(vec4(0.0, 0.0, 0.0, 1.0), base_color, mask);
}
