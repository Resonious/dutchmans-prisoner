#![feature(globs, macro_rules)]
extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;

use glfw::Context;

use render::shader;
use render::texture;

pub mod render;
pub mod asset;

fn main() {
	let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

	let (window, _event) = glfw
		.create_window(600, 480, "Crattle Crute maybe?", glfw::Windowed)
		.expect("Failed to create window!");

	window.set_key_polling(true);
	window.set_size_polling(true);
	window.make_current();

	gl::load_with(|s| window.get_proc_address(s));

	let prog = shader::create_program(shader::STANDARD_VERTEX, shader::STANDARD_FRAGMENT);
	println!("Hello, world! Here is the program id: {}", prog);

	let path = asset::path("basicshading.png");
	println!("here it is: {}", path.display());

	let _test = texture::load_texture("basicshading.png");
}
