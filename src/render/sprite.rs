extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use cgmath::*;
use gl::types::*;
use std::vec::Vec;
use std::ptr;
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

    texcoords: Texcoords
}

impl Frame {
    pub fn generate_texcoords(&mut self, texture: &Texture) {
        let tex_width  = texture.width as f32;
        let tex_height = texture.height as f32;
        let position  = self.position;
        let size      = self.size;

        self.texcoords = [
            // Top right
            (position.x + size.x) / tex_width,
            (position.y)          / tex_height,

            // Bottom right
            (position.x + size.x) / tex_width,
            (position.y + size.y) / tex_height,

            // Top left
            (position.x)          / tex_width,
            (position.y)          / tex_height,

            // Bottom left
            (position.x)          / tex_width,
            (position.y + size.y) / tex_height
        ];
    }
}

pub struct Sprite {
    pub texture_manager: *mut TextureManager,
    pub texture_index: uint,
    // TODO gonna make frames attached to texturemanager
    // pub frames: Vec<Frame>,

    pub dirty: bool,
    // TODO This isn't even used?
    pub buffer_pos: i32
}


impl Sprite {
    // So basically, please destroy all sprites before destroying the texture manager.
    pub fn new(tex_manager: &mut TextureManager, tex: &'static str) -> Sprite {
        Sprite {
            texture_manager: tex_manager,
            texture_index: tex_manager.load(tex),
            dirty: false,

            // frames: vec!(),
            buffer_pos: 0
        }
    }

    pub fn blank() -> Sprite {
        Sprite {
            texture_manager: ptr::null_mut(),
            texture_index: 0,
            dirty: false,
            // frames: vec!(),
            buffer_pos: 0
        }
    }

    pub fn texture(&self) -> Option<*const Texture> {
        // if self.texture_index < 0 { return None }
        if self.texture_manager == ptr::null_mut()
            { return None; }

        unsafe {
            Some(&(*self.texture_manager).textures[self.texture_index] as *const Texture)
        }
    }

    // TODO hiding this stuff for until we make frames a part of texturemanager
    /*
    #[inline]
    fn add_frame_to_tex(&mut self, x: f32, y: f32,
                        width: f32, height: f32, texture: &Texture) {
        self.frames.push(
            Frame {
                position: Vector2::<f32>::new(x, y),
                size: Vector2::<f32>::new(width, height),
                texcoords: [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]
            }
        );
        let &mut frame = &self.frames[self.frames.len() - 1];
        frame.generate_texcoords(texture);
    }

    pub fn add_frame(&mut self, x: f32, y: f32, width: f32, height: f32) -> bool {
        let texture = match self.texture() {
            Some(tex) => unsafe { &*tex },
            None      => return false
        };

        self.add_frame_to_tex(x, y, width, height, texture);
        true
    }

    pub fn add_frames(&mut self, count: i16, width: f32, height: f32) {
        let texture = match self.texture() {
            Some(tex) => unsafe { &*tex },
            None      => return
        };
        let tex_width  = texture.width  as f32;
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

            self.add_frame_to_tex(current_pos.x, current_pos.y, width, height, texture);

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
    */
}
