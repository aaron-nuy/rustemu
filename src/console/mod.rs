pub mod cartridge;
pub mod memory;
pub mod display;
pub mod audio;
pub mod input;
pub mod constants;
pub mod instruction;
pub mod register;
pub mod bit_utils;
pub mod instruction_operands;
pub mod cpu;

pub use cartridge::Cartridge;
pub use cpu::Cpu;
pub use memory::Memory;
pub use display::Display;
pub use audio::Audio;

struct Console<'a> {
    cpu: Cpu<'a>,
    memory: Memory,
    cartridge: Cartridge,
    display: Display,
    audio: Audio
}