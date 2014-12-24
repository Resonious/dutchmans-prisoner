extern crate core;
extern crate libc;

extern crate gl;
extern crate cgmath;

use std::mem::{transmute, size_of, size_of_val};

use cgmath::*;
use gl::types::*;

pub struct SpriteData {
    pub position: Vector2<GLfloat>,
    pub frame: GLint
}
