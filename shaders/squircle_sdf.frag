#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform vec2 u_size;           // Dimensiones de la superficie (ancho, alto en px)
uniform float u_corner_radius; // Radio de curvatura (ej. 24.0)
uniform vec4 u_bg_color;       // Color de fondo
uniform vec4 u_border_color;   // Color del borde reflectante
uniform float u_border_width;  // Grosor del borde en px

// Distancia con signo analítica a un squircle superelíptico (n ≈ 4.2)
float squircle_sdf(vec2 p, vec2 b, float r) {
    vec2 d = abs(p) - b + vec2(r);
    if (d.x <= 0.0 || d.y <= 0.0) {
        return max(d.x, d.y) - r;
    }
    // Aproximación superelíptica n=4.2
    return pow(pow(d.x, 4.2) + pow(d.y, 4.2), 1.0 / 4.2) - r;
}

void main() {
    vec2 half_size = u_size * 0.5;
    vec2 p = (v_tex_coords - 0.5) * u_size;

    float dist = squircle_sdf(p, half_size, u_corner_radius);
    
    // Suavizado subpixel analítico (anti-aliasing)
    float edge_aa = fwidth(dist);
    float alpha = 1.0 - smoothstep(-edge_aa, 0.0, dist);

    // Contorno / borde reflectante
    if (u_border_width > 0.0) {
        float border_mask = smoothstep(-u_border_width - edge_aa, -u_border_width, dist);
        vec4 col = mix(u_bg_color, u_border_color, border_mask);
        frag_color = col * alpha;
    } else {
        frag_color = u_bg_color * alpha;
    }
}
