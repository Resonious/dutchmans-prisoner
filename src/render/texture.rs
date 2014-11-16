extern crate native;
extern crate core;
extern crate lodepng;
extern crate gl;
extern crate libc;

use gl::types::*;
use std::mem::transmute;

use asset;

pub fn load_texture(filename: &str) -> GLuint {
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

    tex_id
}
