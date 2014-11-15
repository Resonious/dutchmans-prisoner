#![feature(globs, macro_rules)]
extern crate native;
extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;

use glfw::Context;

use render::shader;
mod render { pub mod shader; }


fn main() {
	let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

	let vertex_shader_src = "
		#version 330 core

		layout (location = 0) in vec2 position;

		void main()
		{
			gl_Position = vec4(position, 0.0f, 1.0f);
		}
	";

	let fragment_shader_src = "
		#version 330 core

		out vec4 color;

		void main()
		{
			color = vec4(1.0f, 0.5f, 0.2f, 1.0f);
		}
	";

	let (window, _event) = glfw
		.create_window(600, 480, "Crattle Crute maybe?", glfw::Windowed)
		.expect("Failed to create window!");

	window.set_key_polling(true);
	window.set_size_polling(true);
	window.make_current();

	gl::load_with(|s| window.get_proc_address(s));

	let prog = shader::create_program(vertex_shader_src, fragment_shader_src);

	println!("Hello, world! Here is the program id: {}", prog)
}
