//! Pipeline de desenfoque de fondo acelerado por GPU (Dual Kawase Background Blur)
//!
//! Implementa el algoritmo Dual Kawase para desenfoque en tiempo real de alto rendimiento
//! con soporte para recorte squircle continuo (superelipse n=4.2).

use smithay::{
    backend::renderer::gles::ffi,
    utils::{Physical, Rectangle, Size},
};
use tracing::{error, info};

const VERTEX_SHADER_SRC: &str = r#"#version 300 es
precision highp float;

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = a_uv;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

const DUAL_KAWASE_DOWN_SRC: &str = r#"#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;

void main() {
    vec4 sum = texture(u_texture, v_tex_coords) * 4.0;
    sum += texture(u_texture, v_tex_coords - u_halfpixel.xy * u_offset);
    sum += texture(u_texture, v_tex_coords + u_halfpixel.xy * u_offset);
    sum += texture(u_texture, v_tex_coords + vec2(u_halfpixel.x, -u_halfpixel.y) * u_offset);
    sum += texture(u_texture, v_tex_coords - vec2(u_halfpixel.x, -u_halfpixel.y) * u_offset);
    frag_color = sum / 8.0;
}
"#;

const DUAL_KAWASE_UP_SRC: &str = r#"#version 300 es
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
"#;

const BLIT_SQUIRCLE_SRC: &str = r#"#version 300 es
precision highp float;

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_texture;
uniform vec2 u_size;
uniform float u_corner_radius;

float squircle_sdf(vec2 p, vec2 b, float r) {
    vec2 d = abs(p) - b + vec2(r);
    if (d.x <= 0.0 || d.y <= 0.0) {
        return max(d.x, d.y) - r;
    }
    return pow(pow(d.x, 4.2) + pow(d.y, 4.2), 1.0 / 4.2) - r;
}

void main() {
    vec4 col = texture(u_texture, v_tex_coords);
    if (u_corner_radius > 1.0) {
        vec2 half_size = u_size * 0.5;
        vec2 p = (v_tex_coords - 0.5) * u_size;
        float dist = squircle_sdf(p, half_size, u_corner_radius);
        float edge_aa = max(fwidth(dist), 1.0);
        float alpha = 1.0 - smoothstep(-edge_aa, 0.0, dist);
        frag_color = vec4(col.rgb, alpha);
    } else {
        frag_color = vec4(col.rgb, 1.0);
    }
}
"#;

pub struct DualKawaseBlur {
    vbo: ffi::types::GLuint,
    down_program: ffi::types::GLuint,
    up_program: ffi::types::GLuint,
    blit_program: ffi::types::GLuint,

    down_u_texture: ffi::types::GLint,
    down_u_halfpixel: ffi::types::GLint,
    down_u_offset: ffi::types::GLint,

    up_u_texture: ffi::types::GLint,
    up_u_halfpixel: ffi::types::GLint,
    up_u_offset: ffi::types::GLint,

    blit_u_texture: ffi::types::GLint,
    blit_u_size: ffi::types::GLint,
    blit_u_corner_radius: ffi::types::GLint,

    source_tex: ffi::types::GLuint,
    source_tex_size: (i32, i32),

    fbo_textures: [ffi::types::GLuint; 2],
    fbos: [ffi::types::GLuint; 2],
    fbo_sizes: [(i32, i32); 2],
}

impl DualKawaseBlur {
    pub fn new(gl: &ffi::Gles2) -> Option<Self> {
        unsafe {
            let vert_shader = compile_shader(gl, ffi::VERTEX_SHADER, VERTEX_SHADER_SRC)?;
            let down_frag = compile_shader(gl, ffi::FRAGMENT_SHADER, DUAL_KAWASE_DOWN_SRC)?;
            let up_frag = compile_shader(gl, ffi::FRAGMENT_SHADER, DUAL_KAWASE_UP_SRC)?;
            let blit_frag = compile_shader(gl, ffi::FRAGMENT_SHADER, BLIT_SQUIRCLE_SRC)?;

            let down_program = link_program(gl, vert_shader, down_frag)?;
            let up_program = link_program(gl, vert_shader, up_frag)?;
            let blit_program = link_program(gl, vert_shader, blit_frag)?;

            gl.DeleteShader(vert_shader);
            gl.DeleteShader(down_frag);
            gl.DeleteShader(up_frag);
            gl.DeleteShader(blit_frag);

            let down_u_texture = gl.GetUniformLocation(down_program, c"u_texture".as_ptr());
            let down_u_halfpixel = gl.GetUniformLocation(down_program, c"u_halfpixel".as_ptr());
            let down_u_offset = gl.GetUniformLocation(down_program, c"u_offset".as_ptr());

            let up_u_texture = gl.GetUniformLocation(up_program, c"u_texture".as_ptr());
            let up_u_halfpixel = gl.GetUniformLocation(up_program, c"u_halfpixel".as_ptr());
            let up_u_offset = gl.GetUniformLocation(up_program, c"u_offset".as_ptr());

            let blit_u_texture = gl.GetUniformLocation(blit_program, c"u_texture".as_ptr());
            let blit_u_size = gl.GetUniformLocation(blit_program, c"u_size".as_ptr());
            let blit_u_corner_radius = gl.GetUniformLocation(blit_program, c"u_corner_radius".as_ptr());

            let quad_vertices: [f32; 16] = [
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
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const std::ffi::c_void,
                ffi::STATIC_DRAW,
            );
            gl.BindBuffer(ffi::ARRAY_BUFFER, 0);

            let mut source_tex = 0;
            gl.GenTextures(1, &mut source_tex);

            let mut fbos = [0; 2];
            let mut fbo_textures = [0; 2];
            gl.GenFramebuffers(2, fbos.as_mut_ptr());
            gl.GenTextures(2, fbo_textures.as_mut_ptr());

            info!("✨ [DualKawaseBlur] Pipeline de desenfoque por GPU inicializado con éxito.");

            Some(Self {
                vbo,
                down_program,
                up_program,
                blit_program,
                down_u_texture,
                down_u_halfpixel,
                down_u_offset,
                up_u_texture,
                up_u_halfpixel,
                up_u_offset,
                blit_u_texture,
                blit_u_size,
                blit_u_corner_radius,
                source_tex,
                source_tex_size: (0, 0),
                fbo_textures,
                fbos,
                fbo_sizes: [(0, 0), (0, 0)],
            })
        }
    }

    /// Aplica desenfoque Dual Kawase a la región de pantalla dada
    pub fn render_blur(
        &mut self,
        gl: &ffi::Gles2,
        screen_size: Size<i32, Physical>,
        rect: Rectangle<i32, Physical>,
        corner_radius: f32,
    ) {
        let w = rect.size.w;
        let h = rect.size.h;

        if w < 4 || h < 4 {
            return;
        }

        let gl_y = screen_size.h - (rect.loc.y + h);

        unsafe {
            // 1. Guardar estado GL previo
            let mut prev_fbo = 0;
            let mut prev_viewport = [0; 4];
            let prev_blend = gl.IsEnabled(ffi::BLEND);

            gl.GetIntegerv(ffi::FRAMEBUFFER_BINDING, &mut prev_fbo);
            gl.GetIntegerv(ffi::VIEWPORT, prev_viewport.as_mut_ptr());

            // 2. Asegurar tamaño de textura origen
            if w > self.source_tex_size.0 || h > self.source_tex_size.1 {
                let alloc_w = w.max(self.source_tex_size.0).max(64);
                let alloc_h = h.max(self.source_tex_size.1).max(64);
                gl.BindTexture(ffi::TEXTURE_2D, self.source_tex);
                gl.TexImage2D(
                    ffi::TEXTURE_2D,
                    0,
                    ffi::RGBA as i32,
                    alloc_w,
                    alloc_h,
                    0,
                    ffi::RGBA,
                    ffi::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MIN_FILTER, ffi::LINEAR as i32);
                gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MAG_FILTER, ffi::LINEAR as i32);
                gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_S, ffi::CLAMP_TO_EDGE as i32);
                gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_T, ffi::CLAMP_TO_EDGE as i32);
                self.source_tex_size = (alloc_w, alloc_h);
            }

            // 3. Copiar región del framebuffer de pantalla actual a source_tex
            gl.BindTexture(ffi::TEXTURE_2D, self.source_tex);
            gl.CopyTexSubImage2D(ffi::TEXTURE_2D, 0, 0, 0, rect.loc.x, gl_y, w, h);

            // 4. Asegurar tamaños de texturas FBO intermedias
            let w0 = (w / 2).max(2);
            let h0 = (h / 2).max(2);
            let w1 = (w / 4).max(2);
            let h1 = (h / 4).max(2);

            let targets = [(w0, h0), (w1, h1)];
            for i in 0..2 {
                if targets[i].0 > self.fbo_sizes[i].0 || targets[i].1 > self.fbo_sizes[i].1 {
                    let alloc_w = targets[i].0.max(self.fbo_sizes[i].0).max(32);
                    let alloc_h = targets[i].1.max(self.fbo_sizes[i].1).max(32);
                    gl.BindTexture(ffi::TEXTURE_2D, self.fbo_textures[i]);
                    gl.TexImage2D(
                        ffi::TEXTURE_2D,
                        0,
                        ffi::RGBA as i32,
                        alloc_w,
                        alloc_h,
                        0,
                        ffi::RGBA,
                        ffi::UNSIGNED_BYTE,
                        std::ptr::null(),
                    );
                    gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MIN_FILTER, ffi::LINEAR as i32);
                    gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MAG_FILTER, ffi::LINEAR as i32);
                    gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_S, ffi::CLAMP_TO_EDGE as i32);
                    gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_T, ffi::CLAMP_TO_EDGE as i32);

                    gl.BindFramebuffer(ffi::FRAMEBUFFER, self.fbos[i]);
                    gl.FramebufferTexture2D(
                        ffi::FRAMEBUFFER,
                        ffi::COLOR_ATTACHMENT0,
                        ffi::TEXTURE_2D,
                        self.fbo_textures[i],
                        0,
                    );
                    self.fbo_sizes[i] = (alloc_w, alloc_h);
                }
            }

            gl.Disable(ffi::BLEND);
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

            // PASO 1: Downsample inicial desde source_tex -> FBO 0
            gl.BindFramebuffer(ffi::FRAMEBUFFER, self.fbos[0]);
            gl.Viewport(0, 0, w0, h0);
            gl.UseProgram(self.down_program);
            gl.ActiveTexture(ffi::TEXTURE0);
            gl.BindTexture(ffi::TEXTURE_2D, self.source_tex);
            gl.Uniform1i(self.down_u_texture, 0);
            gl.Uniform2f(self.down_u_halfpixel, 0.5 / w as f32, 0.5 / h as f32);
            gl.Uniform1f(self.down_u_offset, 1.5);
            gl.DrawArrays(ffi::TRIANGLE_STRIP, 0, 4);

            // PASO 2: Downsample adicional desde FBO 0 -> FBO 1
            gl.BindFramebuffer(ffi::FRAMEBUFFER, self.fbos[1]);
            gl.Viewport(0, 0, w1, h1);
            gl.BindTexture(ffi::TEXTURE_2D, self.fbo_textures[0]);
            gl.Uniform2f(self.down_u_halfpixel, 0.5 / w0 as f32, 0.5 / h0 as f32);
            gl.Uniform1f(self.down_u_offset, 2.5);
            gl.DrawArrays(ffi::TRIANGLE_STRIP, 0, 4);

            // PASO 3: Upsample desde FBO 1 -> FBO 0
            gl.BindFramebuffer(ffi::FRAMEBUFFER, self.fbos[0]);
            gl.Viewport(0, 0, w0, h0);
            gl.UseProgram(self.up_program);
            gl.BindTexture(ffi::TEXTURE_2D, self.fbo_textures[1]);
            gl.Uniform1i(self.up_u_texture, 0);
            gl.Uniform2f(self.up_u_halfpixel, 0.5 / w1 as f32, 0.5 / h1 as f32);
            gl.Uniform1f(self.up_u_offset, 2.5);
            gl.DrawArrays(ffi::TRIANGLE_STRIP, 0, 4);

            // PASO 4: Blit final desde FBO 0 hacia la pantalla con soporte squircle
            gl.BindFramebuffer(ffi::FRAMEBUFFER, prev_fbo as u32);
            gl.Viewport(rect.loc.x, gl_y, w, h);
            gl.Enable(ffi::BLEND);
            gl.BlendFunc(ffi::SRC_ALPHA, ffi::ONE_MINUS_SRC_ALPHA);

            gl.UseProgram(self.blit_program);
            gl.BindTexture(ffi::TEXTURE_2D, self.fbo_textures[0]);
            gl.Uniform1i(self.blit_u_texture, 0);
            gl.Uniform2f(self.blit_u_size, w as f32, h as f32);
            gl.Uniform1f(self.blit_u_corner_radius, corner_radius);
            gl.DrawArrays(ffi::TRIANGLE_STRIP, 0, 4);

            // 5. Restaurar estado GL previo
            gl.DisableVertexAttribArray(0);
            gl.DisableVertexAttribArray(1);
            gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
            gl.BindTexture(ffi::TEXTURE_2D, 0);
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

impl Drop for DualKawaseBlur {
    fn drop(&mut self) {
        // En Rust/Smithay la limpieza de recursos GL debe hacerse con el contexto EGL activo
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
            "❌ [DualKawaseBlur] Error al compilar shader GLSL: {}",
            String::from_utf8_lossy(&buffer)
        );
        gl.DeleteShader(shader);
        return None;
    }
    Some(shader)
}

unsafe fn link_program(
    gl: &ffi::Gles2,
    vert: ffi::types::GLuint,
    frag: ffi::types::GLuint,
) -> Option<ffi::types::GLuint> {
    let program = gl.CreateProgram();
    gl.AttachShader(program, vert);
    gl.AttachShader(program, frag);
    gl.LinkProgram(program);

    let mut success = 0;
    gl.GetProgramiv(program, ffi::LINK_STATUS, &mut success);
    if success == 0 {
        let mut len = 0;
        gl.GetProgramiv(program, ffi::INFO_LOG_LENGTH, &mut len);
        let mut buffer: Vec<u8> = vec![0; len as usize];
        gl.GetProgramInfoLog(
            program,
            len,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut ffi::types::GLchar,
        );
        error!(
            "❌ [DualKawaseBlur] Error al enlazar programa shader GLSL: {}",
            String::from_utf8_lossy(&buffer)
        );
        gl.DeleteProgram(program);
        return None;
    }
    Some(program)
}
