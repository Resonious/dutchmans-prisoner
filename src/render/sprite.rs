extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use cgmath::*;
use std::vec::Vec;
use render::texture::{Texture, TextureManager};

pub struct Frame {
    position: Vector2<f32>,
    size: Vector2<f32>
}

// impl Frame {
// }

pub struct Sprite {
    texture: *const Texture,
    frames: Vec<Frame>
}

impl Sprite {
    // So basically, please destroy all sprites before destroying the texture manager.
    pub fn new(tex_manager: &mut TextureManager, tex: &'static str) -> Sprite {
        Sprite {
            texture: tex_manager.load(tex),
            frames: vec!()
        }
    }

    pub fn add_frame(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.frames.push(
            Frame {
                position: Vector2::<f32>::new(x, y),
                size: Vector2::<f32>::new(width, height)
            }
        );
    }

    pub fn add_frames(&mut self, count: i16, width: f32, height: f32) {
        let texture = unsafe { &*self.texture };
        let tex_width = texture.width as f32;
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

