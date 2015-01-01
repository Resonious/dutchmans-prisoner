extern crate core;

extern crate glfw;

use glfw::{Action, Key};

use std::mem::{transmute, size_of, size_of_val};
use std::slice;
use std::slice::IterMut;

pub struct Control {
    pub down: bool,
    pub just_down: bool,
    pub just_up: bool
}

impl Control {
    pub fn check_just_down(&mut self) {
        if self.down && self.just_down {
            self.just_down = false;
        }
        else if !self.down && self.just_up {
            self.just_up = false;
        }
    }

    pub fn process_input(&mut self, action: Action) {
        match action {
            Action::Press => {
                if !self.down {
                    self.down = true;
                    self.just_down = true;
                }
            }

            Action::Release => {
                if self.down {
                    self.just_up = true;
                }
                self.down = false;
            }

            _ => {}
        }
    }
}

pub struct Controls {
    pub up: Control,
    pub down: Control,
    pub left: Control,
    pub right: Control,
    pub debug: Control,
}

impl Controls {
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Control> {
        unsafe {
            let len = size_of::<Controls>() / size_of::<Control>();
            let mut ctrls_slice: &mut [Control] =
                slice::from_raw_mut_buf(transmute(&self), len);
            ctrls_slice.iter_mut()
        }
    }
}
