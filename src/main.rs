#![feature(globs, macro_rules)]
extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;

use glfw::Context;

use render::shader;
use render::texture;
use std::mem::{transmute, size_of, size_of_val};
use gl::types::*;
use libc::c_void;
use std::ptr;

pub mod render;
pub mod asset;

type GlfwEvent = Receiver<(f64, glfw::WindowEvent)>;

macro_rules! gen_buffer(
    ($obj:ident, $buf:ident, $typ:ident) => (
        {
            gl::GenBuffers(1, &mut $obj);
            gl::BindBuffer(gl::$typ, $obj);
            gl::BufferData(gl::$typ,
                size_of_val(&$buf) as GLsizeiptr,
                transmute(&$buf[0]),
                gl::STATIC_DRAW
            );
        }
    )
)

macro_rules! as_void(
    ($val:expr) => (transmute::<_, *const c_void>($val))
)

fn test_loop(glfw: &glfw::Glfw, window: &glfw::Window, event: &GlfwEvent) {
    let vertices: [GLfloat, ..16] = [
    //    position       texcoords
         0.5,  0.5,      1.0, 1.0,
         0.5, -0.5,      1.0, 0.0,
        -0.5, -0.5,      0.0, 0.0,
        -0.5,  0.5,      0.0, 1.0
    ];

    let indices: [GLuint, ..6] = [
        0, 1, 3,
        1, 2, 3
    ];

    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    let mut ebo: GLuint = 0;
    
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gen_buffer!(vbo, vertices, ARRAY_BUFFER);
        gen_buffer!(ebo, indices, ELEMENT_ARRAY_BUFFER);

        let stride = 4 * size_of::<GLfloat>() as i32;

        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride, as_void!(0u64));
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride, as_void!(2 * size_of::<GLfloat>()));
        gl::EnableVertexAttribArray(1);
    }

    // println!("OK we got {} {} {}", vao, vbo, ebo);

    let prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
    unsafe { gl::UseProgram(prog); }

    let test_tex = texture::load_texture("testtex.png");
    
    unsafe {
        // It may already be bound, but just to be clear...
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, test_tex);
        let sampler_location = "tex".with_c_str(|t| gl::GetUniformLocation(prog, t));
        gl::Uniform1i(sampler_location, 0);
    }

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(event) {
            match event {
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => {
                    window.set_should_close(true)
                }

                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.0, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        window.swap_buffers();
    }
}

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (window, event) = glfw
        .create_window(480, 480, "Crattle Crute maybe?", glfw::Windowed)
        .expect("Failed to create window!");

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    let path = asset::path("basicshading.png");
    println!("here it is: {}", path.display());

    test_loop(&glfw, &window, &event);
}
