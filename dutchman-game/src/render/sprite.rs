extern crate core;
extern crate libc;

extern crate gl;
extern crate cgmath;

use std::mem::{transmute, size_of};

use render::shader;
use cgmath::*;
use gl::types::*;

#[deriving(Copy)]
#[allow(missing_copy_implementations)]
pub struct SpriteData {
    pub position: Vector2<GLfloat>,
    pub frame: GLint,
    pub flipped: GLint
}

#[allow(missing_copy_implementations)]
pub struct Sprite {
    pub vbo: GLuint,
    pub buffer_index: uint
}

impl Sprite {
    // TODO THIS IS NOT EVEN USED. SPRITE IS NOT EVEN USED.
    fn set(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            let size_of_sprite = size_of::<SpriteData>() as GLint;
            assert_eq!(size_of_sprite, 16);

            // == Position ==
            gl::EnableVertexAttribArray(shader::ATTR_POSITION);
            gl::VertexAttribPointer(
                shader::ATTR_POSITION, 2, gl::FLOAT, gl::FALSE as GLboolean,
                size_of_sprite, transmute(0i64)
            );
            gl::VertexAttribDivisor(shader::ATTR_POSITION, 1);
            let offset = 2 * size_of::<GLfloat>() as i64;
            assert_eq!(offset, 8);

            // == Frame ==
            gl::EnableVertexAttribArray(shader::ATTR_FRAME);
            gl::VertexAttribIPointer(
                shader::ATTR_FRAME, 1, gl::INT,
                size_of_sprite, transmute(offset)
            );
            gl::VertexAttribDivisor(shader::ATTR_FRAME, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}
