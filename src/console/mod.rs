pub mod cartridge;
pub mod cpu;
pub mod memory;
pub mod display;
pub mod audio;
pub mod input;
pub mod constants;

pub use cartridge::Cartridge;
pub use cpu::Cpu;
pub use memory::Memory;
pub use display::Display;
pub use audio::Audio;

struct Console {
    cpu: Cpu,
    memory: Memory,
    cartridge: Cartridge,
    display: Display,
    audio: Audio
}