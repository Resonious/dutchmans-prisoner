extern crate gl;
extern crate core;
extern crate libc;

use gl::types::*;
use std::ptr;

// NOTE make sure these constants match what's in the shader.
pub static ATTR_VERTEX_POS: u32 = 0;
pub static ATTR_POSITION: u32 = 1;
pub static ATTR_FRAME: u32 = 2;
pub static ATTR_FLIPPED: u32 = 3;

pub static FRAME_UNIFORM_MAX: i64 = 256;

pub static STANDARD_VERTEX: &'static str = "
        #version 330 core

        // Per vertex, normalized:
        layout (location = 0) in vec2 vertex_pos;
        // Per instance:
        layout (location = 1) in vec2 position;       // in pixels
        layout (location = 2) in int frame;
        layout (location = 3) in int flipped; // actually a bool

        uniform vec2[256] frames;
        uniform vec2 screen_size;
        uniform vec2 cam_pos;     // in pixels
        uniform vec2 sprite_size; // in pixels
        uniform float scale;

        out vec2 texcoord;

        int call_index = 0;

        vec2 from_pixel(vec2 pos)
        {
            return pos / screen_size;
        }

        vec2 brute_force_texcoord(int id)
        {
            if      (id == 0) { return vec2(1.0, 1.0); }
            else if (id == 1) { return vec2(1.0, 0.0); }
            else if (id == 2) { return vec2(0.0, 0.0); }
            else if (id == 3) { return vec2(0.0, 1.0); }
            // Should not happen:
            else { return vec2(2.2, 2.2); }
        }

        // For testing purposes:
        vec2 brute_force_half_texcoord(int id)
        {
            if      (id == 0) { return vec2(0.375, 1); }
            else if (id == 1) { return vec2(0.375, 0.5); }
            else if (id == 2) { return vec2(0.25, 0.5); }
            else if (id == 3) { return vec2(0.25, 1.0); }
            // Should not happen:
            else { return vec2(2.2, 2.2); }
        }

        int flipped_vertex_id() {
            if      (gl_VertexID == 0) return 3;
            else if (gl_VertexID == 1) return 2;
            else if (gl_VertexID == 2) return 1;
            else if (gl_VertexID == 3) return 0;
            // Should not happen:
            else return 0;
        }

        void main()
        {
            vec2 pixel_screen_pos = (position - cam_pos) * 2;
            gl_Position = vec4(
                (vertex_pos * from_pixel(sprite_size) + from_pixel(pixel_screen_pos)) * scale,
                0.0f, 1.0f
            );

            int index = flipped != 0 ? flipped_vertex_id() : gl_VertexID;

            if (frame == -1)
                texcoord = brute_force_texcoord(index);
            else
                texcoord = frames[frame * 4 + index];
            texcoord.y = 1 - texcoord.y;

            call_index += 1;
            if (call_index >= 6) call_index = 0;
        }
    ";

pub static STANDARD_FRAGMENT: &'static str = "
        #version 330 core
        in vec2 texcoord;

        out vec4 color;

        uniform sampler2D tex;

        void main()
        {
            color = texture(tex, texcoord);
        }
    ";

macro_rules! check_log(
    ($typ:expr $get_iv:ident | $get_log:ident $val:ident $status:ident $on_error:ident) => (
        unsafe {
            let mut status = 0;
            gl::$get_iv($val, gl::$status, &mut status);
            if status == 0 {
                let mut len = 0;
                gl::$get_iv($val, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::from_elem(len as uint - 1, 0u8);
                gl::$get_log($val, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                
                $on_error!("{} ERROR: {}", $typ, String::from_utf8(buf));
                false
            } else {
                println!("I THINK THE {} COMPILED", $typ);
                true
            }
        }
    )
);

macro_rules! make_shader(
    ($name:ident: $shader_type:ident) => (
        unsafe {
            let sh = gl::CreateShader(gl::$shader_type);
            $name.with_c_str(|src|
                gl::ShaderSource(sh, 1, &src, ptr::null())
            );
            gl::CompileShader(sh);
            sh
        }
    )
);

pub fn create_program(vert: &str, frag: &str) -> Option<GLuint> {
    let vert_id = make_shader!(vert: VERTEX_SHADER);
    let vert_result = check_log!("VERTEX SHADER"
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
        println
    );
    if !vert_result {
        unsafe { gl::DeleteShader(vert_id); }
        return None;
    }

    let frag_id = make_shader!(frag: FRAGMENT_SHADER);
    let frag_result = check_log!("FRAGMENT SHADER"
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
        println
    );
    if !frag_result {
        unsafe { gl::DeleteShader(vert_id); }
        unsafe { gl::DeleteShader(frag_id); }
        return None;
    }

    let program_id = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program_id, vert_id);
        gl::AttachShader(program_id, frag_id);
        gl::LinkProgram(program_id);
    }

    let link_result = check_log!("SHADER PROGRAM"
        GetProgramiv | GetProgramInfoLog
        program_id LINK_STATUS
        println
    );
    if !link_result {
        unsafe { gl::DeleteProgram(program_id); }
        unsafe { gl::DeleteShader(vert_id); }
        unsafe { gl::DeleteShader(frag_id); }
        return None;
    }

    unsafe {
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
    }

    Some(program_id)
}
