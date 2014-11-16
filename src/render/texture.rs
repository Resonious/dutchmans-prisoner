extern crate native;
extern crate core;
extern crate image;
extern crate gl;

use self::image::GenericImage;
use std::io::File;
use gl::types::*;

use asset;

pub fn load_texture(filename: &str) -> GLuint {
	let image = image::open(&asset::path(filename)).unwrap();
	println!("dimensions of {}: {}", filename, image.dimensions());

	1
}
