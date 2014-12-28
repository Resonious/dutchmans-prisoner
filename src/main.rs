#![feature(globs, macro_rules)]

extern crate core;
extern crate libc;

extern crate glfw;
extern crate gl;
extern crate cgmath;

// extern crate dutchman_game;

use glfw::{Context, Action, Key};

use std::mem::{uninitialized, transmute, size_of, size_of_val};
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
    // === GLFW stuff: ===
    pub fn glfwGetCurrentContext() -> u64;
    pub fn glfwSetTestIdent(i: int);
    pub fn glfwTestIdent() -> int;
    static _glfw: u8;
}

fn load_game_dylib() -> DynamicLibrary {
    // NOTE this assumes we are running from the project root.
    let game_path     = Path::new("./dutchman-game/dutchman_game.dll");
    let abs_game_path = os::make_absolute(&game_path).unwrap();
    // println!("path: {} abs path: {}", game_path.display(), abs_game_path.display());
    match DynamicLibrary::open(Some(&abs_game_path)) {
        Ok(l) => l,
        Err(e) => panic!("Couldn't load game lib: {}", e)
    }
}

fn test_loop_fn(lib: &DynamicLibrary) -> (LoadFn, TestLoopFn) {
    unsafe {
        let test_loop: TestLoopFn = match lib.symbol::<u8>("update_and_render") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Damn! {}", e)
        };

        let load: LoadFn = match lib.symbol::<u8>("load") {
            Ok(f) => transmute(f),
            Err(e) => panic!(";_;")
        };

        (load, test_loop)
    }
}

fn static_test_loop_fn(lib: &DynamicLibrary) -> () {
    panic!("Hey make this make sense.");
    // dutchman_game::old_test_loop
}

// === Winapi stuff for file listening. ===
#[cfg(target_os = "windows")]
extern "C" {

}

#[cfg(target_os = "windows")]
fn watch_for_updated_dll(tx: &Sender<Box<DynamicLibrary>>) {
    loop {
    }
}

#[cfg(not(target_os = "windows"))]
fn watch_for_updated_dll(tx: Sender<(LoadFn, TestLoopFn)>) {
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

    let mut lib = box load_game_dylib();

    let mut load      = unsafe { uninitialized() };
    let mut test_loop = unsafe { uninitialized() };
    match test_loop_fn(&*lib) {
        (load_fn, test_loop_fn) => {
            load      = load_fn;
            test_loop = test_loop_fn;
        }
    }

    // TODO Stack memory is nice, but might want to box it if it gets too big.
    let mut game_memory = [0u8, ..2048];

    unsafe { load(&_glfw, &window, &mut game_memory[0]); }

    let (tx, rx) = channel();
    spawn(move || watch_for_updated_dll(&tx));

    while !window.should_close() {
        match rx.try_recv() {
            Ok(new_lib) => {
                lib = new_lib;
                match test_loop_fn(&*lib) {
                    (load_fn, test_loop_fn) => {
                        load      = load_fn;
                        test_loop = test_loop_fn;
                    }
                }
                load(&_glfw, &window, &mut game_memory[0]);
            },
            _ => {}
        }

        test_loop(&mut game_memory[0], &glfw, &window, &event);
    }
    println!("Hey, I compiled and ran!");
}
