#![feature(globs, macro_rules)]
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::{Context, Action, Key};

use render::shader;
use render::texture;
use render::texture::TextureManager;
use render::texture::Texcoords;
use render::sprite::*;
use std::mem::{transmute, size_of, size_of_val};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;

pub mod render;
pub mod asset;

pub type GlfwEvent = Receiver<(f64, glfw::WindowEvent)>;

macro_rules! gen_buffer(
    ($obj:ident, $buf:ident, $typ:ident, $draw:ident) => (
        {
            gl::GenBuffers(1, &mut $obj);
            gl::BindBuffer(gl::$typ, $obj);
            gl::BufferData(gl::$typ,
                size_of_val(&$buf) as GLsizeiptr,
                transmute(&$buf[0]),
                gl::$draw
            );
        }
    )
);

macro_rules! check_error(
    () => (
        match gl::GetError() {
            gl::NO_ERROR => {}
            gl::INVALID_ENUM => panic!("Invalid enum!"),
            gl::INVALID_VALUE => panic!("Invalid value!"),
            gl::INVALID_OPERATION => panic!("Invalid operation!"),
            gl::INVALID_FRAMEBUFFER_OPERATION => panic!("Invalid framebuffer operation?!"),
            gl::OUT_OF_MEMORY => panic!("Out of memory bro!!!!!!!"),
            // gl::STACK_UNDERFLOW => panic!("Stack UNDERflow!"),
            // gl::STACK_OVERFLOW => panic!("Stack overflow!")
            _ => panic!("I DON'T KNOW. FULL BANANNACAKES.")
        }
    )
);

macro_rules! as_void(
    ($val:expr) => (transmute::<i64, *const c_void>($val))
);

macro_rules! stride(
    ($val:expr) => (($val * size_of::<GLfloat>() as i32))
);


// TODO BLAHHHHHHHH
fn set_sprite_attribute(vbo: GLuint) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        let size_of_sprite = size_of::<SpriteData>() as GLint;
        assert_eq!(size_of_sprite, 12);

        // == Position ==
        gl::EnableVertexAttribArray(shader::ATTR_POSITION);
        gl::VertexAttribPointer(
            shader::ATTR_POSITION, 2, gl::FLOAT, gl::FALSE as GLboolean,
            size_of_sprite, as_void!(0)
        );
        gl::VertexAttribDivisor(shader::ATTR_POSITION, 1);
        let offset = 2 * size_of::<GLfloat>() as i64;
        assert_eq!(offset, 8);

        // == Frame ==
        gl::EnableVertexAttribArray(shader::ATTR_FRAME);
        gl::VertexAttribIPointer(
            shader::ATTR_FRAME, 1, gl::INT,
            size_of_sprite, as_void!(offset)
        );
        gl::VertexAttribDivisor(shader::ATTR_FRAME, 1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}


extern "C" {
    pub fn glfwGetCurrentContext() -> u64;
    pub fn glfwMakeContextCurrent(window: *mut u8);
    pub fn glfwCopyDataFrom(data: *const u8);
    pub fn glfwTestIdent() -> int;
}

#[no_mangle]
pub extern "C" fn copy_glfw(data: *const u8) {
    unsafe { glfwCopyDataFrom(data); }
}

#[no_mangle]
pub extern "C" fn old_test_loop(glfw: &glfw::Glfw, window: &glfw::Window, event: &GlfwEvent) {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    window.make_current();
    gl::load_with(|s| window.get_proc_address(s));

    let vertices: [GLfloat, ..8] = [
    //    position
         2.0,  2.0, //   1.0, 1.0, // Top right
         2.0,  0.0, //   1.0, 0.0, // Bottom right
         0.0,  0.0, //   0.0, 0.0, // Bottom left
         0.0,  2.0, //   0.0, 1.0  // Top left
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

    let zero_zero_positions = [
        SpriteData {
            position: Vector2::new(0.0, 0.0),
            frame: -1
        }
    ];

    // let blob_positions = vec![
    //     Vector2::<GLfloat>::new(600.0, -200.0),
    //     Vector2::<GLfloat>::new(300.0, 100.0)
    // ];

    let crattle_positions = [
        SpriteData {
            position: Vector2::new(100.0, 100.0),
            frame: 2
        },
        SpriteData {
            position: Vector2::new(-200.0, -200.0),
            frame: 0
        }
    ];

    let mut texture_manager = TextureManager::new();

    // TODO RIIIIGHT HERE. TextureManager#load is definitely the perpetrator.
    let     zero_zero_tex = unsafe { &*texture_manager.load("zero-zero.png") };
    let mut crattle_tex   = unsafe { &mut*texture_manager.load("tile-test.png") };
    crattle_tex.add_frames(10, 64, 64);

    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    let mut ebo: GLuint = 0;

    let mut zero_zero_positions_vbo: GLuint = 0;
    let mut crattle_positions_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gen_buffer!(vbo, vertices, ARRAY_BUFFER, STATIC_DRAW);
        gen_buffer!(ebo, indices, ELEMENT_ARRAY_BUFFER, STATIC_DRAW);
        gen_buffer!(zero_zero_positions_vbo, zero_zero_positions, ARRAY_BUFFER, DYNAMIC_DRAW);
        gen_buffer!(crattle_positions_vbo, crattle_positions, ARRAY_BUFFER, DYNAMIC_DRAW);

        // per-vertex stuff
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(shader::ATTR_VERTEX_POS);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride!(2), as_void!(0));
        
        // gl::EnableVertexAttribArray(1);
        // gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE as GLboolean,
        //                         stride!(4), as_void!(2 * size_of::<GLfloat>() as i64));

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // println!("OK we got {} {} {}", vao, vbo, ebo);

    let prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
    unsafe { gl::UseProgram(prog) }
    let cam_pos_uniform     = unsafe {     "cam_pos".with_c_str(|c| gl::GetUniformLocation(prog, c)) };
    let sprite_size_uniform = unsafe { "sprite_size".with_c_str(|s| gl::GetUniformLocation(prog, s)) };
    let screen_size_uniform = unsafe { "screen_size".with_c_str(|s| gl::GetUniformLocation(prog, s)) };
    let tex_uniform         = unsafe {         "tex".with_c_str(|t| gl::GetUniformLocation(prog, t)) };
    let frames_uniform      = unsafe {      "frames".with_c_str(|f| gl::GetUniformLocation(prog, f)) };
    unsafe {
        gl::Uniform2f(cam_pos_uniform, 0f32, 0f32);
        match window.get_size() {
            (width, height) => gl::Uniform2f(screen_size_uniform, width as f32, height as f32)
        }
    }
    crattle_tex.generate_texcoords_buffer();

    let mut cam_pos = Vector2::<GLfloat>::new(0.0, 0.0);

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(event) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }

                glfw::WindowEvent::Key(Key::Up, _, press, _) => 
                    if press != Action::Release {
                        cam_pos.y += 10.0;
                        println!("Cam is at {}", cam_pos);
                    },
                glfw::WindowEvent::Key(Key::Down, _, press, _) => 
                    if press != Action::Release {
                        cam_pos.y -= 10.0;
                        println!("Cam is at {}", cam_pos);
                    },
                glfw::WindowEvent::Key(Key::Right, _, press, _) => 
                    if press != Action::Release {
                        cam_pos.x += 10.0;
                        println!("Cam is at {}", cam_pos);
                    },
                glfw::WindowEvent::Key(Key::Left, _, press, _) => 
                    if press != Action::Release {
                        cam_pos.x -= 10.0;
                        println!("Cam is at {}", cam_pos);
                    },

                glfw::WindowEvent::Key(Key::B, _, Action::Release, _) => {
                    let frames = &crattle_tex.frames;
                    println!("Break time!");
                },

                glfw::WindowEvent::Size(width, height) => unsafe {
                    println!("screen is now {} x {}", width, height);
                    gl::Viewport(0, 0, width, height);
                    gl::Uniform2f(screen_size_uniform, width as f32, height as f32);
                },

                _ => {}
            }
        }

        unsafe {
            gl::Uniform2f(cam_pos_uniform, cam_pos.x, cam_pos.y);

            gl::ClearColor(0.0, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw zero-zero sign
            let zero_zero_len = zero_zero_positions.len();
            zero_zero_tex.set_full(tex_uniform, sprite_size_uniform);
            set_sprite_attribute(zero_zero_positions_vbo);
            gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), zero_zero_len as i32);
            // Draw CRATTLE!!!
            crattle_tex.set(tex_uniform, sprite_size_uniform, frames_uniform, 64.0, 64.0);
            set_sprite_attribute(crattle_positions_vbo);
            gl::DrawElementsInstanced(
                gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), crattle_positions.len() as i32
            );

            check_error!();
        }

        window.swap_buffers();
    }
}

#[test]
fn it_works() {
    assert!(false);
}
