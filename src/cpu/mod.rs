pub mod instructions;
pub mod lookup_table;
pub mod registers;

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

use crate::bus::Bus;
use registers::{Flag, Registers};

fn uint_to_string_literal<T: std::fmt::Display + std::fmt::LowerHex + std::fmt::UpperHex>(
    value: T,
) -> &'static str {
    Box::leak(Box::new(format!("{:0002X}", value)))
}

fn append_to_file(file_path: &str, content: &str) -> Result<(), io::Error> {
    // Open the file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // Append the content followed by a newline to the file
    writeln!(file, "{}", content)?;

    Ok(())
}

pub struct CPU {
    pub bus: Bus,
    pub pc: u16,
    pub flags: Flag,
    pub reg: Registers,
    pub halted: bool,
    pub stack_loc: u16,
}

impl CPU {
    pub fn new(b: Bus) -> Self {
        CPU {
            bus: b,
            pc: 0,
            flags: Flag::from(0b100100_u8),
            reg: Registers {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xfd,
            },
            halted: false,
            stack_loc: 0x100,
        }
    }

    pub fn run<F: FnMut(&mut CPU)>(&mut self, mut callback: F) {
        while !self.halted {
            self.exec();
            callback(self);
        }
    }

    pub fn load_rom_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        self.load(buffer);
        Ok(())
    }

    pub fn load(&mut self, data: Vec<u8>) {
        self.bus.memory[0x0600..(0x0600 + data.len())].copy_from_slice(&data[..]);
        self.bus.write(0xFFFC, 0x00);
        self.bus.write(0xFFFD, 0x06);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.reg.a = 0;
        self.reg.x = 0;
        self.reg.y = 0;
        self.reg.sp = 0xfd;
        self.flags = Flag::from(0b100100_u8);
        self.pc = self.bus.read(0xFFFC) as u16 | ((self.bus.read(0xFFFD) as u16) << 8);
    }

    pub fn exec(&mut self) {
        let opcode = self.bus.read(self.pc);
        let i = lookup_table::lookup(opcode);

        match append_to_file(
            "./log.txt",
            &(uint_to_string_literal(self.pc).to_string() + "|" + uint_to_string_literal(opcode)),
        ) {
            Ok(_) => (),
            Err(e) => panic!("Error: {}", e),
        };

        let (unpakt, pagecross) = i.mode.unpack(self);
        if pagecross {
            self.bus.tick(1);
        }

        (i.run)(unpakt, self);
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn stack_push(&mut self, data: u16) {
        if data > 0xFF {
            let lo = data & 0xFF;
            let hi = (data & 0xFF00) >> 8;
            self.stack_push(hi);
            self.stack_push(lo);
            return;
        }
        self.bus
            .write(self.stack_loc + self.reg.sp as u16, data as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.bus.read(self.reg.sp as u16 | self.stack_loc)
    }

    pub fn stack_pop16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        lo | (hi << 8)
    }

    pub fn u8_operand(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        self.bus.read(self.pc)
    }

    pub fn i8_operand(&mut self) -> i8 {
        self.pc = self.pc.wrapping_add(1);
        self.bus.read(self.pc) as i8
    }

    pub fn u16_operand(&mut self) -> u16 {
        self.pc = self.pc.wrapping_add(1);
        let lo = self.bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi = self.bus.read(self.pc) as u16;

        (hi << 8) | lo
    }

    pub fn branch(&mut self, w: i8, c: bool) {
        if !c {
            return;
        };

        self.bus.tick(1);

        let addr = self.pc.wrapping_add(w as u16);
        if addr & 0xFF00 != self.pc & 0xFF00 {
            self.bus.tick(1);
        }

        self.pc = addr;
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::Flag;
    use crate::cpu::*;

    #[test]
    fn initialise_cpu() {
        let b = Bus { memory: [0; 65535] };
        let mut pu = CPU::new(b);
        let game_code = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9,
            0x02, 0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85,
            0x12, 0xa9, 0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60,
            0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60,
            0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9,
            0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04,
            0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04,
            0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d,
            0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60,
            0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09,
            0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6,
            0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28,
            0x60, 0xe6, 0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69,
            0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c,
            0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10,
            0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa6, 0xff, 0xea, 0xea, 0xca, 0xd0, 0xfb,
            0x60,
        ];

        pu.load(game_code);
        pu.reset();
    }

    #[test]
    fn get_flag() {
        let mut flag = Flag::default();
        flag.reset();
        // flag.zero = true;
        flag.negative = true;
        flag.b = true;

        println!("FLAG, {:b}", u8::from(flag));

        let q = u8::from(flag);
        let mut w = Flag::from(q);
        w.zero = false;

        assert_eq!(q, u8::from(w));
    }
}
