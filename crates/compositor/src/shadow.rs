//! Pipeline de sombras proyectadas de ventanas (Window Drop Shadows) aceleradas por GPU.
//!
//! Genera sombras volumétricas ultra suaves estilo macOS con modelo analítico SDF
//! siguiendo exactamente la superelipse continua (Squircle n=4.2) de cada ventana.
//!
//! Características:
//! - Modelo de doble lóbulo (Dual-Lobe): sombra de contacto nítida + penumbra difusa ambiental.
//! - Recorte analítico interno (Inner Cutout) para preservar transparencia y acrílico.
//! - Diferenciación dinámica entre ventana activa (enfocada) e inactivas.
//! - Cero asignación de texturas intermedias y renderizado a 60/120 FPS constante.

use smithay::{
    backend::renderer::gles::ffi,
    utils::{Physical, Rectangle, Size},
};
use tracing::{error, info};

const SHADOW_VERT_SRC: &str = r#"#version 300 es
precision highp float;

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

const SHADOW_FRAG_SRC: &str = r#"#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 frag_color;

uniform vec2 u_quad_size;       // Dimensiones físicas del quad de sombra
uniform vec2 u_win_size;        // Dimensiones físicas de la ventana visible
uniform vec2 u_pad;             // Margen de expansión (pad_x, pad_y_top)
uniform float u_corner_radius;  // Radio squircle de la ventana (ej. 32.0 px)
uniform float u_exponent;       // Exponente superelíptico (4.2 continuo)
uniform float u_blur_radius;    // Radio de dispersión gaussiana
uniform float u_spread;         // Expansión sólida de la sombra
uniform float u_offset_y;       // Desplazamiento de luz cenital (luz hacia abajo)
uniform float u_opacity;        // Opacidad pico de la sombra

// Distancia con signo analítica a un squircle superelíptico (n=4.2)
float squircle_sdf(vec2 p, vec2 b, float r, float exp_val) {
    vec2 d = abs(p) - b + vec2(r);
    if (d.x <= 0.0 || d.y <= 0.0) {
        return max(d.x, d.y) - r;
    }
    return pow(pow(d.x, exp_val) + pow(d.y, exp_val), 1.0 / exp_val) - r;
}

void main() {
    // Coordenadas locales en píxeles dentro del quad de sombra (origen arriba-izquierda)
    vec2 px = vec2(v_uv.x * u_quad_size.x, (1.0 - v_uv.y) * u_quad_size.y);

    // Centro físico de la ventana dentro del quad
    vec2 win_center = u_pad + u_win_size * 0.5;
    vec2 win_half = u_win_size * 0.5;

    // 1. Recorte interior (Inner Cutout): la sombra sólo existe FUERA del perímetro de la ventana
    vec2 p_win = px - win_center;
    float effective_r = min(u_corner_radius, min(win_half.x, win_half.y));
    float d_win = squircle_sdf(p_win, win_half, effective_r, u_exponent);
    float inner_mask = smoothstep(-1.2, 0.2, d_win);

    if (inner_mask <= 0.001) {
        discard;
    }

    // 2. Cálculo de la sombra con desplazamiento cenital (key light) y dispersión
    vec2 shadow_center = win_center + vec2(0.0, u_offset_y);
    vec2 p_shadow = px - shadow_center;
    vec2 shadow_half = win_half + vec2(u_spread);
    float shadow_r = effective_r + u_spread;
    float d_shadow = squircle_sdf(p_shadow, shadow_half, shadow_r, u_exponent);

    float d = max(d_shadow, 0.0);

    // 3. Modelo de doble lóbulo estilo macOS:
    // - Sombra de contacto: intensa y cercana al borde
    // - Sombra ambiental: amplia y suave penumbra
    float contact_sigma = max(u_blur_radius * 0.25, 1.0);
    float contact = exp(-pow(d / contact_sigma, 2.0)) * 0.42;

    float ambient_sigma = max(u_blur_radius, 1.0);
    float ambient = exp(-pow(d / ambient_sigma, 1.65)) * 0.58;

    float shadow_intensity = (d_shadow <= 0.0) ? 1.0 : (contact + ambient);
    float final_alpha = shadow_intensity * u_opacity * inner_mask;

    if (final_alpha <= 0.002) {
        discard;
    }

    frag_color = vec4(0.0, 0.0, 0.0, final_alpha);
}
"#;

pub struct WindowShadowPipeline {
    program: ffi::types::GLuint,
    vbo: ffi::types::GLuint,
    u_quad_size: ffi::types::GLint,
    u_win_size: ffi::types::GLint,
    u_pad: ffi::types::GLint,
    u_corner_radius: ffi::types::GLint,
    u_exponent: ffi::types::GLint,
    u_blur_radius: ffi::types::GLint,
    u_spread: ffi::types::GLint,
    u_offset_y: ffi::types::GLint,
    u_opacity: ffi::types::GLint,
}

impl WindowShadowPipeline {
    pub fn new(gl: &ffi::Gles2) -> Option<Self> {
        unsafe {
            let vert = compile_shader(gl, ffi::VERTEX_SHADER, SHADOW_VERT_SRC)?;
            let frag = compile_shader(gl, ffi::FRAGMENT_SHADER, SHADOW_FRAG_SRC)?;

            let program = gl.CreateProgram();
            gl.AttachShader(program, vert);
            gl.AttachShader(program, frag);
            gl.LinkProgram(program);

            gl.DeleteShader(vert);
            gl.DeleteShader(frag);

            let mut status = 0;
            gl.GetProgramiv(program, ffi::LINK_STATUS, &mut status);
            if status == 0 {
                let mut len = 0;
                gl.GetProgramiv(program, ffi::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl.GetProgramInfoLog(program, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut ffi::types::GLchar);
                error!("❌ [WindowShadowPipeline] Error en enlace de shaders: {}", String::from_utf8_lossy(&buffer));
                gl.DeleteProgram(program);
                return None;
            }

            let u_quad_size = gl.GetUniformLocation(program, c"u_quad_size".as_ptr() as *const ffi::types::GLchar);
            let u_win_size = gl.GetUniformLocation(program, c"u_win_size".as_ptr() as *const ffi::types::GLchar);
            let u_pad = gl.GetUniformLocation(program, c"u_pad".as_ptr() as *const ffi::types::GLchar);
            let u_corner_radius = gl.GetUniformLocation(program, c"u_corner_radius".as_ptr() as *const ffi::types::GLchar);
            let u_exponent = gl.GetUniformLocation(program, c"u_exponent".as_ptr() as *const ffi::types::GLchar);
            let u_blur_radius = gl.GetUniformLocation(program, c"u_blur_radius".as_ptr() as *const ffi::types::GLchar);
            let u_spread = gl.GetUniformLocation(program, c"u_spread".as_ptr() as *const ffi::types::GLchar);
            let u_offset_y = gl.GetUniformLocation(program, c"u_offset_y".as_ptr() as *const ffi::types::GLchar);
            let u_opacity = gl.GetUniformLocation(program, c"u_opacity".as_ptr() as *const ffi::types::GLchar);

            // VBO con quad completo en Normalized Device Coordinates [-1.0, 1.0]
            let vertices: [f32; 16] = [
                -1.0, -1.0, 0.0, 0.0,
                 1.0, -1.0, 1.0, 0.0,
                -1.0,  1.0, 0.0, 1.0,
                 1.0,  1.0, 1.0, 1.0,
            ];

            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(ffi::ARRAY_BUFFER, vbo);
            gl.BufferData(
                ffi::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as ffi::types::GLsizeiptr,
                vertices.as_ptr() as *const std::ffi::c_void,
                ffi::STATIC_DRAW,
            );
            gl.BindBuffer(ffi::ARRAY_BUFFER, 0);

            info!("✨ [WindowShadowPipeline] Pipeline de sombras squircle superelípticas (n=4.2) inicializado en GPU");

            Some(Self {
                program,
                vbo,
                u_quad_size,
                u_win_size,
                u_pad,
                u_corner_radius,
                u_exponent,
                u_blur_radius,
                u_spread,
                u_offset_y,
                u_opacity,
            })
        }
    }

    /// Renderiza la sombra proyectada de una ventana
    pub fn render_shadow(
        &self,
        gl: &ffi::Gles2,
        screen_size: Size<i32, Physical>,
        win_rect: Rectangle<i32, Physical>,
        corner_radius: f32,
        is_active: bool,
        scale: f64,
    ) {
        if win_rect.size.w <= 0 || win_rect.size.h <= 0 {
            return;
        }

        // Parámetros de sombra calibrados al estilo macOS Sequoia / Sonoma
        let (blur_radius, spread, offset_y, opacity) = if is_active {
            (
                40.0 * scale as f32,
                3.5 * scale as f32,
                14.0 * scale as f32,
                0.46f32,
            )
        } else {
            (
                18.0 * scale as f32,
                1.0 * scale as f32,
                6.0 * scale as f32,
                0.24f32,
            )
        };

        // Expansión del quad de sombra para dar margen a la penumbra difusa
        let pad_x = (blur_radius * 2.2 + spread).round() as i32;
        let pad_y_top = (blur_radius * 2.2 + spread - offset_y).max(0.0).round() as i32;
        let pad_y_bottom = (blur_radius * 2.2 + spread + offset_y).round() as i32;

        let quad_x = win_rect.loc.x - pad_x;
        let quad_y = win_rect.loc.y - pad_y_top;
        let quad_w = win_rect.size.w + pad_x * 2;
        let quad_h = win_rect.size.h + pad_y_top + pad_y_bottom;

        // Coordenada Y en OpenGL (origen abajo-izquierda)
        let gl_y = screen_size.h - (quad_y + quad_h);

        unsafe {
            // Guardar estado GL previo
            let mut prev_viewport = [0i32; 4];
            gl.GetIntegerv(ffi::VIEWPORT, prev_viewport.as_mut_ptr());
            let prev_blend = gl.IsEnabled(ffi::BLEND);

            gl.Enable(ffi::BLEND);
            gl.BlendFunc(ffi::SRC_ALPHA, ffi::ONE_MINUS_SRC_ALPHA);

            gl.Viewport(quad_x, gl_y, quad_w, quad_h);

            gl.UseProgram(self.program);

            gl.Uniform2f(self.u_quad_size, quad_w as f32, quad_h as f32);
            gl.Uniform2f(self.u_win_size, win_rect.size.w as f32, win_rect.size.h as f32);
            gl.Uniform2f(self.u_pad, pad_x as f32, pad_y_top as f32);
            gl.Uniform1f(self.u_corner_radius, corner_radius);
            gl.Uniform1f(self.u_exponent, 4.2f32);
            gl.Uniform1f(self.u_blur_radius, blur_radius);
            gl.Uniform1f(self.u_spread, spread);
            gl.Uniform1f(self.u_offset_y, offset_y);
            gl.Uniform1f(self.u_opacity, opacity);

            gl.BindBuffer(ffi::ARRAY_BUFFER, self.vbo);
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0,
                2,
                ffi::FLOAT,
                ffi::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(
                1,
                2,
                ffi::FLOAT,
                ffi::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                (2 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
            );

            gl.DrawArrays(ffi::TRIANGLE_STRIP, 0, 4);

            // Restaurar estado GL previo
            gl.DisableVertexAttribArray(0);
            gl.DisableVertexAttribArray(1);
            gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
            gl.UseProgram(0);

            if prev_blend == ffi::FALSE {
                gl.Disable(ffi::BLEND);
            }
            gl.Viewport(
                prev_viewport[0],
                prev_viewport[1],
                prev_viewport[2],
                prev_viewport[3],
            );
        }
    }
}

unsafe fn compile_shader(
    gl: &ffi::Gles2,
    shader_type: ffi::types::GLenum,
    src: &str,
) -> Option<ffi::types::GLuint> {
    let shader = gl.CreateShader(shader_type);
    if shader == 0 {
        return None;
    }
    let length = src.len() as ffi::types::GLint;
    gl.ShaderSource(
        shader,
        1,
        &(src.as_ptr() as *const ffi::types::GLchar),
        &length,
    );
    gl.CompileShader(shader);

    let mut success = 0;
    gl.GetShaderiv(shader, ffi::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut len = 0;
        gl.GetShaderiv(shader, ffi::INFO_LOG_LENGTH, &mut len);
        let mut buffer: Vec<u8> = vec![0; len as usize];
        gl.GetShaderInfoLog(
            shader,
            len,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut ffi::types::GLchar,
        );
        error!(
            "❌ [WindowShadowPipeline] Error compilando shader: {}",
            String::from_utf8_lossy(&buffer)
        );
        gl.DeleteShader(shader);
        return None;
    }

    Some(shader)
}
