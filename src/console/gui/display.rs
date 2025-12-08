use crate::console::constants::*;

pub struct Display {
    pub buffer: Box<[u32; BUFFER_SIZE]>,
}

impl Display {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0; BUFFER_SIZE]),
        }
    }
}