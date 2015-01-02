#![feature(globs, macro_rules)]
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::{Context, Action, Key};

use render::shader;
use render::texture;
use render::texture::{Texture, Texcoords, TextureManager, Frame};
use render::sprite::*;
use controls::{Controls, Control};
use std::mem::{transmute, size_of, size_of_val, zeroed};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;
use std::time::duration::Duration;

pub mod render;
pub mod asset;
pub mod controls;

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

pub struct Options {
    pub controls: Controls
}

pub struct Game {
    pub initialized: bool,

    // NOTE Do whatever you want with this.
    pub debug_flag: int,

    pub vao: GLuint,
    pub square_vbo: GLuint,
    pub square_ebo: GLuint,

    pub shader_prog:         GLuint,
    pub cam_pos_uniform:     GLint,
    pub scale_uniform:       GLint,
    pub sprite_size_uniform: GLint,
    pub screen_size_uniform: GLint,
    pub tex_uniform:         GLint,
    pub frames_uniform:      GLint,

    pub texture_manager: TextureManager,

    pub zero_zero_tex: *mut Texture,
    pub zero_zero_vbo: GLuint,
    pub zero_zero_positions: [SpriteData, ..1],

    pub tile_tex: *mut Texture,
    pub tile_frame_space: [Frame, ..14], // <- number of frames.
    pub tile_texcoords_space: [Texcoords, ..14], // Perhaps we can remove frames.
    pub tile_vbo: GLuint,
    pub tile_positions: [SpriteData, ..10*10],

    pub player_tex: *mut Texture,
    pub player_frame_space: [Frame, ..3],
    pub player_texcoords_space: [Texcoords, ..3],
    pub player_vbo: GLuint,
    pub player_state: SpriteData,

    pub cam_pos: Vector2<GLfloat>,
}

#[no_mangle]
pub extern "C" fn load(glfw_data: *const u8,
                       window:    &glfw::Window,
                       game:      &mut Game,
                       options:   &mut Options)
{
    // TODO Put OpenGL state into another separate struct.
    unsafe {
        println!("Loading up!");
        glfwInit();
        glfwCopyDataFrom(glfw_data); 
        window.make_current();
        gl::load_with(|s| window.get_proc_address(s));
    }

    if !game.initialized {
        game.initialized = true;

        options.controls = unsafe { zeroed() };

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        game.debug_flag = 0;

        // === Generate textures and the like ===
        game.texture_manager = TextureManager::new();

        game.zero_zero_tex = game.texture_manager.load("zero-zero.png");
        game.zero_zero_positions = [
            SpriteData {
                position: Vector2::new(0.0, 0.0),
                frame: -1
            }
        ];

        game.tile_tex = game.texture_manager.load("wood-tiles.png");
        let mut tile_tex = unsafe { &mut *game.tile_tex };
        tile_tex.add_frames(game.tile_frame_space.as_mut_slice(), 32, 32);
        // game.tile_positions = [
        //     SpriteData {
        //         position: Vector2::new(100.0, 100.0),
        //         frame: 2
        //     },
        //     SpriteData {
        //         position: Vector2::new(-200.0, -200.0),
        //         frame: 0
        //     }
        // ];
         let tilemap: [[uint, ..10], ..10] = [
            [8, 8, 8, 8, 8, 8, 8, 8, 8, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 2, 2, 2, 2, 2, 2, 2, 2, 8],
            [8, 8, 8, 8, 8, 8, 8, 8, 8, 8]
        ];

        // NOTE Slow but works
        game.tile_positions = unsafe { zeroed() };
        let mut count = 0u;
        for (x, ys) in tilemap.iter().enumerate() {
            for (y, frame) in ys.iter().enumerate() {
                game.tile_positions[count] = SpriteData {
                    position: Vector2::new(x as f32 * 32.0, y as f32 * 32.0 + 128.0),
                    frame: *frame as i32
                };
                count += 1;
            }
        }

        game.player_tex = game.texture_manager.load("dutchman.png");
        let mut player_tex = unsafe { &mut *game.player_tex };
        player_tex.add_frames(game.player_frame_space.as_mut_slice(), 32, 32);
        game.player_state = SpriteData {
            position: Vector2::new(256.0, 256.0),
            frame: 1
        };


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
            gen_buffer!(game.tile_vbo, game.tile_positions, ARRAY_BUFFER, STATIC_DRAW);
            gen_buffer!(game.player_vbo, [game.player_state], ARRAY_BUFFER, DYNAMIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // === Generate shaders ===
        game.shader_prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
        unsafe { gl::UseProgram(game.shader_prog) }
        game.cam_pos_uniform     = unsafe {     "cam_pos".with_c_str(|c| gl::GetUniformLocation(game.shader_prog, c)) };
        game.scale_uniform       = unsafe {       "scale".with_c_str(|s| gl::GetUniformLocation(game.shader_prog, s)) };
        game.sprite_size_uniform = unsafe { "sprite_size".with_c_str(|s| gl::GetUniformLocation(game.shader_prog, s)) };
        game.screen_size_uniform = unsafe { "screen_size".with_c_str(|s| gl::GetUniformLocation(game.shader_prog, s)) };
        game.tex_uniform         = unsafe {         "tex".with_c_str(|t| gl::GetUniformLocation(game.shader_prog, t)) };
        game.frames_uniform      = unsafe {      "frames".with_c_str(|f| gl::GetUniformLocation(game.shader_prog, f)) };
        unsafe {
            gl::Uniform2f(game.cam_pos_uniform, 0f32, 0f32);
            gl::Uniform1f(game.scale_uniform, 2.0);
            match window.get_size() {
                (width, height) => gl::Uniform2f(game.screen_size_uniform, width as f32, height as f32)
            }
        }
        tile_tex.generate_texcoords_buffer(&mut game.tile_texcoords_space);
        player_tex.generate_texcoords_buffer(&mut game.player_texcoords_space);

        game.cam_pos = Vector2::new(0.0, 0.0);
    }
}

#[no_mangle]
pub extern "C" fn update_and_render(
        game:    &mut Game,
        options: &mut Options,
        delta:   &Duration,
        glfw:    &glfw::Glfw,
        window:  &glfw::Window,
        event:   &GlfwEvent)
{
    glfw.poll_events();

    // TODO testing delta
    game.debug_flag += delta.num_milliseconds() as int;
    if game.debug_flag >= 1000 {
        game.player_state.frame = match game.player_state.frame {
            2 => 0,
            _ => game.player_state.frame + 1
        };
        game.debug_flag = 0;
    }

    let mut tile_tex  = unsafe { &mut *game.tile_tex };
    let zero_zero_tex = unsafe { &*game.zero_zero_tex };
    let mut player_tex = unsafe { &mut *game.player_tex };

    let mut controls = &mut options.controls;
    for control in controls.iter_mut() {
        control.last_frame = control.this_frame;
    }

    for (_, event) in glfw::flush_messages(event) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            glfw::WindowEvent::Key(key, _, action, _) => {
                let mut control = match key {
                    Key::W  => &mut controls.up,
                    Key::Up => &mut controls.up,

                    Key::S    => &mut controls.down,
                    Key::Down => &mut controls.down,

                    Key::A    => &mut controls.left,
                    Key::Left => &mut controls.left,

                    Key::D     => &mut controls.right,
                    Key::Right => &mut controls.right,

                    Key::B => &mut controls.debug,

                    _ => break
                };

                match action {
                    Action::Press => control.this_frame = true,
                    Action::Release => control.this_frame = false,
                    _ => {}
                }
            }

            glfw::WindowEvent::Size(width, height) => unsafe {
                println!("screen is now {} x {}", width, height);
                gl::Viewport(0, 0, width, height);
                gl::Uniform2f(game.screen_size_uniform, width as f32, height as f32);
            },

            _ => {}
        }
    }

    // === Reacting to input ===
    let delta_sec = delta.num_microseconds().unwrap() as f32 / 1_000_000.0;
    if controls.up.down() {
        game.cam_pos.y += 100.0 * delta_sec;
    }
    if controls.down.down() {
        game.cam_pos.y -= 100.0 * delta_sec;
    }
    if controls.left.down() {
        game.cam_pos.x -= 100.0 * delta_sec;
    }
    if controls.right.down() {
        game.cam_pos.x += 100.0 * delta_sec;
    }

    if controls.debug.just_down() {
        println!("Just down!");
    }
    if controls.debug.just_up() {
        println!("Just up!");
    }

    // === Updating buffers ===
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, game.player_vbo);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0,
                          size_of::<SpriteData>() as i64,
                          transmute(&game.player_state));
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // === Drawing ===
    unsafe {
        gl::Uniform2f(game.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);

        gl::ClearColor(0.1, 0.1, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw zero-zero sign
        let zero_zero_len = game.zero_zero_positions.len();
        zero_zero_tex.set_full(game.tex_uniform, game.sprite_size_uniform);
        set_sprite_attribute(game.zero_zero_vbo);
        gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), zero_zero_len as i32);
        // Draw tile!!!
        tile_tex.set(game.tex_uniform, game.sprite_size_uniform, game.frames_uniform, 32.0, 32.0);
        set_sprite_attribute(game.tile_vbo);
        gl::DrawElementsInstanced(
            gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), game.tile_positions.len() as i32
        );

        // Draw PLAYER
        player_tex.set(game.tex_uniform, game.sprite_size_uniform, game.frames_uniform, 32.0, 32.0);
        set_sprite_attribute(game.player_vbo);
        gl::DrawElementsInstanced(
            gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), 1
        );

        check_error!();
    }

    window.swap_buffers();
}

#[test]
fn it_works() {
    assert!(false);
}
