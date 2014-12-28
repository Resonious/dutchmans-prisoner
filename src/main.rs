#![feature(globs, macro_rules)]

extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

// extern crate dutchman_game;

use glfw::{Context, Action, Key};

use std::mem::{transmute, size_of, size_of_val};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;
use std::dynamic_lib::DynamicLibrary;
use std::os;
use std::io::fs::PathExtensions;
use std::mem;

type GlfwEvent = Receiver<(f64, glfw::WindowEvent)>;
type TestLoopFn = extern "C" fn(&mut u8, &glfw::Glfw, &glfw::Window, &GlfwEvent);
type LoadFn = extern "C" fn(&u8, &glfw::Window, &mut u8);

extern "C" {
    pub fn glfwGetCurrentContext() -> u64;
    pub fn glfwSetTestIdent(i: int);
    pub fn glfwTestIdent() -> int;
    static _glfw: u8;
}

fn test_loop_fn() -> (LoadFn, TestLoopFn) {
    // NOTE this assumes we are running from the project root.
    let game_path = Path::new("./dutchman-game/dutchman_game.dll");
    let abs_game_path = os::make_absolute(&game_path).unwrap();
    // println!("path: {} abs path: {}", game_path.display(), abs_game_path.display());
    let lib = match DynamicLibrary::open(Some(&abs_game_path)) {
        Ok(l) => l,
        Err(e) => panic!("Couldn't load game lib: {}", e)
    };

    unsafe {
        let test_loop: TestLoopFn = match lib.symbol::<u8>("update_and_render") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Damn! {}", e)
        };

        let load: LoadFn = match lib.symbol::<u8>("load") {
            Ok(f) => transmute(f),
            Err(e) => panic!(";_;")
        };

        // TODO no
        mem::forget(lib);
        (load, test_loop)
    }
}

fn static_test_loop_fn() -> () {
    panic!("Hey make this make sense.");
    // dutchman_game::old_test_loop
}

fn main() {
    // test_static();

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (window, event) = glfw
        .create_window(480, 480, "The Flying Dutchman's Prisoner", glfw::WindowMode::Windowed)
        .expect("OH GOD WHY");

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    unsafe {
        glfwSetTestIdent(99);
        println!(".exe: Test ident: {}", glfwTestIdent());
        println!(".exe: glfwGetCurrentContext(): {}", glfwGetCurrentContext());
    }

    let (load, test_loop) = test_loop_fn();
    // TODO Stack memory is nice, but might want to box it if it gets too big.
    let mut game_memory = [0u8, ..1024];

    unsafe { load(&_glfw, &window, &mut game_memory[0]); }
    while !window.should_close() {
        test_loop(&mut game_memory[0], &glfw, &window, &event);
    }
    println!("Hey, I compiled and ran!");
}
