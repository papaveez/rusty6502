pub struct Bus {
    pub memory: [u8; 0xFFFF],
}

impl Default for Bus {
    fn default() -> Self {
        Bus {
            memory: [0; 0xFFFF],
        }
    }
}

impl Bus {
    pub fn read(&mut self, adr: u16) -> u8 {
        self.memory[adr as usize]
    }

    pub fn write(&mut self, adr: u16, data: u8) {
        self.memory[adr as usize] = data
    }

    pub fn tick(&mut self, cycles: u8) {}
}
