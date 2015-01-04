#![feature(globs, macro_rules)]
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::{Context, Action, Key};

use render::shader;
use render::texture;
use render::texture::{Texture, Texcoords, Frame};
use render::sprite::*;
use controls::{Controls};
use std::mem::{transmute, size_of, size_of_val, zeroed};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;
use std::time::duration::Duration;
use std::num::Float;
use std::slice;

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
        assert_eq!(size_of_sprite, 16);

        // == Position ==
        gl::EnableVertexAttribArray(shader::ATTR_POSITION);
        gl::VertexAttribPointer(
            shader::ATTR_POSITION, 2, gl::FLOAT, gl::FALSE as GLboolean,
            size_of_sprite, as_void!(0)
        );
        gl::VertexAttribDivisor(shader::ATTR_POSITION, 1);
        let mut offset = 2 * size_of::<GLfloat>() as i64;
        assert_eq!(offset, 8);

        // == Frame ==
        gl::EnableVertexAttribArray(shader::ATTR_FRAME);
        gl::VertexAttribIPointer(
            shader::ATTR_FRAME, 1, gl::INT,
            size_of_sprite, as_void!(offset)
        );
        gl::VertexAttribDivisor(shader::ATTR_FRAME, 1);
        offset += 1 * size_of::<GLint>() as i64;
        assert_eq!(offset, 12);

        // == Flipped ==
        gl::EnableVertexAttribArray(shader::ATTR_FLIPPED);
        gl::VertexAttribIPointer(
            shader::ATTR_FLIPPED, 1, gl::INT,
            size_of_sprite, as_void!(offset)
        );
        gl::VertexAttribDivisor(shader::ATTR_FLIPPED, 1);

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

pub struct GlData {
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

    pub zero_zero_tex: Texture,
    pub zero_zero_vbo: GLuint,

    pub tile_tex: Texture,
    pub tile_texcoords: [Texcoords, ..14],
    pub tile_vbo: GLuint,

    pub player_tex: Texture,
    pub player_texcoords: [Texcoords, ..3],
    pub player_vbo: GLuint,
}

pub struct Game {
    pub initialized: bool,

    // NOTE Do whatever you want with this.
    pub debug_flag: int,

    pub zero_zero_positions: [SpriteData, ..1],

    pub tile_frame_space: [Frame, ..14], // <- number of frames.
    pub tilemap_position: Vector2<GLfloat>,
    pub tilemap: [[i32, ..10], ..10],
    // pub tile_positions: [SpriteData, ..10*10],

    pub player_frame_space: [Frame, ..3],
    pub player_state: SpriteData,

    pub cam_pos: Vector2<GLfloat>,
}

#[no_mangle]
pub extern "C" fn load(fresh_load: bool,
                       glfw_data: *const u8,
                       window:    &glfw::Window,
                       game:      &mut Game,
                       options:   &mut Options,
                       gldata:   &mut GlData)
{
    unsafe {
        println!("Loading up!");
        glfwInit();
        glfwCopyDataFrom(glfw_data); 
        window.make_current();
        gl::load_with(|s| window.get_proc_address(s));
    }

    // === Initialize game state ===
    if !game.initialized {
        game.initialized = true;

        game.cam_pos = Vector2::new(0.0, 0.0);

        game.zero_zero_positions = [
            SpriteData {
                position: Vector2::new(0.0, 0.0),
                frame: -1,
                flipped: false as GLint
            }
        ];

        game.player_state = SpriteData {
            position: Vector2::new(256.0, 256.0),
            frame: 1,
            flipped: true as GLint
        };

        game.tilemap_position.x = 20.0;
        game.tilemap_position.y = 128.0;
        game.tilemap = [
            [9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 11, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 11, 2, 2, 9],
            [9, 2, 2, 2, 2, 2, 2, 2, 2, 9],
            [9, 8, 8, 8, 7, 8, 8, 8, 8, 9]
        ];

        game.debug_flag = 0;
    }

    // === Initialize GL data if necessary ===
    if fresh_load {
        // TODO load options from a file
        options.controls = unsafe { zeroed() };

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // === Generate textures and the like ===
        gldata.zero_zero_tex = texture::load_texture("zero-zero.png");

        gldata.tile_tex = texture::load_texture("wood-tiles.png");
        gldata.tile_tex.add_frames(game.tile_frame_space.as_mut_slice(), 32, 32);

        gldata.player_tex = texture::load_texture("dutchman.png");
        gldata.player_tex.add_frames(game.player_frame_space.as_mut_slice(), 32, 32);

        // === Generate global VAO ===
        unsafe {
            gl::GenVertexArrays(1, &mut gldata.vao);
            gl::BindVertexArray(gldata.vao);

            // === Generate and populate global rectangle buffers ===
            gen_buffer!(gldata.square_vbo, SQUARE_VERTICES, ARRAY_BUFFER, STATIC_DRAW);
            gen_buffer!(gldata.square_ebo, SQUARE_INDICES, ELEMENT_ARRAY_BUFFER, STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, gldata.square_vbo);
            gl::EnableVertexAttribArray(shader::ATTR_VERTEX_POS);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                    stride!(2), as_void!(0));

            // === Generate (by hand) stuff on the screen ===
            gen_buffer!(gldata.zero_zero_vbo, game.zero_zero_positions, ARRAY_BUFFER, DYNAMIC_DRAW);
            gen_buffer!(gldata.player_vbo, [game.player_state], ARRAY_BUFFER, DYNAMIC_DRAW);

            gl::GenBuffers(1, &mut gldata.tile_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, gldata.tile_vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                (10 * 10) * size_of::<SpriteData>() as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW
            );

            let mut tile_buf = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
            let mut tile_sprites = slice::from_raw_mut_buf::<SpriteData>(
                transmute(&tile_buf),
                10 * 10
            );
            let mut count = 0u;
            for (y, xs) in game.tilemap.iter().enumerate() {
                for (x, frame) in xs.iter().enumerate() {
                    tile_sprites[count] = SpriteData {
                        position: Vector2::new(x as f32 * 32.0, y as f32 * 32.0) + game.tilemap_position,
                        frame: *frame,
                        flipped: false as GLint
                    };
                    count += 1;
                }
            }
            gl::UnmapBuffer(gl::ARRAY_BUFFER);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // === Generate shaders ===
        if !compile_shaders(gldata, game, window) {
            panic!("Failed to compile or link shaders.");
        }
        gldata.player_tex.generate_texcoords_buffer(&mut gldata.player_texcoords);
        gldata.tile_tex.generate_texcoords_buffer(&mut gldata.tile_texcoords);
    }
    // if NOT fresh_load:
    else {
        if !compile_shaders(gldata, game, window) {
            println!("ERROR COMPILING SHADERS. Shaders not reloaded.");
        }
    }
}

fn compile_shaders(gl_data: &mut GlData, game: &Game, window: &glfw::Window) -> bool {
    let existing_program = unsafe {
        if gl::IsProgram(gl_data.shader_prog) == gl::TRUE {
            Some(gl_data.shader_prog)
        } else { None }
    };

    gl_data.shader_prog = match shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT) {
        Some(program) => program,
        None          => return false
    };
    unsafe { gl::UseProgram(gl_data.shader_prog) }
    gl_data.cam_pos_uniform     = unsafe {     "cam_pos".with_c_str(|c| gl::GetUniformLocation(gl_data.shader_prog, c)) };
    gl_data.scale_uniform       = unsafe {       "scale".with_c_str(|s| gl::GetUniformLocation(gl_data.shader_prog, s)) };
    gl_data.sprite_size_uniform = unsafe { "sprite_size".with_c_str(|s| gl::GetUniformLocation(gl_data.shader_prog, s)) };
    gl_data.screen_size_uniform = unsafe { "screen_size".with_c_str(|s| gl::GetUniformLocation(gl_data.shader_prog, s)) };
    gl_data.tex_uniform         = unsafe {         "tex".with_c_str(|t| gl::GetUniformLocation(gl_data.shader_prog, t)) };
    gl_data.frames_uniform      = unsafe {      "frames".with_c_str(|f| gl::GetUniformLocation(gl_data.shader_prog, f)) };
    unsafe {
        gl::Uniform2f(gl_data.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);
        gl::Uniform1f(gl_data.scale_uniform, 2.0);
        match window.get_size() {
            (width, height) => gl::Uniform2f(gl_data.screen_size_uniform, width as f32, height as f32)
        }
    }

    match existing_program {
        Some(program) => unsafe { gl::DeleteProgram(program) },
        _ => {}
    }

    true
}

// NOTE A negative amount will cause us to go backwards (no duh, right).
fn towards(start: f32, target: f32, amount: f32) -> f32 {
    let mut value = start;
    if value > target {
        value -= amount;
        if value < target { value = target }
    }
    else if value < target {
        value += amount;
        if start > target { value = target }
    }
    value
}

#[test]
fn towards_works() {
    assert_eq!(towards(10, 20, 5), 15);
}

fn pos_to_tile(position: Vector2<f32>, tilemap_position: Vector2<f32>) -> Vector2<f32> {
    let offset_pos = position - tilemap_position;
    // NOTE assumes 32*32 tiles.
    Vector2::new(offset_pos.x / 32.0, offset_pos.y / 32.0)
}

fn pos_to_tile_index(position: Vector2<f32>, tilemap_position: Vector2<f32>) -> Vector2<i32> {
    let float_tile_pos = pos_to_tile(position, tilemap_position);
    Vector2::new(float_tile_pos.x.floor() as i32, float_tile_pos.y.floor() as i32)
}

#[no_mangle]
pub extern "C" fn update_and_render(
        game:    &mut Game,
        options: &mut Options,
        gl_data: &mut GlData,
        delta:   &Duration,
        glfw:    &glfw::Glfw,
        window:  &glfw::Window,
        event:   &GlfwEvent)
{
    glfw.poll_events();

    // TODO testing delta
    // game.debug_flag += delta.num_milliseconds() as int;
    // if game.debug_flag >= 1000 {
    //     game.player_state.frame = match game.player_state.frame {
    //         2 => 0,
    //         _ => game.player_state.frame + 1
    //     };
    //     game.debug_flag = 0;
    // }

    let tile_tex      = &gl_data.tile_tex;
    let zero_zero_tex = &gl_data.zero_zero_tex;
    let player_tex    = &gl_data.player_tex;

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
                let control = match key {
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
                gl::Uniform2f(gl_data.screen_size_uniform, width as f32, height as f32);
            },

            _ => {}
        }
    }

    // === Reacting to input ===
    let delta_sec = delta.num_microseconds().unwrap() as f32 / 1_000_000.0;
    let mut target_player_pos = game.player_state.position.clone();

    if controls.left.down() {
        target_player_pos.x -= 100.0 * delta_sec;
        game.player_state.frame = 1;
        game.player_state.flipped = false as GLint;
    }
    if controls.right.down() {
        target_player_pos.x += 100.0 * delta_sec;
        game.player_state.frame = 1;
        game.player_state.flipped = true as GLint;
    }
    if controls.up.down() {
        target_player_pos.y += 100.0 * delta_sec;
        game.player_state.frame = 2;
        game.player_state.flipped = false as GLint;
    }
    if controls.down.down() {
        target_player_pos.y -= 100.0 * delta_sec;
        game.player_state.frame = 0;
        game.player_state.flipped = false as GLint;
    }
    // Dumb collision
    let center_offset = Vector2::new(16.0, 0.0);
    let target_tile_pos = pos_to_tile(target_player_pos + center_offset,
                                      game.tilemap_position);
    // TODO Move the player one axis at a time:
    // split it into target_x_tile and target_y_tile where
    // target_x_tile = pos_to_tile((target_pos.x, player_pos.y), tilemap_position)
    // etc.
    // This way the player does not get stuck on tile edges when hugging the walls.
    let target_tile = game.tilemap[target_tile_pos.y.floor() as uint]
                                  [target_tile_pos.x.floor() as uint];
    game.player_state.position =
        if target_tile == 9 || target_tile == 8 {
            let current_tile_index = pos_to_tile_index(
                game.player_state.position + center_offset, game.tilemap_position
            );
            let target_tile_index = Vector2::new(
                target_tile_pos.x.floor() as i32,
                target_tile_pos.y.floor() as i32
            );
            let wall_direction = target_tile_index - current_tile_index;

            let mut offset_past_wall = Vector2::new(
                target_tile_pos.x - target_tile_pos.x.floor(),
                target_tile_pos.y - target_tile_pos.y.floor()
            );
            if wall_direction.x < 0 {
                offset_past_wall.x = 1.0 - offset_past_wall.x;
            }
            if wall_direction.y < 0 {
                offset_past_wall.y = 1.0 - offset_past_wall.y;
            }

            let mut offset = Vector2::from_value(0.0);
            if wall_direction.x != 0 {
                offset.x += offset_past_wall.x * wall_direction.x as f32;
                // Convert into pixels:
                // As usual, assumes 32x32 tiles.
                offset.x *= 32.0;
                offset.x += 1.0 * wall_direction.x as f32;
            }
            if wall_direction.y != 0 {
                offset.y += offset_past_wall.y * wall_direction.y as f32;
                // Convert into pixels:
                offset.y *= 32.0;
                offset.y += 1.0 * wall_direction.y as f32;
            }

            // println!("wall direction: {}", wall_direction);

            // target_player_pos - Vector2::new((wall_direction.x * 10) as f32,
            //                                 (wall_direction.y * 10) as f32)

            target_player_pos - offset
        }
        else { target_player_pos };


    // === Updating camera position ===
    game.cam_pos.y = towards(
        game.cam_pos.y, game.player_state.position.y,
        ((game.player_state.position.y - game.cam_pos.y).abs() * 10.0) * delta_sec
    );
    game.cam_pos.x = towards(
        game.cam_pos.x, game.player_state.position.x,
        ((game.player_state.position.x - game.cam_pos.x).abs() * 10.0) * delta_sec
    );

    // === Updating buffers ===
    // Player
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.player_vbo);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0,
                          size_of::<SpriteData>() as i64,
                          transmute(&game.player_state));
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
    // Tilemap
    unsafe {
        let player_tile = pos_to_tile_index(
            game.player_state.position + center_offset, game.tilemap_position
        );

        if controls.debug.just_down() {
            println!("tile num: {}", player_tile);
        }

        let stepped_on_tile = SpriteData {
            position: Vector2::new(
                          player_tile.x as f32 * 32.0, player_tile.y as f32 * 32.0
                      ) + game.tilemap_position,
            frame: 0,
            flipped: false as GLint
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.tile_vbo);
        let byte_offset = (player_tile.x as uint +  10 * player_tile.y as uint) * size_of::<SpriteData>();
        gl::BufferSubData(gl::ARRAY_BUFFER, byte_offset as i64,
                          size_of::<SpriteData>() as i64,
                          transmute(&stepped_on_tile));
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // === Drawing ===
    unsafe {
        gl::Uniform2f(gl_data.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);

        gl::ClearColor(0.1, 0.1, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw zero-zero sign
        let zero_zero_len = game.zero_zero_positions.len();
        zero_zero_tex.set_full(gl_data.tex_uniform, gl_data.sprite_size_uniform);
        set_sprite_attribute(gl_data.zero_zero_vbo);
        gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), zero_zero_len as i32);
        // Draw tile!!!
        tile_tex.set(gl_data.tex_uniform, gl_data.sprite_size_uniform, gl_data.frames_uniform, 32.0, 32.0);
        set_sprite_attribute(gl_data.tile_vbo);
        gl::DrawElementsInstanced(
            gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), 10 * 10
        );

        // Draw PLAYER
        player_tex.set(gl_data.tex_uniform, gl_data.sprite_size_uniform, gl_data.frames_uniform, 32.0, 32.0);
        set_sprite_attribute(gl_data.player_vbo);
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
