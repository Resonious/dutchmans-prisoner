extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use render::sprite::Sprite;
use gl::types::*;
use std::mem::transmute;
use render::texture::{Texture, TextureManager};

pub struct DisplayList<'d> {
    vao: GLuint,
    texture_manager: *mut TextureManager,
    // sprites_by_texture: Vec<Vec<Sprite>>,
    pub sprites: &'d mut[Sprite],
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
                sprite_count: 0
            }
        }
    }

    pub fn add_sprite(&mut self, sprite: Sprite) -> bool {
        if self.sprite_count >= self.sprites.len() {
            println!("DisplayList ran out of room!");
            false
        }
        else {
            self.sprites[self.sprite_count] = sprite;
            self.sprite_count += 1;
            true
        }
    }

    pub fn insert_sprite(&mut self, tex: &'static str) -> Option<*mut Sprite> {
        let mut texture_manager_ptr = unsafe { &mut*self.texture_manager };
        if self.add_sprite(Sprite::new(texture_manager_ptr, tex)) {
            Some(&mut self.sprites[self.sprite_count - 1] as *mut Sprite)
        }
        else { None }
    }

    // pub fn add_sprite(&mut self, sprite: Sprite) -> bool {
    //     let texture = match sprite.texture() {
    //         Some(tex) => unsafe { transmute::<_, &Texture>(tex) },
    //         None      => return false
    //     };
    //     let mut by_texture = &mut self.sprites_by_texture;
    //     let by_texture_len = by_texture.len();
    //     let texture_id     = texture.id as uint;

    //     if texture_id >= by_texture_len {
    //         println!("(displaylist) growing by {}", texture_id - by_texture_len + 1);
    //         println!("(displaylist) sprite's texture id is {}, vector length is {}",
    //                  texture_id, by_texture_len);
    //         by_texture.grow(texture_id - by_texture_len + 1, Vec::with_capacity(0));
    //     }

    //     let mut sprite_list = &mut by_texture[texture_id];
    //     sprite_list.push(sprite);
    //     true
    // }

    // pub fn insert_sprite(&mut self, tex: &'static str) -> Option<&Sprite> {
    //     let mut texture_manager_ptr = unsafe { transmute::<_, &mut TextureManager>(self.texture_manager) };
    //     // let sprite_rc = Rc::new(Sprite::new(texture_manager_ptr, tex));

    //     unsafe {
    //         println!("(displaylist) I just made a sprite with texture_id {}",
    //                  (*sprite_rc.texture().unwrap()).id);
    //     }

    //     if self.add_sprite(Sprite::new(texture_manager_ptr, tex))



    //     if self.add_sprite(sprite_rc.clone())
    //         { Some(sprite_rc.clone()) }
    //     else
    //         { None }
    //
    // }
}

impl<'d> Drop for DisplayList<'d> {
    #![unsafe_destructor]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

