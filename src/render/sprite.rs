extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use cgmath::*;
use gl::types::*;
use std::vec::Vec;
use render::texture::{Texture, TextureManager};

// pub struct Texcoord {
//     top_right:    Vector2<GLfloat>,
//     bottom_right: Vector2<GLfloat>,
//     top_left:     Vector2<GLfloat>,
//     bottom_left:  Vector2<GLfloat>
// }

pub type Texcoords = [GLfloat, ..8];

pub struct Frame {
    position: Vector2<f32>,
    size: Vector2<f32>,

    texture: *const Texture,
    texcoords: Texcoords
}

impl Frame {
    pub fn generate_texcoords(&mut self) {
        let tex_width = unsafe { (*self.texture).width } as f32;
        let position  = self.position;
        let size      = self.size;

        self.texcoords = [
            // Top right
            (position.x + size.x) / tex_width,
            (position.y)          / tex_width,

            // Bottom right
            (position.x + size.x) / tex_width,
            (position.y + size.y) / tex_width,

            // Top left
            (position.x)          / tex_width,
            (position.y)          / tex_width,

            // Bottom left
            (position.x)          / tex_width,
            (position.y + size.y) / tex_width
        ];
    }
}

pub struct Sprite {
    texture_manager: *mut TextureManager,
    texture_index: i32,
    // TODO maybe frames should not be attached to the sprite like this.
    // Maybe frames should be attached to TextureManager or something??
    pub frames: Vec<Frame>,

    pub buffer_pos: i32
}

impl Sprite {
    // So basically, please destroy all sprites before destroying the texture manager.
    pub fn new(tex_manager: &mut TextureManager, tex: &'static str) -> Sprite {
        Sprite {
            texture_manager: tex_manager,
            texture_index: tex_manager.load(tex),

            frames: vec!(),
            buffer_pos: 0
        }
    }

    pub unsafe fn texture(&self) -> Option(*const Texture) {
        if self.texture_index < 0 { return None }

        match (*self.texture_manager).textures[self.texture_index] {
            (_, ref texture) => Some(texture),
            _ => None
        }
    }

    pub fn add_frame(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.frames.push(
            Frame {
                position: Vector2::<f32>::new(x, y),
                size: Vector2::<f32>::new(width, height),
                texture: self.texture,
                texcoords: [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]
            }
        );
        let &mut frame = &self.frames[self.frames.len() - 1];
        frame.generate_texcoords();
    }

    pub fn add_frames(&mut self, count: i16, width: f32, height: f32) {
        let texture    = unsafe { &*self.texture };
        let tex_width  = texture.width as f32;
        let tex_height = texture.height as f32;

        let mut current_pos = Vector2::<f32>::new(0.0, 0.0);
        let mut counter = 0;

        loop {
            if current_pos.x + width > tex_width {
                current_pos.x = 0.0;
                current_pos.y += height;
            }
            if current_pos.y + height > tex_height {
                panic!(
                    "Too many frames! Asked for {} {}x{} frames on a {}x{} texture.",
                    count, width, height, tex_width, tex_height
                );
            }

            self.add_frame(current_pos.x, current_pos.y, width, height);

            current_pos.x += width;
            counter += 1;
            if counter == count { break }
        }
    }

    // This is for debugging purposes only
    pub fn print_frames(&self) {
        for frame in self.frames.iter() {
            println!("Position: {}, size: {}", frame.position, frame.size);
        }
    }
}

/*
impl Clone for Sprite {
    fn clone(&self) -> Sprite {
        Sprite {
            texture: self.texture,
            // For now, frames are not copied.
            frames: vec!(),
            buffer_pos: self.buffer_pos
        }
    }
}
*/

