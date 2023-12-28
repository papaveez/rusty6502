fn bool_u8(b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}

#[derive(Clone, Copy, Default)]
pub struct Flag {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal: bool,
    pub b: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl Flag {
    pub fn reset(&mut self) {
        self.carry = false; // C | 0
        self.zero = false; // Z | 1
        self.interrupt_disable = false; // I | 2
        self.decimal = false; // D | 3
        self.b = false; // B | 4
        self.overflow = false; // V | 6
        self.negative = false; // N | 7
    }
}

impl Flag {
    pub fn set_zero_negative(&mut self, i: u8) {
        self.zero = i == 0;
        self.negative = i & 0x80 != 0;
    }
}

impl std::convert::From<Flag> for u8 {
    fn from(f: Flag) -> u8 {
        bool_u8(f.carry)
            | bool_u8(f.zero) << 1
            | bool_u8(f.interrupt_disable) << 2
            | bool_u8(f.decimal) << 3
            | bool_u8(f.b) << 4
            | 1 << 5
            | bool_u8(f.overflow) << 6
            | bool_u8(f.negative) << 7
    }
}

impl std::convert::From<u8> for Flag {
    fn from(b: u8) -> Flag {
        Flag {
            carry: (1 & b) > 0,
            zero: (1 << 1 & b) > 0,
            interrupt_disable: (1 << 2 & b) > 0,
            decimal: (1 << 3 & b) > 0,
            b: (1 << 4 & b) > 0,
            overflow: (1 << 6 & b) > 0,
            negative: (1 << 7 & b) > 0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
}
