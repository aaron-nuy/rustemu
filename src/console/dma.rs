#[derive(Default)]
pub struct DMAData {
    pub running: bool,
    pub start_addr: u16,
    pub current_addr: u16,
    pub dot_cycle_since_start: u16,
}

impl DMAData {
    pub fn init_transfer_data(&mut self, start_addr: u16) {
        self.start_addr = start_addr;
        self.current_addr = start_addr;
        self.dot_cycle_since_start = 0;
        self.running = true;
    }
}
