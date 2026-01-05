
use std::mem::transmute;

use crate::core::*;
use crate::generateable::*;
use crate::math::split_u16_to_strings;
use crate::math::u16_from_string;
use crate::math::u8_from_string;
use crate::parseable::*;

pub trait StarGenerateable {
    fn generate(&mut self, ast: &Ast) -> Result<usize, (String, StarPosition)>;
    fn generate_data_memory(&mut self, ast: &Ast) -> Result<usize, (String, StarPosition)>;
    fn generate_instruction_memory(&mut self, ast: &Ast) -> Option<(String, StarPosition)>;
}

impl StarGenerateable for Star {
    fn generate(&mut self, ast: &Ast) -> Result<usize, (String, StarPosition)> {
        match self.generate_data_memory(ast) {
            Ok(data_section_size) => {
                self.generate_instruction_memory(ast);
                return Ok(data_section_size);
            }
            Err(e) => return Err(e),
        }
        
    }

    fn generate_data_memory(&mut self, ast: &Ast) -> Result<usize, (String, StarPosition)> {
        let mut is_data_section_empty = true;
        let mut data_memory_pointer: usize = 0;
        for data_camp in ast.data_field.iter() {
            is_data_section_empty = false;
            match data_camp.directive.token {
                StarToken::StarDirective(StarDirective::Byte)
                | StarToken::StarDirective(StarDirective::Word)
                => {

                    if let DataCampArg::Multiple(values) = &data_camp.arg {
                        for val_ptk in values.iter() {
                            if let StarToken::NumberLiteral(num_string) = &val_ptk.token {
                                match data_camp.directive.token {
                                    StarToken::StarDirective(StarDirective::Byte) => {
                                        match u8_from_string((*num_string).to_string()) {
                                            Ok(num) => {
                                                match self.data_memory.get_mut(data_memory_pointer) {
                                                    Some(byte) => {
                                                        *byte = num;
                                                        data_memory_pointer += 1;
                                                    }
                                                    None => {
                                                        return Err((
                                                            "Data memory overflow".to_string(),
                                                            val_ptk.position,
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                return Err((
                                                    e.to_string(),
                                                    val_ptk.position,
                                                ));
                                            }
                                        }
                                    }
                                    StarToken::StarDirective(StarDirective::Word) => {
                                        match u16_from_string((*num_string).to_string()) {
                                            Ok(num) => {
                                                let (low, high) = split_u16_to_strings(num);

                                                if let Some(byte1) = self.data_memory.get_mut(data_memory_pointer) {
                                                    *byte1 = u8_from_string(high).unwrap_or(0);
                                                } else {
                                                    return Err((
                                                        "Data memory overflow".to_string(),
                                                        val_ptk.position,
                                                    ));
                                                }

                                                data_memory_pointer += 1;

                                                if let Some(byte2) = self.data_memory.get_mut(data_memory_pointer) {
                                                    *byte2 = u8_from_string(low).unwrap_or(0);
                                                } else {
                                                    return Err((
                                                        "Data memory overflow".to_string(),
                                                        val_ptk.position,
                                                    ));
                                                }

                                                data_memory_pointer += 1;
                                            }
                                            Err(e) => {
                                                return Err((
                                                    e,
                                                    val_ptk.position,
                                                ));
                                            }
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            } 
                        }
                    } else {
                        unreachable!();
                    }
                }
                StarToken::StarDirective(StarDirective::Space) => {
                    if let DataCampArg::Unique(value) = data_camp.arg.clone() {
                        if let StarToken::NumberLiteral(num_string) = value.token {
                            match u16_from_string((*num_string).to_string()) {
                                Ok(num) => {
                                    data_memory_pointer += num as usize;
                                }
                                Err(e) => {
                                    return Err((
                                        e,
                                        value.position,
                                    ));
                                }
                            }
                        } else {
                            unreachable!();
                        }
                    } else {
                        unreachable!();
                    }
                }
                StarToken::StarDirective(StarDirective::String)
                | StarToken::StarDirective(StarDirective::Stringz) => {
                    if let DataCampArg::Unique(value) = data_camp.arg.clone() {
                        if let StarToken::StringLiteral(string) = value.token {
                            let string_bytes = string.as_bytes();
                            for &byte in string_bytes.iter() {
                                if let Some(data_byte) = self.data_memory.get_mut(data_memory_pointer) {
                                    *data_byte = byte;
                                    data_memory_pointer += 1;
                                } else {
                                    return Err((
                                        "Data memory overflow".to_string(),
                                        value.position,
                                    ));
                                }
                            }

                            // If it's a null-terminated string, add a null byte
                            if data_camp.directive.token == StarToken::StarDirective(StarDirective::Stringz) {
                                if let Some(data_byte) = self.data_memory.get_mut(data_memory_pointer) {
                                    *data_byte = 0; // Null terminator
                                    data_memory_pointer += 1;
                                } else {
                                    return Err((
                                        "Data memory overflow".to_string(),
                                        value.position,
                                    ));
                                }
                            }
                        } else {
                            unreachable!();
                        }
                    } else {
                        unreachable!();
                    }
                }
                StarToken::StarDirective(StarDirective::Checkpoint) => {
                    if let DataCampArg::Empty = data_camp.arg {
                        // Nothing to do here, just a checkpoint
                    } else {
                        return Err((
                            "Checkpoint directive does not accept arguments".to_string(),
                            data_camp.directive.position,
                        ));
                    }
                }
                _ => unreachable!(),
            }
        }
    
        if is_data_section_empty {
            return Ok(0);
        } else {
            return Ok(data_memory_pointer);
        }
    }
    fn generate_instruction_memory(&mut self, ast: &Ast) -> Option<(String, StarPosition)> {
        for instr_camp in ast.instr_field.iter() {
            if let StarToken::StarInstruction(instruction) = instr_camp.instruction.token.clone() {
                match instruction.format() {
                    StarFormat::Trinity => {
                        if let StarSequence::Three(reg_ptk_1, reg_ptk_2, reg_ptk_3) = instr_camp.sequence.clone() {
                            if let (StarToken::StarGeneralRegister(reg1), StarToken::StarGeneralRegister(reg2), StarToken::StarGeneralRegister(reg3)) = (reg_ptk_1.token.clone(), reg_ptk_2.token.clone(), reg_ptk_3.token.clone()) {
                                let format: u16 = fold_trinity(
                                    instruction,
                                    reg1,
                                    reg2,
                                    reg3,
                                );

                                self.position_memory.push(instr_camp.instruction.position);

                                let (instr_low, instr_high) = unsafe { transmute::<u16, (u8, u8)>(format) };
                                self.instruction_memory.push(instr_high);
                                self.instruction_memory.push(instr_low);
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    StarFormat::Hime => {
                        if let StarSequence::Two(reg_ptk, imm_ptk) = instr_camp.sequence.clone() {
                            if let (StarToken::StarGeneralRegister(reg), StarToken::NumberLiteral(imm_string)) = (reg_ptk.token.clone(), imm_ptk.token.clone()) {
                                match u8_from_string(imm_string) {
                                    Ok(imm) => {
                                        let format: u16 = fold_hime(
                                            instruction,
                                            reg,
                                            imm,
                                        );
                                        
                                        self.position_memory.push(instr_camp.instruction.position);

                                        let (instr_low, instr_high) = unsafe { transmute::<u16, (u8, u8)>(format) };
                                        self.instruction_memory.push(instr_high);
                                        self.instruction_memory.push(instr_low);
                                    }
                                    Err(e) => {
                                        return Some((
                                            e,
                                            imm_ptk.position,
                                        ));
                                    }
                                }
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    StarFormat::Pair => {
                        if let StarSequence::Two(reg_ptk_1, reg_ptk_2) = instr_camp.sequence.clone() {
                            if let (StarToken::StarGeneralRegister(reg1), StarToken::StarGeneralRegister(reg2)) = (reg_ptk_1.token.clone(), reg_ptk_2.token.clone()) {
                                let format: u16 = fold_pair(
                                    instruction,
                                    reg1,
                                    reg2,
                                );
                                
                                self.position_memory.push(instr_camp.instruction.position);

                                let (instr_low, instr_high) = unsafe { transmute::<u16, (u8, u8)>(format) };
                                self.instruction_memory.push(instr_high);
                                self.instruction_memory.push(instr_low);
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    StarFormat::Clover => {
                        if let StarSequence::One(reg_ptk) = instr_camp.sequence.clone() {
                            if let StarToken::StarGeneralRegister(reg) = reg_ptk.token.clone() {
                                let format: u16 = fold_clover(
                                    instruction,
                                    reg,
                                );
                                
                                self.position_memory.push(instr_camp.instruction.position);

                                let (instr_low, instr_high) = unsafe { transmute::<u16, (u8, u8)>(format) };
                                self.instruction_memory.push(instr_high);
                                self.instruction_memory.push(instr_low);
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    StarFormat::Ark => {
                        if let StarSequence::Zero = instr_camp.sequence.clone() {
                            let format: u16 = fold_ark(instruction);
                            
                            self.position_memory.push(instr_camp.instruction.position);

                            let (instr_low, instr_high) = unsafe { transmute::<u16, (u8, u8)>(format) };
                            self.instruction_memory.push(instr_high);
                            self.instruction_memory.push(instr_low);
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }
        return None;
    }
}