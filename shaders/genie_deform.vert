#version 300 es
precision highp float;

// Coordenadas del vértice en la malla rectangular normalizada [0, 1]
layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec2 a_tex_coords;

out vec2 v_tex_coords;

uniform mat4 u_matrix;
uniform float u_progress;      // Progreso de la animación [0.0 = ventana completa, 1.0 = minimizado en el dock]
uniform vec4 u_window_rect;    // (x, y, ancho, alto) de la ventana en pantalla
uniform vec4 u_dock_slot_rect; // (x, y, ancho, alto) de la ranura de destino en el Dock

// Curva sigmoide suave
float s_curve(float t) {
    return smoothstep(0.0, 1.0, t);
}

void main() {
    v_tex_coords = a_tex_coords;

    // Altura relativa del vértice en la ventana original (0.0 = base, 1.0 = tope)
    float vertex_y_factor = 1.0 - a_pos.y;

    // Desfase no lineal en la deformación: la base de la ventana desciende primero hacia el Dock
    float delay = (1.0 - vertex_y_factor) * 0.45;
    float local_t = clamp((u_progress - delay) / (1.0 - delay + 0.0001), 0.0, 1.0);
    float t = s_curve(local_t);

    // Interpolación no lineal en Y: desciende hacia la posición Y del Dock
    float current_y = mix(u_window_rect.y + a_pos.y * u_window_rect.w, 
                          u_dock_slot_rect.y + a_pos.y * u_dock_slot_rect.w, 
                          t);

    // Estrechamiento en X hacia la ranura del Dock formando la curvatura cóncava de lámpara de Aladín
    float current_width = mix(u_window_rect.z, u_dock_slot_rect.z, t);
    float current_center_x = mix(u_window_rect.x + u_window_rect.z * 0.5,
                                 u_dock_slot_rect.x + u_dock_slot_rect.z * 0.5,
                                 t);

    float current_x = current_center_x + (a_pos.x - 0.5) * current_width;

    gl_Position = u_matrix * vec4(current_x, current_y, 0.0, 1.0);
}
