extern crate native;
extern crate core;
extern crate lodepng;
extern crate gl;
extern crate libc;

use gl::types::*;
use std::mem::transmute;
use std::rc::Rc;

use asset;

// TODO THIS priobably needs a destructor :(
pub struct Texture {
    id: GLuint,
    width: i32,
    height: i32
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
}

// TODO this probably needs a destructor :(
pub struct TextureManager {
    textures: Vec<(&'static str, Rc<Texture>)>
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager { textures: vec![] }
    }

    pub fn load(&mut self, filename: &'static str) -> Rc<Texture> {
        let mut textures = &mut self.textures;

        for item in textures.iter() {
            match *item {
                (ref tex_filename, ref tex) => if *tex_filename == filename
                    { return tex.clone() }
            }
        }

        let tex = Rc::new(load_texture(filename));
        textures.push( (filename, tex) );
        match textures[textures.len() - 1] {
            (_, ref tex) => return tex.clone()
        }
    }
}

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

