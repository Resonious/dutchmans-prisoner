extern crate core;
extern crate image;
extern crate gl;
extern crate libc;
extern crate cgmath;

use cgmath::*;
use gl::types::*;
use std::mem::{size_of, transmute, uninitialized};
use std::vec::Vec;
use self::image::{GenericImage};
use gl::types::*;

use asset;

pub type Texcoords = [GLfloat, ..8];

// Represents an animation frame; a square section of a Texture.
pub struct Frame {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,

    // Texcoords are generated via generate_texcoords.
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
    pub frames: Box<[Frame]>,
    pub offset: i32
}

// Represents an actual texture that is currently on the GPU.
pub struct Texture {
    pub id: GLuint,
    pub width: i32,
    pub height: i32,
    pub filename: &'static str,
    pub frame_sets: Vec<FrameSet>,
    pub frame_sets_vbo: GLuint
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

    // This will assign the offset field to all its framesets.
    // And assign the field in the texture.
    pub fn generate_frames_vbo(&mut self) {
        // Create vbo
        unsafe {
            gl::GenBuffers(1, &mut self.frame_sets_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.frame_sets_vbo);
        }
        // Send the texcoords of every frame
        let mut frame_set_offset = 0i32;
        for frame_set in self.frame_sets.iter_mut() {
            frame_set.offset = frame_set_offset;

            for frame in frame_set.frames.iter() {
                unsafe {
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        frame_set_offset as i64,
                        size_of::<Texcoords>() as i64,
                        transmute(&frame.texcoords[0])
                    );
                }
                frame_set_offset += size_of::<Texcoords>() as i32;
            }
        }
    }

    pub fn add_frame_set<'t>(&'t mut self, count: uint, width: uint, height: uint) {
        let frames = generate_frames(self, count, width as f32, height as f32);
        self.frame_sets.push(
            FrameSet {
                frames: frames,
                offset: -1
            }
        );
    }

    // TODO man, should this be a destructor?
    pub fn unload(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

// Makes sure the same texture is never loaded twice.
pub struct TextureManager {
    pub textures: Vec<Texture>
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager { textures: vec!() }
    }

    // If the texture with the given file name is present, return
    // the index of the texture.
    pub fn load(&mut self, filename: &'static str) -> *mut Texture {
        let mut textures = &mut self.textures;

        let mut count = 0u32;
        for item in textures.iter_mut() {
            if item.filename == filename {
                // println!("(TextureManager) found it!");
                return item;
            }
            else
                { count += 1 }
        }

        let index = textures.len();
        textures.push(load_texture(filename));
        // println!("(TextureManager) made it!");
        return &mut textures[index];
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
        frame_sets: vec![],
        frame_sets_vbo: 0
    }
}

pub fn generate_frames<'t>(texture: &'t Texture, count: uint, width: f32, height: f32) -> Box<[Frame]> {
    let tex_width  = texture.width as f32;
    let tex_height = texture.height as f32;

    let mut current_pos = Vector2::<f32>::new(0.0, 0.0);

    Vec::from_fn(count, |_| {
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

        let mut frame = Frame {
            position:  current_pos,
            size:      Vector2::new(width, height),
            texcoords: unsafe { uninitialized() }
        };
        frame.generate_texcoords(texture);

        current_pos.x += width;

        frame
    }).into_boxed_slice()
}
