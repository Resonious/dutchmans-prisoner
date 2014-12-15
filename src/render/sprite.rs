extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use std::ptr;
use render::texture::{Texture, TextureManager, Frame};

#[deriving(Copy)]
pub struct Sprite {
    // === Crazy internal stuff ===
    pub texture_manager: *mut TextureManager,
    pub texture_index: uint,
    pub frame_set_index: uint,

    // TODO do we want dirty?
    // Do we want the physics component or whatever to sync its position
    // with sprite, which syncs its position with the buffer?
    // Or do we want a function that directly sends a position/rotation/
    // whatever to gpu, so that physics can say 'ok use my stuff'
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
            frame_set_index: 0,
            dirty: false,

            // frames: vec!(),
            buffer_pos: 0
        }
    }

    pub fn blank() -> Sprite {
        Sprite {
            texture_manager: ptr::null_mut(),
            texture_index: 0,
            frame_set_index: 0,
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
            let texture_manager = &*self.texture_manager;
            Some(&texture_manager.textures[self.texture_index] as *const Texture)
        }
    }

    pub fn frames(&self) -> Option<&[Frame]> {
        match self.texture() {
            None => return None,

            Some(texture_ptr) => {
                let texture    = unsafe { &*texture_ptr };
                let frame_sets = &texture.frame_sets;

                if frame_sets.len() <= self.frame_set_index
                    { return None }

                let set    = &frame_sets[self.frame_set_index];
                let frames = &set.frames;
                // Technically this is super unsafe, but the frame vector
                // within a FrameSet will never change.
                Some(frames.as_slice())
            }
        }
    }

    pub fn set_frames(&mut self, count: uint, width: f32, height: f32) {
        panic!("Implement me or gtfo");
    }
}
