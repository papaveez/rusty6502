use crate::cpu::CPU;

#[derive(Clone, Copy)]
pub enum Data {
    Immediate(u16),
    Address(u16),
}

impl Data {
    fn default_unwrap(d: Data, cpu: &mut CPU) -> u8 {
        match d {
            Data::Immediate(x) => x as u8,
            Data::Address(x) => cpu.bus.read(x),
        }
    }

    fn address_unwrap(d: Data) -> u16 {
        match d {
            Data::Address(x) => x,
            _ => panic!("Attempt to unwrap address from immediate value !"),
        }
    }

    fn int_unwrap(d: Data, cpu: &mut CPU) -> i8 {
        match d {
            Data::Immediate(x) => x as i8,
            Data::Address(x) => cpu.bus.read(x) as i8,
        }
    }
}

#[derive(Debug)]
pub enum Addrmode {
    A,
    Abs,
    AbsX,
    AbsY,
    Imm,
    Impl,
    Ind,
    XInd,
    IndY,
    Rel, // Integer
    Zpg,
    ZpgX,
    ZpgY,
}

pub fn join_bytes(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | lo as u16
}

fn page_crossed(a1: u16, a2: u16) -> bool {
    a1 & 0xFF00 != a2 & 0xFF00
}

impl Addrmode {
    pub fn unpack(&self, cpu: &mut CPU) -> (Data, bool) {
        use Addrmode::*;
        use Data::*;
        match self {
            Rel => (Immediate(cpu.u8_operand() as u16), false),
            Imm => (Immediate(cpu.u8_operand() as u16), false),
            A => (Immediate(cpu.reg.a as u16), false),
            Abs => (Address(cpu.u16_operand()), false),
            AbsX => {
                let base = cpu.u16_operand();
                let addr = base.wrapping_add(cpu.reg.x as u16);
                (Address(addr), page_crossed(base, addr))
            }
            AbsY => {
                let base = cpu.u16_operand();
                let addr = base.wrapping_add(cpu.reg.y as u16);
                (Address(addr), page_crossed(base, addr))
            }
            Zpg => (Address(cpu.u8_operand() as u16), false),
            ZpgX => (
                Address(cpu.u8_operand().wrapping_add(cpu.reg.x) as u16),
                false,
            ),
            ZpgY => (
                Address(cpu.u8_operand().wrapping_add(cpu.reg.y) as u16),
                false,
            ),
            Ind => (
                {
                    let adr = cpu.u16_operand();
                    Address(join_bytes(
                        cpu.bus.read(adr),
                        cpu.bus.read(adr.wrapping_add(1)),
                    ))
                },
                false,
            ),
            XInd => (
                {
                    let zp_base = cpu.u8_operand();
                    let ptr = zp_base.wrapping_add(cpu.reg.x);
                    let lo = cpu.bus.read(ptr as u16);
                    let hi = cpu.bus.read(ptr.wrapping_add(1) as u16);
                    Address(join_bytes(lo, hi))
                },
                false,
            ),
            IndY => {
                let base = cpu.u8_operand();
                let baseptr = join_bytes(
                    cpu.bus.read(base as u16),
                    cpu.bus.read(base.wrapping_add(1) as u16),
                );
                let new = baseptr.wrapping_add(cpu.reg.y as u16);
                (Address(new), page_crossed(baseptr, new))
            }
            Impl => (Address(0x00), false),
        }
    }
}

pub struct Instr {
    pub run: fn(Data, &mut CPU),
    pub mode: Addrmode,
    pub cycles: u8,
}

pub mod instruction_set {
    use crate::cpu::instructions::Data;
    use crate::cpu::CPU;
    pub fn adc(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);

        let sum: u16 = cpu.reg.a as u16 + w as u16 + if cpu.flags.carry { 1 } else { 0 };
        let result = sum as u8;

        cpu.flags.carry = sum > 0xFF;
        cpu.flags.set_zero_negative(result);
        // cpu.flags.overflow = (cpu.reg.a >> 7) == (w >> 7) && (cpu.reg.a >> 7) != (result >> 7);
        cpu.flags.overflow = (w ^ result) & (cpu.reg.a ^ result) & 0x80 != 0;
        cpu.reg.a = result;
    }

    pub fn sbc(d: Data, cpu: &mut CPU) {
        let q = Data::default_unwrap(d, cpu);
        let w = (q as i8).wrapping_neg().wrapping_sub(1) as u8;

        let sum: u16 = cpu.reg.a as u16 + w as u16 + if cpu.flags.carry { 1 } else { 0 };
        let result = sum as u8;

        cpu.flags.carry = sum > 0xFF;
        cpu.flags.set_zero_negative(result);
        cpu.flags.overflow = (w ^ result) & (cpu.reg.a ^ result) & 0x80 != 0;
        cpu.reg.a = result;
    }

    pub fn inc(d: Data, cpu: &mut CPU) {
        let q = Data::default_unwrap(d, cpu).wrapping_add(1);
        let addr = Data::address_unwrap(d);
        cpu.bus.write(addr, q);
        cpu.flags.set_zero_negative(q);
    }

    pub fn inx(_: Data, cpu: &mut CPU) {
        cpu.reg.x = cpu.reg.x.wrapping_add(1);
        cpu.flags.set_zero_negative(cpu.reg.x);
    }

    pub fn iny(_: Data, cpu: &mut CPU) {
        cpu.reg.y = cpu.reg.y.wrapping_add(1);
        cpu.flags.set_zero_negative(cpu.reg.y);
    }

    pub fn dec(d: Data, cpu: &mut CPU) {
        let q = Data::default_unwrap(d, cpu).wrapping_sub(1);
        let addr = Data::address_unwrap(d);
        cpu.bus.write(addr, q);
        cpu.flags.set_zero_negative(q);
    }

    pub fn dex(_: Data, cpu: &mut CPU) {
        cpu.reg.x = cpu.reg.x.wrapping_sub(1);
        cpu.flags.set_zero_negative(cpu.reg.x);
    }

    pub fn dey(_: Data, cpu: &mut CPU) {
        cpu.reg.y = cpu.reg.y.wrapping_sub(1);
        cpu.flags.set_zero_negative(cpu.reg.y);
    }

    pub fn ldy(d: Data, cpu: &mut CPU) {
        cpu.reg.y = Data::default_unwrap(d, cpu);
        cpu.flags.set_zero_negative(cpu.reg.y);
    }

    pub fn ldx(d: Data, cpu: &mut CPU) {
        cpu.reg.x = Data::default_unwrap(d, cpu);
        cpu.flags.set_zero_negative(cpu.reg.y)
    }

    pub fn lda(d: Data, cpu: &mut CPU) {
        cpu.reg.a = Data::default_unwrap(d, cpu);
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn sta(d: Data, cpu: &mut CPU) {
        cpu.bus.write(Data::address_unwrap(d), cpu.reg.a);
    }

    pub fn stx(d: Data, cpu: &mut CPU) {
        cpu.bus.write(Data::address_unwrap(d), cpu.reg.x);
    }

    pub fn sty(d: Data, cpu: &mut CPU) {
        cpu.bus.write(Data::address_unwrap(d), cpu.reg.y);
    }

    pub fn tax(_: Data, cpu: &mut CPU) {
        // implied
        cpu.reg.x = cpu.reg.a;
        cpu.flags.set_zero_negative(cpu.reg.x)
    }

    pub fn tay(_: Data, cpu: &mut CPU) {
        // implied
        cpu.reg.y = cpu.reg.a;
        cpu.flags.set_zero_negative(cpu.reg.y);
    }

    pub fn tsx(_: Data, cpu: &mut CPU) {
        cpu.reg.x = cpu.reg.sp;
        cpu.flags.set_zero_negative(cpu.reg.x);
    }

    pub fn txa(_: Data, cpu: &mut CPU) {
        cpu.reg.a = cpu.reg.x;
        cpu.flags.set_zero_negative(cpu.reg.a)
    }

    pub fn txs(_: Data, cpu: &mut CPU) {
        cpu.reg.sp = cpu.reg.x;
    }

    pub fn tya(_: Data, cpu: &mut CPU) {
        cpu.reg.a = cpu.reg.y;
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn pha(_: Data, cpu: &mut CPU) {
        cpu.stack_push(cpu.reg.a as u16);
    }

    pub fn php(_: Data, cpu: &mut CPU) {
        let t = u8::from(cpu.flags) | 0b110000;
        cpu.stack_push(t as u16);
    }

    pub fn plp(_: Data, cpu: &mut CPU) {
        let t = cpu.stack_pop() & 0b11001111_u8; // ignore bit 4 and 5;
        cpu.flags = crate::cpu::Flag::from(t);
    }

    pub fn pla(_: Data, cpu: &mut CPU) {
        cpu.reg.a = cpu.stack_pop();
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn and(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.reg.a &= w;
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn eor(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.reg.a ^= w;
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn ora(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.reg.a |= w;
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn asl(d: Data, cpu: &mut CPU) {
        let mut w = Data::default_unwrap(d, cpu) as u16;
        w <<= 1;
        cpu.flags.carry = w >= 0xFF;
        cpu.reg.a = w as u8;
        cpu.flags.set_zero_negative(cpu.reg.a);
    }

    pub fn lsr(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.flags.carry = w & 1 == 1;
        cpu.reg.a = w >> 1;
        cpu.flags.set_zero_negative(w);
    }

    pub fn rol(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        let c = cpu.flags.carry;
        cpu.flags.carry = w >> 7 == 1;
        let mut q = w << 1;

        if c {
            q |= 1;
        }

        cpu.reg.a = q;
        cpu.flags.set_zero_negative(q);
    }

    pub fn ror(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        let c = cpu.flags.carry;
        cpu.flags.carry = w & 1 == 1;
        let mut q = w >> 1;

        if c {
            q |= 0x80;
        }

        cpu.reg.a = q;
        cpu.flags.set_zero_negative(q);
    }

    pub fn clc(_: Data, cpu: &mut CPU) {
        cpu.flags.carry = false;
    }

    pub fn cld(_: Data, cpu: &mut CPU) {
        cpu.flags.decimal = false;
    }

    pub fn cli(_: Data, cpu: &mut CPU) {
        cpu.flags.interrupt_disable = false;
    }

    pub fn clv(_: Data, cpu: &mut CPU) {
        cpu.flags.overflow = false;
    }

    pub fn sec(_: Data, cpu: &mut CPU) {
        cpu.flags.carry = true;
    }

    pub fn sed(_: Data, cpu: &mut CPU) {
        cpu.flags.decimal = true;
    }

    pub fn sei(_: Data, cpu: &mut CPU) {
        cpu.flags.interrupt_disable = true;
    }

    pub fn cmp(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.flags.zero = w == cpu.reg.a;
        cpu.flags.carry = cpu.reg.a >= w;
        cpu.flags.negative = cpu.reg.a.wrapping_sub(w) >> 7 == 1;
    }

    pub fn cpx(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.flags.zero = w == cpu.reg.x;
        cpu.flags.carry = cpu.reg.x >= w;
        cpu.flags.negative = cpu.reg.x.wrapping_sub(w) >> 7 == 1;
    }

    pub fn cpy(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.flags.zero = w == cpu.reg.y;
        cpu.flags.carry = cpu.reg.y >= w;
        cpu.flags.negative = cpu.reg.y.wrapping_sub(w) >> 7 == 1;
    }

    pub fn bcc(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, !cpu.flags.carry);
    }

    pub fn bcs(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, cpu.flags.carry);
    }

    pub fn beq(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, cpu.flags.zero);
    }

    pub fn bmi(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, cpu.flags.negative);
    }

    pub fn bne(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, !cpu.flags.zero);
    }

    pub fn bpl(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, !cpu.flags.negative);
    }

    pub fn bvc(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, !cpu.flags.overflow);
    }

    pub fn bvs(d: Data, cpu: &mut CPU) {
        let i = Data::int_unwrap(d, cpu);
        cpu.branch(i, cpu.flags.overflow);
    }

    // Jumps
    pub fn jmp(d: Data, cpu: &mut CPU) {
        cpu.pc = Data::address_unwrap(d).wrapping_sub(1);
    }

    pub fn jsr(d: Data, cpu: &mut CPU) {
        cpu.stack_push(cpu.pc.wrapping_add(1));
        cpu.pc = Data::address_unwrap(d).wrapping_sub(1);
    }

    pub fn rts(_: Data, cpu: &mut CPU) {
        cpu.pc = cpu.stack_pop16().wrapping_sub(1);
    }

    pub fn brk(_: Data, cpu: &mut CPU) {
        cpu.halted = true;
    }

    pub fn rti(_: Data, _cpu: &mut CPU) {
        // do nothing for now
    }

    pub fn bit(d: Data, cpu: &mut CPU) {
        let w = Data::default_unwrap(d, cpu);
        cpu.flags.zero = cpu.reg.a & w == 0;
        cpu.flags.negative = w & 0x80 > 0;
        cpu.flags.overflow = w & 0x40 > 0;
    }

    pub fn nop(_: Data, _cpu: &mut CPU) {}
}
