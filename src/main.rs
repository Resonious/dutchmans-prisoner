#![feature(globs, macro_rules)]
extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::Context;

use render::shader;
use render::texture;
use std::mem::{transmute, size_of, size_of_val};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;

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

macro_rules! stride(
    ($val:expr) => (($val * size_of::<GLfloat>() as i32))
)

fn test_loop(glfw: &glfw::Glfw, window: &glfw::Window, event: &GlfwEvent) {
    let vertices: [GLfloat, ..16] = [
    //    position
         1.0,  1.0,    1.0, 1.0, // Top right
         1.0, -1.0,    1.0, 0.0, // Bottom right
        -1.0, -1.0,    0.0, 0.0, // Top left
        -1.0,  1.0,    0.0, 1.0 // Bottom left
    ];
    /*  texcoords (for full image)
        1.0, 1.0,
        1.0, 0.0,
        0.0, 0.0,
        0.0, 1.0
    */
    let indices: [GLuint, ..6] = [
        0, 1, 3,
        1, 2, 3
    ];

    let positions = vec![
        Vector2::<GLfloat>::new(0.1, 0.1),
        Vector2::<GLfloat>::new(0.5, 0.5)
    ];

    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    let mut ebo: GLuint = 0;

    let mut positions_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gen_buffer!(vbo, vertices, ARRAY_BUFFER);
        gen_buffer!(ebo, indices, ELEMENT_ARRAY_BUFFER);
        gen_buffer!(positions_vbo, positions, ARRAY_BUFFER);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride!(4), as_void!(0u64));
        
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride!(4), as_void!(2 * size_of::<GLfloat>()));

        gl::BindBuffer(gl::ARRAY_BUFFER, positions_vbo);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride!(2), as_void!(0u64));
        gl::VertexAttribDivisor(2, 1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // println!("OK we got {} {} {}", vao, vbo, ebo);

    let prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
    unsafe {
        gl::UseProgram(prog);
    }
    let cam_pos_uniform = unsafe { "cam_pos".with_c_str(|c| gl::GetUniformLocation(prog, c)) };
    unsafe { gl::Uniform2f(cam_pos_uniform, 0f32, 0f32) };
    let mut cam_pos = Vector2::<GLfloat>::new(0.0, 0.0);

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

                glfw::KeyEvent(glfw::KeyUp, _, glfw::Press, _) => cam_pos.y += 0.1,
                glfw::KeyEvent(glfw::KeyDown, _, glfw::Press, _) => cam_pos.y -= 0.1,
                glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) => cam_pos.x += 0.1,
                glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) => cam_pos.x -= 0.1,

                _ => {}
            }
        }

        unsafe {
            gl::Uniform2f(cam_pos_uniform, cam_pos.x, cam_pos.y);

            gl::ClearColor(0.0, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), 2);
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
