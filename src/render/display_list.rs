extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use render::sprite::Sprite;
use gl::types::*;
// use std::mem::transmute;
use render::texture::{TextureManager};
use std::mem::{transmute, replace};

pub struct DisplayList<'d> {
    vao: GLuint,
    texture_manager: *mut TextureManager,
    // sprites_by_texture: Vec<Vec<Sprite>>,
    pub sprites: &'d mut[Sprite],
    pub unused_indices: Vec<uint>,
    pub sprite_count: uint
}

impl<'d> DisplayList<'d> {
    // Just like with Sprite, do not destroy this texture manager!
    pub fn new<'a>(texture_manager: &mut TextureManager,
                   sprite_space:    &'a mut[Sprite]) -> DisplayList<'a> {
        let mut vao: GLuint = 0;
        // gl::BindVertexArray(vao);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            DisplayList {
                vao: vao,
                texture_manager: texture_manager as *mut TextureManager,
                // sprites_by_texture: vec!()
                sprites: sprite_space,
                unused_indices: vec!(),
                sprite_count: 0
            }
        }
    }

    // Adds existing sprite data to the sprite buffer.
    pub fn add_sprite(&mut self, sprite: Sprite) -> bool {
        if self.sprite_count >= self.sprites.len() {
            println!("DisplayList ran out of room!");
            return false;
        }

        let mut unused = &mut self.unused_indices;
        if unused.len() > 0 {
            let index = unused.pop().unwrap();
            replace(&mut self.sprites[index], sprite);
        }
        else {
            replace(&mut self.sprites[self.sprite_count], sprite);
            self.sprite_count += 1;
        }

        true
    }

    // Creates a new sprite with the given texture, and
    // returns a pointer to it if it succeeds.
    pub fn insert_sprite(&mut self, tex: &'static str) -> Option<*mut Sprite> {
        let mut texture_manager_ptr = unsafe { &mut*self.texture_manager };
        if self.add_sprite(Sprite::new(texture_manager_ptr, tex)) {
            Some(&mut self.sprites[self.sprite_count - 1] as *mut Sprite)
        }
        else { None }
    }
}

impl<'d> Drop for DisplayList<'d> {
    // Delete VAO associated with the display list.
    #![unsafe_destructor]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
