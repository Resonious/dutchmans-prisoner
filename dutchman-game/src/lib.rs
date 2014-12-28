#![feature(globs, macro_rules)]
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::{Context, Action, Key};

use render::shader;
use render::texture;
use render::texture::{Texture, Texcoords, TextureManager};
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
    ($obj:expr, $buf:expr, $typ:ident, $draw:ident) => (
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

extern "C" {
    pub fn glfwGetCurrentContext() -> u64;
    pub fn glfwMakeContextCurrent(window: *mut u8);
    // NOTE This is a custom function.
    pub fn glfwCopyDataFrom(data: *const u8);
    pub fn glfwTestIdent() -> int;
    pub fn glfwInit() -> bool;
}

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

static SQUARE_VERTICES: [GLfloat, ..8] = [
//    position
     2.0,  2.0, //   1.0, 1.0, // Top right
     2.0,  0.0, //   1.0, 0.0, // Bottom right
     0.0,  0.0, //   0.0, 0.0, // Bottom left
     0.0,  2.0, //   0.0, 1.0  // Top left
];
static SQUARE_INDICES: [GLuint, ..6] = [
    0, 1, 3,
    1, 2, 3
];
/*  texcoords (for full image)
    1.0, 1.0,
    1.0, 0.0,
    0.0, 0.0,
    0.0, 1.0
*/

pub struct Game {
    pub initialized: bool,

    pub vao: GLuint,
    pub square_vbo: GLuint,
    pub square_ebo: GLuint,

    pub shader_prog:         GLuint,
    pub cam_pos_uniform:     GLint,
    pub sprite_size_uniform: GLint,
    pub screen_size_uniform: GLint,
    pub tex_uniform:         GLint,
    pub frames_uniform:      GLint,

    pub texture_manager: TextureManager,

    pub zero_zero_tex: *mut Texture,
    pub zero_zero_vbo: GLuint,
    pub zero_zero_positions: [SpriteData, ..1],

    pub crattle_tex: *mut Texture,
    pub crattle_vbo: GLuint,
    pub crattle_positions: [SpriteData, ..2],

    pub cam_pos: Vector2<GLfloat>
}

#[no_mangle]
pub extern "C" fn load(glfw_data: *const u8, window: &glfw::Window, game: &mut Game) {
    // TODO Kinda hacky. Maybe refactor so that all GL calls happen within
    // the "platform" layer and we don't need to mess with glfw here.
    unsafe {
        println!("Loading up!");
        glfwInit();
        glfwCopyDataFrom(glfw_data); 
        window.make_current();
        gl::load_with(|s| window.get_proc_address(s));
    }

    if !game.initialized {
        game.initialized = true;

        // === Generate textures and the like ===
        game.texture_manager = TextureManager::new();

        game.zero_zero_tex = game.texture_manager.load("zero-zero.png");
        game.zero_zero_positions = [
            SpriteData {
                position: Vector2::new(0.0, 0.0),
                frame: -1
            }
        ];

        game.crattle_tex = game.texture_manager.load("tile-test.png");
        let mut crattle_tex = unsafe { &mut *game.crattle_tex };
        crattle_tex.add_frames(10, 64, 64);
        game.crattle_positions = [
            SpriteData {
                position: Vector2::new(100.0, 100.0),
                frame: 2
            },
            SpriteData {
                position: Vector2::new(-200.0, -200.0),
                frame: 0
            }
        ];
 
        // === Generate global VAO ===
        unsafe {
            gl::GenVertexArrays(1, &mut game.vao);
            gl::BindVertexArray(game.vao);

            // === Generate and populate global rectangle buffers ===
            gen_buffer!(game.square_vbo, SQUARE_VERTICES, ARRAY_BUFFER, STATIC_DRAW);
            gen_buffer!(game.square_ebo, SQUARE_INDICES, ELEMENT_ARRAY_BUFFER, STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, game.square_vbo);
            gl::EnableVertexAttribArray(shader::ATTR_VERTEX_POS);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                    stride!(2), as_void!(0));

            // === Generate (by hand) stuff on the screen ===
            gen_buffer!(game.zero_zero_vbo, game.zero_zero_positions, ARRAY_BUFFER, DYNAMIC_DRAW);
            gen_buffer!(game.crattle_vbo, game.crattle_positions, ARRAY_BUFFER, DYNAMIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // === Generate shaders ===
        game.shader_prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
        unsafe { gl::UseProgram(game.shader_prog) }
        game.cam_pos_uniform     = unsafe {     "cam_pos".with_c_str(|c| gl::GetUniformLocation(game.shader_prog, c)) };
        game.sprite_size_uniform = unsafe { "sprite_size".with_c_str(|s| gl::GetUniformLocation(game.shader_prog, s)) };
        game.screen_size_uniform = unsafe { "screen_size".with_c_str(|s| gl::GetUniformLocation(game.shader_prog, s)) };
        game.tex_uniform         = unsafe {         "tex".with_c_str(|t| gl::GetUniformLocation(game.shader_prog, t)) };
        game.frames_uniform      = unsafe {      "frames".with_c_str(|f| gl::GetUniformLocation(game.shader_prog, f)) };
        unsafe {
            gl::Uniform2f(game.cam_pos_uniform, 0f32, 0f32);
            match window.get_size() {
                (width, height) => gl::Uniform2f(game.screen_size_uniform, width as f32, height as f32)
            }
        }
        // NOTE this, among other things, uses a std::Vec, which obviously won't persist.
        crattle_tex.generate_texcoords_buffer();

        game.cam_pos = Vector2::new(0.0, 0.0);
    }
}

#[no_mangle]
pub extern "C" fn update_and_render(game: &mut Game, glfw: &glfw::Glfw, window: &glfw::Window, event: &GlfwEvent) {
    glfw.poll_events();

    let mut crattle_tex = unsafe { &mut *game.crattle_tex };
    let zero_zero_tex   = unsafe { &*game.zero_zero_tex };

    for (_, event) in glfw::flush_messages(event) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            glfw::WindowEvent::Key(Key::Up, _, press, _) => 
                if press != Action::Release {
                    game.cam_pos.y += 10.0;
                    // println!("Cam is at {}", cam_pos);
                },
            glfw::WindowEvent::Key(Key::Down, _, press, _) => 
                if press != Action::Release {
                    game.cam_pos.y -= 10.0;
                    // println!("Cam is at {}", cam_pos);
                },
            glfw::WindowEvent::Key(Key::Right, _, press, _) => 
                if press != Action::Release {
                    game.cam_pos.x += 10.0;
                    // println!("Cam is at {}", cam_pos);
                },
            glfw::WindowEvent::Key(Key::Left, _, press, _) => 
                if press != Action::Release {
                    game.cam_pos.x -= 10.0;
                    // println!("Cam is at {}", cam_pos);
                },

            glfw::WindowEvent::Key(Key::B, _, Action::Release, _) => {
                let frames = &crattle_tex.frames;
                println!("Break time!");
            },

            glfw::WindowEvent::Size(width, height) => unsafe {
                println!("screen is now {} x {}", width, height);
                gl::Viewport(0, 0, width, height);
                gl::Uniform2f(game.screen_size_uniform, width as f32, height as f32);
            },

            _ => {}
        }
    }

    unsafe {
        gl::Uniform2f(game.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);

        gl::ClearColor(0.0, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw zero-zero sign
        let zero_zero_len = game.zero_zero_positions.len();
        zero_zero_tex.set_full(game.tex_uniform, game.sprite_size_uniform);
        set_sprite_attribute(game.zero_zero_vbo);
        gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), zero_zero_len as i32);
        // Draw CRATTLE!!!
        crattle_tex.set(game.tex_uniform, game.sprite_size_uniform, game.frames_uniform, 64.0, 64.0);
        set_sprite_attribute(game.crattle_vbo);
        gl::DrawElementsInstanced(
            gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), game.crattle_positions.len() as i32
        );

        check_error!();
    }

    window.swap_buffers();
}

#[test]
fn it_works() {
    assert!(false);
}
