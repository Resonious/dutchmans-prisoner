#![feature(globs, macro_rules)]

extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

use glfw::{Context, Action, Key};

use std::mem::{transmute, size_of, size_of_val};
use gl::types::*;
use libc::c_void;
use std::ptr;
use cgmath::*;
use std::dynamic_lib::DynamicLibrary;
use std::os;
use std::io::fs::PathExtensions;

type GlfwEvent = Receiver<(f64, glfw::WindowEvent)>;

#[cfg(package_name = "dutchmans_prisoner")]
fn test_static() {
    println!("I AM STATIC");
}

#[cfg(not(name = "dutchmans_prisoner"))]
fn test_static() {
    println!("I AM NOTTTTTTTTTTTTTTT STATIC");
}

fn main() {
    test_static();

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (window, event) = glfw
        .create_window(480, 480, "The Flying Dutchman's Prisoner", glfw::WindowMode::Windowed)
        .expect("OH GOD WHY");

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    // NOTE this assumes we are running from the project root.
    let game_path = Path::new("./dutchman-game/target/dutchman_game.dll");
    let abs_game_path = os::make_absolute(&game_path).unwrap();
    // println!("path: {} abs path: {}", game_path.display(), abs_game_path.display());
    let lib = match DynamicLibrary::open(Some(&abs_game_path)) {
        Ok(l) => l,
        Err(e) => panic!("Couldn't load game lib: {}", e)
    };

    unsafe {
        type TestLoopFn = extern "C" fn(&glfw::Glfw, &glfw::Window, &GlfwEvent);
        let test_loop: TestLoopFn = match lib.symbol::<u8>("old_test_loop") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Damn! {}", e)
        };
        println!("ABOUT TO CALL IT, AND THEN...");
        type TestPFn = extern "C" fn();
        let test_p: TestPFn = match lib.symbol::<u8>("update_and_render") {
            Ok(f) => transmute(f),
            Err(e) => panic!(":(")
        };
        test_p();
        test_loop(&glfw, &window, &event);
    }

    // test_loop(&glfw, &window, &event);
    println!("Hey, I compiled and ran!");
}
