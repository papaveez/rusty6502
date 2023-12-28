pub mod instructions;
pub mod lookup_table;
pub mod registers;

use crate::bus::Bus;
use registers::{Flag, Registers};

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

    pub fn reset(&mut self) {
        self.reg.a = 0;
        self.reg.x = 0;
        self.reg.y = 0;
        self.reg.sp = 0xfd;
        self.flags = Flag::from(0b100100_u8);
        self.pc = self.bus.read(0xFFFC) as u16 | (self.bus.read(0xFFFD) as u16) << 8;
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

    pub fn exec(&mut self) {
        let opcode = self.bus.read(self.pc);
        let i = lookup_table::lookup(opcode);
        let (unpakt, pagecross) = i.mode.unpack(self);
        if pagecross {
            self.bus.tick(1);
        }
        (i.run)(unpakt, self);
        self.pc = self.pc.wrapping_add(1);
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
        if c {
            self.bus.tick(1);

            self.pc = self.pc.wrapping_add(1);

            let addr = self.pc.wrapping_add(w as u16);
            if addr & 0xFF00 != self.pc & 0xFF00 {
                self.bus.tick(1);
            }

            self.pc = addr;
        }
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
        pu.bus.write(0x00_u16, 0x01);
        pu.exec();
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
