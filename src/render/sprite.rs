extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use cgmath::*;
use std::vec::Vec;
use render::texture::*;

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

    // TODO pub fn add_frames or something!
}

