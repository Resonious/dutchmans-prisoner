extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use render::sprite::Sprite;
use gl::types::*;
use std::rc::Rc;
use std::mem::transmute;
use render::texture::{Texture, TextureManager};

pub struct DisplayList {
    vao: GLuint,
    texture_manager: *mut TextureManager,
    sprites_by_texture: Vec<Vec<Rc<Sprite>>>
}

impl DisplayList {
    // Just like Sprite, do not destroy this texture manager!
    pub fn new(texture_manager: &mut TextureManager) -> DisplayList {
        let mut vao: GLuint = 0;
        // gl::BindVertexArray(vao);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            DisplayList {
                vao: vao,
                texture_manager: texture_manager as *mut TextureManager,
                sprites_by_texture: vec!()
            }
        }
    }

    pub fn add_sprite(&mut self, sprite: Rc<Sprite>) -> bool {
        let texture = match sprite.texture() {
            Some(tex) => unsafe { transmute::<_, &Texture>(tex) },
            None      => return false
        };
        let mut by_texture = &mut self.sprites_by_texture;
        let by_texture_len = by_texture.len();
        let texture_id     = texture.id as uint;

        if texture_id >= by_texture_len {
            println!("(displaylist) growing by {}", texture_id - by_texture_len + 1);
            println!("(displaylist) sprite's texture id is {}, vector length is {}",
                     texture_id, by_texture_len);
            by_texture.grow(texture_id - by_texture_len + 1, Vec::with_capacity(0));
        }

        let mut sprite_list = &mut by_texture[texture_id];
        sprite_list.push(sprite.clone());
        true
    }

    pub fn insert_sprite(&mut self, tex: &'static str) -> Option<Rc<Sprite>> {
        let mut texture_manager_ptr = unsafe { transmute::<_, &mut TextureManager>(self.texture_manager) };
        let sprite_rc = Rc::new(Sprite::new(texture_manager_ptr, tex));

        unsafe {
            println!("(displaylist) I just made a sprite with texture_id {}",
                     (*sprite_rc.texture().unwrap()).id);
        }

        if self.add_sprite(sprite_rc.clone())
            { Some(sprite_rc.clone()) }
        else
            { None }
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

