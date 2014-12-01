#![feature(globs, unsafe_destructor)]
extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use render::sprite::Sprite;
use gl::types::*;
use std::rc::Rc;

pub struct DisplayList {
    vao: GLuint,
    sprites_by_texture: Vec<Vec<Rc<Sprite>>>
}

impl DisplayList {
    pub fn new() -> DisplayList {
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        // gl::BindVertexArray(vao);

        DisplayList {
            vao: vao,
            sprites_by_texture: vec!()
        }
    }

    pub fn add_sprite(&mut self, sprite: Rc<Sprite>) {
        let texture        = sprite.texture_ref();
        let mut by_texture = &mut self.sprites_by_texture;
        let texture_len    = by_texture.len();
        let texture_id     = texture.id as uint;

        if texture_id >= texture_len {
            println!("(displaylist) growing by {}", texture_id - texture_len + 1);
            by_texture.grow(texture_id - texture_len + 1, Vec::with_capacity(0));
        }

        let mut sprite_list = &mut by_texture[texture_id];
        sprite_list.push(sprite.clone());
    }
}

impl Drop for DisplayList {
    #![unsafe_destructor]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

