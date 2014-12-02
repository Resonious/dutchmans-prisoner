extern crate native;
extern crate core;
extern crate lodepng;
extern crate gl;
extern crate libc;

use gl::types::*;
use std::mem::transmute;
use std::vec::Vec;

use asset;

pub struct Texture {
    pub id: GLuint,
    pub width: i32,
    pub height: i32
}

impl Texture {
    // Set the texture to the TEXTURE0 uniform slot
    pub fn set(&self, sampler_uniform: GLint, sprite_size_uniform: GLint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::Uniform1i(sampler_uniform, 0);
            gl::Uniform2f(sprite_size_uniform, self.width as f32, self.height as f32);
        }
    }

    pub fn unload(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub struct TextureManager {
    pub textures: Vec<(&'static str, Texture)>
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager { textures: vec!() }
    }

    // If the texture with the given file name is present, return
    // the index of the texture.
    pub fn load(&mut self, filename: &'static str) -> uint {
        let mut textures = &mut self.textures;

        let mut count = 0;
        for item in textures.iter() {
            match *item {
                (ref tex_filename, _) =>
                    if *tex_filename == filename
                        { return count }
                    else
                        { count += 1 }
            }
        }

        let tex   = load_texture(filename);
        let index = textures.len();

        textures.push( (filename, tex) );
        return index;
    }

    // Unload texture with the given name. Returns true if it
    // was successfully removed.
    pub fn unload(&mut self, filename: &'static str) -> bool {
        let mut index = 0;

        for item in self.textures.iter() {
            match *item {
                (ref tex_filename, _) =>
                    if *tex_filename == filename
                        { break }
                    else
                        { index += 1 }
            }
        }

        self.unload_at(index)
    }

    pub fn unload_at(&mut self, index: uint) -> bool {
        panic!("Hey this function might suck because it changes the index by removing from the vector");

        /*
        match self.textures.remove(index) {
            Some( (_, mut texture) ) => {
                texture.unload();
                true
            }
            None => false
        }
        */
    }
}

// Load a texture from the given filename into the GPU
// memory, returning a struct holding the OpenGL ID and
// dimensions.
pub fn load_texture(filename: &str) -> Texture {
    let image = lodepng::decode32_file(&asset::path(filename)).unwrap();
    // println!("dimensions of {}: {}", filename, image.dimensions());
    let (width, height) = (image.width as i32, image.height as i32);

    let mut tex_id: GLuint = 0;

    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);

        // TODO Maybe change these around I dunno.....
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        // Set texture filtering
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        let buf = image.buffer.as_slice();

        println!("Sending {} to GPU. Width: {} Height: {}", filename, width, height);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32,
            width, height, 0, gl::RGBA,
            gl::UNSIGNED_BYTE, transmute(&buf[0])
        );
    }

    Texture {
        id: tex_id,
        width: width,
        height: height
    }
}

