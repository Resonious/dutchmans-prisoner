extern crate core;
extern crate image;
extern crate gl;
extern crate libc;
extern crate cgmath;

use cgmath::*;
use gl::types::*;
use std::mem::{size_of, transmute, zeroed, uninitialized, transmute_copy};
use std::vec::Vec;
use std::ptr;
use self::image::{GenericImage};
use gl::types::*;
use render::shader;
use libc::c_void;

use asset;

#[deriving(Clone)]
pub struct Texcoords {
    pub top_right:    Vector2<GLfloat>,
    pub bottom_right: Vector2<GLfloat>,
    pub bottom_left:  Vector2<GLfloat>,
    pub top_left:     Vector2<GLfloat>
}

// Represents an animation frame; a square section of a Texture.
pub struct Frame {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,

    // Texcoords are generated via generate_texcoords.
    pub texcoords: Texcoords
}

impl Frame {
    pub fn generate_texcoords(&mut self, tex_width: f32, tex_height: f32) {
        let position  = self.position;
        let size      = self.size;

        self.texcoords = Texcoords {
            top_right: Vector2::new(
                (position.x + size.x) / tex_width,
                (position.y + size.y) / tex_height
            ),

            bottom_right: Vector2::new(
                (position.x + size.x) / tex_width,
                (position.y)          / tex_height
            ),

            bottom_left: Vector2::new(
                (position.x)          / tex_width,
                (position.y)          / tex_height
            ),

            top_left: Vector2::new(
                (position.x)          / tex_width,
                (position.y + size.y) / tex_height
            )
        };
    }
}

// Represents an actual texture that is currently on the GPU.
pub struct Texture {
    pub id: GLuint,
    pub width: i32,
    pub height: i32,
    pub filename: &'static str,
    pub frame_space: *mut [Frame],
    pub frame_texcoords_size: i64,
    pub texcoords_space: *mut [Texcoords]
}

impl Texture {
    pub fn set_full(&self, sampler_uniform: GLint, sprite_size_uniform: GLint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::Uniform1i(sampler_uniform, 0);
            gl::Uniform2f(sprite_size_uniform, self.width as f32, self.height as f32);
        }
    }

    #[inline]
    pub fn frames_mut(&mut self) -> &mut [Frame] {
        unsafe { transmute(self.frame_space) }
    }

    #[inline]
    pub fn frames(&self) -> &[Frame] {
        unsafe { transmute(self.frame_space) }
    }

    #[inline]
    pub fn texcoords(&self) -> &[Texcoords] {
        unsafe { transmute(self.texcoords_space) }
    }

    #[inline]
    pub fn texcoords_mut(&mut self) -> &mut [Texcoords] {
        unsafe { transmute(self.texcoords_space) }
    }

    #[inline]
    pub fn frame_at_mut(&mut self, i: uint) -> &mut Frame {
        let mut frames = self.frames();
        unsafe { transmute(&frames[i]) }
    }

    #[inline]
    pub fn frame_at(&self, i: uint) -> &Frame {
        let frames = self.frames();
        &frames[i]
    }

    // NOTE this expects generate_texcoords_buffer to have been called
    // if there are frames.
    pub fn set(&self, sampler_uniform:     GLint,
                      sprite_size_uniform: GLint,
                      frames_uniform:      GLint,
                      width: f32, height: f32) {
        unsafe {
            assert!(self.frame_texcoords_size / 8 < shader::FRAME_UNIFORM_MAX);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::Uniform1i(sampler_uniform, 0);
            gl::Uniform2f(sprite_size_uniform, width as f32, height as f32);

            let frames_len = self.frames().len();

            if frames_len > 0 {
                gl::Uniform2fv(
                    frames_uniform,
                    frames_len as GLint * 4,
                    transmute(&(&*self.texcoords_space)[0])
                );
            }
        }
    }

    fn put_texcoord(&mut self, index: uint, texcoord: Texcoords) {
        self.texcoords_mut()[index] = texcoord;
    }

    pub fn generate_texcoords_buffer(&mut self, space: *mut [Texcoords]) {
        let frames_len = self.frames().len();
        unsafe { assert_eq!(frames_len, (&*space).len()); }
        if frames_len == 0 { return; }

        self.texcoords_space = space;
        for i in range(0u, frames_len) {
            let texcoords = self.frame_at(i).texcoords.clone();
            self.put_texcoord(i, texcoords);
        }
    }

    // Fill the given slice with frames of the given width and height.
    pub fn add_frames(&mut self, space: *mut [Frame], uwidth: uint, uheight: uint) {
        let count = unsafe { (*space).len() };
        let tex_width  = self.width as f32;
        let tex_height = self.height as f32;
        let width  = uwidth as f32;
        let height = uheight as f32;

        self.frame_space = space;
        {
            let mut frames = self.frames_mut();

            let mut current_pos = Vector2::<f32>::new(0.0, tex_height - height);

            for i in range(0u, count) {
                if current_pos.x + width > tex_width {
                    current_pos.x = 0.0;
                    current_pos.y -= height;
                }
                if current_pos.y < 0.0 {
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
                frame.generate_texcoords(tex_width, tex_height);
                frames[i] = frame;

                current_pos.x += width;
            }
        }

        self.frame_texcoords_size += size_of::<Texcoords>() as i64 * count as i64;
    }

    // TODO man, should this be a destructor?
    // A: NO
    pub fn unload(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

// Makes sure the same texture is never loaded twice.
pub struct TextureManager {
    // NOTE Remember to bump this up if you run into issues.
    pub textures: [Texture, ..10],
    next_index: uint
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager {
            textures: unsafe { zeroed() },
            next_index: 0
        }
    }

    // If the texture with the given file name is present, return
    // a pointer to the texture.
    pub fn load(&mut self, filename: &'static str) -> *mut Texture {
        let mut textures = &mut self.textures;

        for item in textures.iter_mut() {
            if item.filename == filename {
                // println!("(TextureManager) found it!");
                return item;
            }
        }

        let index = self.next_index;
        self.next_index += 1;
        // textures.push(load_texture(filename));
        textures[index] = load_texture(filename);
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
        frame_space: &mut [],
        frame_texcoords_size: 0,
        texcoords_space: &mut []
    }
}
