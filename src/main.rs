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
use libc::{c_void, c_char};
use std::ptr;
use cgmath::*;
use std::dynamic_lib::DynamicLibrary;
use std::os;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::mem;
use std::str;
use std::io::timer::sleep;
use std::time::duration::Duration;
use std::num::Float;

type GlfwEvent = Receiver<(f64, glfw::WindowEvent)>;
type TestLoopFn = extern "C" fn(&mut u8, &mut u8, &Duration, &glfw::Glfw, &glfw::Window, &GlfwEvent);
type LoadFn = extern "C" fn(&u8, &glfw::Window, &mut u8, &mut u8);

static DYLIB_DIR: &'static str = "./dutchman-game";

extern "C" {
    // === GLFW stuff: ===
    pub fn glfwGetCurrentContext() -> u64;
    pub fn glfwSetTestIdent(i: int);
    pub fn glfwTestIdent() -> int;
    static _glfw: u8;
}

fn game_dylib_path() -> Option<Path> {
    // NOTE this assumes we are running from the project root.
    // let game_path = Path::new("./dutchman-game/dutchman_game.dll");
    // os::make_absolute(&game_path).unwrap()

    let dir = Path::new(DYLIB_DIR);
    let contents = fs::readdir(&dir).unwrap();
    for entry in contents.iter() {
        if entry.is_dir() { continue; }
        let file_name = entry.filename_str().unwrap();

        if file_name.contains("dutchman_game") && file_name.contains(".dll") {
            return Some(os::make_absolute(entry).unwrap());
        }
    }
    println!("WARNING: Failed to find game dylib path!");
    return None;
}

fn load_game_dylib_from(path: &Path) -> DynamicLibrary {
    match DynamicLibrary::open(Some(path)) {
        Ok(l) => l,
        Err(e) => panic!("Couldn't load game lib: {}", e)
    }
}

fn load_game_dylib() -> DynamicLibrary {
    match DynamicLibrary::open(game_dylib_path()) {
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
    pub fn FindFirstChangeNotificationA(path:          *const c_char,
                                       watch_subtree: bool,
                                       filter:        int)
                                        -> *const c_void;
    pub fn FindNextChangeNotification(handle: *const c_void) -> bool;
    pub fn WaitForSingleObject(handle:     *const c_void,
                               timeout_ms: int)
                                -> int;
    pub fn GetLastError() -> int;
}
static INFINITE: int = 0xFFFFFFFF;
static FILE_NOTIFY_CHANGE_LAST_WRITE: int = 0x00000010;
static INVALID_HANDLE_VALUE: *const c_void = -1 as *const c_void;

#[cfg(target_os = "windows")]
fn watch_for_updated_dll(tx: &Sender<(Path, DynamicLibrary, LoadFn, TestLoopFn)>) {
    // current dylib filename
    let mut current_dylib_path = game_dylib_path().unwrap();
    let dylib_dir = current_dylib_path.dir_path();

    unsafe {
        let handle = dylib_dir.with_c_str(|s|
            FindFirstChangeNotificationA(s, false, FILE_NOTIFY_CHANGE_LAST_WRITE)
        );

        if handle == INVALID_HANDLE_VALUE {
            panic!("Failed to acquire file change notification handle: {}", GetLastError());
        }

        loop {
            match WaitForSingleObject(handle, INFINITE) {
                // File was changed (or created)
                0x00000000 => {
                    let files = fs::readdir(&dylib_dir).unwrap();
                    for file in files.iter() {
                        if file.is_dir() { continue; }
                        let file_name = file.filename_str().unwrap();

                        if file_name.contains("dutchman_game") &&
                           file_name.contains(".dll") &&
                           file_name != current_dylib_path.filename_str().unwrap()
                        {
                            println!("New game dylib path: {}\nCurrent game dylib path: {}",
                                     file.display(), current_dylib_path.display());
                            println!("-----------");

                            let lib = load_game_dylib_from(file);
                            match test_loop_fn(&lib) {
                                (load_fn, test_loop_fn) => {
                                    tx.send((file.clone(), lib, load_fn, test_loop_fn));
                                }
                            }

                            current_dylib_path = file.clone();
                            break;
                        }
                    }
                }

                0xFFFFFFFF =>
                    panic!("Error occurred during directory wait! {}", GetLastError()),

                _ => println!("Something happened but don't care.")
            }

            if !FindNextChangeNotification(handle) {
                panic!("Couldn't rewatch directory! {}", GetLastError());
            }
        }
        println!("No longer watching for DLL updates!");
    }
}

#[cfg(not(target_os = "windows"))]
fn watch_for_updated_dll(tx: Sender<(LoadFn, TestLoopFn)>) {
}

#[link(name = "Winmm")]
extern "C" {
    // NOTE needed for accurate frame delay on Windows only.
    pub fn timeBeginPeriod(period: uint) -> uint;
    pub fn timeEndPeriod(period: uint);
}
static TIMERR_NOCANDO: uint = 97;

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

    let mut current_dylib_path = game_dylib_path().unwrap();
    let mut lib = load_game_dylib_from(&current_dylib_path);

    let mut load      = unsafe { uninitialized() };
    let mut test_loop = unsafe { uninitialized() };
    match test_loop_fn(&lib) {
        (load_fn, test_loop_fn) => {
            load      = load_fn;
            test_loop = test_loop_fn;
        }
    }

    // TODO Stack memory is nice, but might want to box it if it gets too big.
    let mut game_memory    = [0u8, ..4096];
    let mut options_memory = [0u8, ..128];
    // let mut game_memory = box [0u8, ..2048 * 1024];

    unsafe { load(&_glfw, &window, &mut game_memory[0], &mut options_memory[0]); }

    let (tx, rx) = channel();
    spawn(move || watch_for_updated_dll(&tx));

    let target_frame_time = Duration::nanoseconds((1.0e9 / 60.0 as f64).floor() as i64);
    // let delta_frame_time = target_frame_time + Duration::microseconds(400);
    let mut delta = target_frame_time.clone();
    let should_sleep = unsafe { timeBeginPeriod(1) != TIMERR_NOCANDO };

    while !window.should_close() {
        let time = Duration::span(|| {
            match rx.try_recv() {
                Ok((new_lib_path, new_lib, new_load, new_test_loop)) => {
                    lib       = new_lib;
                    load      = new_load;
                    test_loop = new_test_loop;

                    loop {
                        match fs::unlink(&current_dylib_path) {
                            Err(e) => continue,
                            _ => break
                        }
                    }
                    current_dylib_path = new_lib_path;

                    load(&_glfw, &window, &mut game_memory[0], &mut options_memory[0]);
                },
                _ => {}
            }

            test_loop(&mut game_memory[0], &mut options_memory[0], &delta, &glfw, &window, &event);
        });

        if time > target_frame_time {
            delta = time;
        }// else {
         //   delta = time + Duration::span(|| {
         //       if should_sleep {
         //           sleep(target_frame_time - time);
         //       }
         //   });
        // }
    }

    if should_sleep {
        unsafe { timeEndPeriod(1); }
    }
    println!("Hey, I compiled and ran!");
}
