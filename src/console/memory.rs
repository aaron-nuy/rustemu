pub struct Memory {
    ram: Box<[u8; crate::console::constants::MEMORY_SIZE]>,
}