extern crate gl;
extern crate core;
extern crate libc;

extern crate native;
use gl::types::*;
use std::ptr;

pub static STANDARD_VERTEX: &'static str = "
        #version 330 core
        layout (location = 0) in vec2 position;
        layout (location = 1) in vec2 in_texcoord;

        out vec2 texcoord;

        void main()
        {
            gl_Position = vec4(position, 0.0f, 1.0f);
            texcoord = vec2(in_texcoord.x, 1 - in_texcoord.y);
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
    ($typ:expr $get_iv:ident | $get_log:ident $val:ident $status:ident) => (
        unsafe {
            let mut status = 0;
            gl::$get_iv($val, gl::$status, &mut status);
            if status == 0 {
                let mut len = 0;
                gl::$get_iv($val, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::from_elem(len as uint - 1, 0u8);
                gl::$get_log($val, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                
                panic!("{} ERROR: {}", $typ, String::from_utf8(buf));
            } else {
                println!("I THINK THE {} COMPILED", $typ);
            }
        }
    )
)

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
)

pub fn create_program(vert: &str, frag: &str) -> GLuint {
    let vert_id = make_shader!(vert: VERTEX_SHADER);
    check_log!("VERTEX SHADER"
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
    )

    let frag_id = make_shader!(frag: FRAGMENT_SHADER);
    check_log!("FRAGMENT SHADER"
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
    )

    let program_id = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program_id, vert_id);
        gl::AttachShader(program_id, frag_id);
        gl::LinkProgram(program_id);
    }

    check_log!("SHADER PROGRAM"
        GetProgramiv | GetProgramInfoLog
        program_id LINK_STATUS
    )

    unsafe {
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
    }

    program_id
}
