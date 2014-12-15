extern crate core;
extern crate image;
extern crate gl;
extern crate libc;
extern crate cgmath;

use cgmath::*;
use gl::types::*;
use std::mem::transmute;
use std::vec::Vec;
use self::image::{GenericImage};

use asset;

pub type Texcoords = [GLfloat, ..8];

pub struct Frame {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,

    pub texcoords: Texcoords
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

pub struct FrameSet {
    pub frames: Vec<Frame>,
    // NOTE This is only mut because *const currently cannot coerce to *mut
    // cleanly and this isn't really a sensitive part of the code.
    texture: *mut Texture
}

impl FrameSet {
    pub fn add_frame(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.frames.push(
            Frame {
                position: Vector2::<f32>::new(x, y),
                size: Vector2::<f32>::new(width, height),
                texcoords: [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]
            }
        );
        let last_frame_index = self.frames.len() - 1;
        let mut frame = &mut self.frames[last_frame_index];
        frame.generate_texcoords(unsafe { &*self.texture });
    }

    pub fn add_frames(&mut self, count: uint, width: f32, height: f32) {
        let texture = unsafe { &*self.texture };
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

            self.add_frame(current_pos.x, current_pos.y, width, height);

            current_pos.x += width;
            counter += 1;
            if counter >= count { break }
        }
    }

    // This is for debugging purposes only
    pub fn print_frames(&self) {
        for frame in self.frames.iter() {
            println!("Position: {}, size: {}", frame.position, frame.size);
        }
    }
}

pub struct Texture {
    pub id: GLuint,
    pub width: i32,
    pub height: i32,
    pub filename: &'static str,
    pub frame_sets: Vec<FrameSet>
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

    // Generates a frame set with the given parameters, and returns its index.
    // The first frame set generated will be the default.
    pub fn generate_frames(&mut self, count: uint, width: f32, height: f32) -> uint {
        let index = self.frame_sets.len();
        self.frame_sets.push(
            FrameSet {
                frames: Vec::with_capacity(count),
                texture: self
            }
        );
        let frame_set = &mut self.frame_sets[index];
        frame_set.add_frames(count, width, height);
        index
    }

    // TODO man, should this be a destructor?
    pub fn unload(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub struct TextureManager {
    pub textures: Vec<Texture>
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
            if item.filename == filename {
                // println!("(TextureManager) found it!");
                return count;
            }
            else
                { count += 1 }
        }

        let index = textures.len();
        textures.push(load_texture(filename));
        // println!("(TextureManager) made it!");
        return index;
    }

    // Unload texture with the given name. Returns true if it
    // was successfully removed.
    pub fn unload(&mut self, filename: &'static str) -> bool {
        let mut index = 0;

        for item in self.textures.iter() {
            if item.filename == filename
                { break }
            else
                { index += 1 }
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
pub fn load_texture(filename: &'static str) -> Texture {
    // let image = lodepng::decode32_file(&asset::path(filename)).unwrap();
    // println!("dimensions of {}: {}", filename, image.dimensions());
    // let (width, height) = (image.width as i32, image.height as i32);

    let img = image::open(&asset::path(filename)).unwrap();
    let (width, height) = match img.dimensions() { (w, h) => (w as i32, h as i32) };

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

        let rgba = img.to_rgba();
        let buf = rgba.as_slice();

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
        height: height,
        filename: filename,
        frame_sets: vec![]
    }
}

