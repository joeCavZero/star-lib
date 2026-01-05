use crate::core::*;

pub fn fold_trinity(instruction: StarInstruction, reg1: StarGeneralRegister, reg2: StarGeneralRegister, reg3: StarGeneralRegister) -> u16 {
    let mut format = 0b0000_0000_0000_0000;
    format |= instruction.opcode();
    format |= reg1.code() << 4;
    format |= reg2.code() << 8;
    format |= reg3.code() << 12;
    format
}

pub fn fold_hime(instruction: StarInstruction, reg1: StarGeneralRegister, immediate: u8) -> u16 {
    let mut format = 0b0000_0000_0000_0000;
    format |= instruction.opcode();
    format |= reg1.code() << 4;
    format |= (immediate as u16) << 8;
    format
}

pub fn fold_pair(instruction: StarInstruction, reg1: StarGeneralRegister, reg2: StarGeneralRegister) -> u16 {
    let mut format = 0b0000_0000_0000_0000;
    format |= instruction.opcode();
    format |= reg1.code() << 8;
    format |= reg2.code() << 12;
    format
}

pub fn fold_clover(instruction: StarInstruction, reg: StarGeneralRegister) -> u16 {
    let mut format = 0b0000_0000_0000_0000;
    format |= instruction.opcode();
    format |= reg.code() << 12;
    format
}

pub fn fold_ark(instruction: StarInstruction) -> u16 {
    let mut format = 0b0000_0000_0000_0000;
    format |= instruction.opcode();
    format
}