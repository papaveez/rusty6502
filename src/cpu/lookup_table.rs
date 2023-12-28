use crate::cpu::instructions::{instruction_set::*, Addrmode::*, Instr};

pub fn lookup(opcode: u8) -> Instr {
    match opcode {
        0x00 => Instr {
            run: brk,
            mode: Impl,
            cycles: 7,
        },
        _ => panic!("Instruction unresolved!"),
    }
}
