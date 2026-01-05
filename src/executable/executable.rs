use std::io;
use std::io::Write;

use std::mem::transmute;

use crate::generateable::*;
use crate::math::*;

use crate::core::*;

pub trait StarExecutable {
    fn execute(&mut self) -> Option<(String, Option<StarPosition>)>;
}

/// Manipula operações de leitura de string da entrada padrão e armazena na memória de dados.
///
/// - Caso `self.registers.aux1 == 14`:
///   - Lê uma linha da entrada padrão (stdin), remove espaços em branco das extremidades e armazena os caracteres na memória de dados a partir do endereço especificado em `self.registers.aux2`.
///   - O número máximo de caracteres a serem armazenados é definido por `self.registers.aux3`.
///   - Não adiciona terminador nulo (`\0`) ao final da string.
///   - Se a string de entrada for maior que o limite, ela é truncada.
///   - Se o endereço de memória exceder os limites, ocorre um erro.
///   - Armazena o tamanho da string lida em `self.registers.aux2`
///
/// - Caso `self.registers.aux1 == 15`:
///   - Lê uma linha da entrada padrão (stdin), remove espaços em branco das extremidades e armazena os caracteres na memória de dados a partir do endereço especificado em `self.registers.aux2`.
///   - O número máximo de bytes a serem escritos é definido por `self.registers.aux3`.
///   - Sempre adiciona um terminador nulo (`\0`) ao final da string armazenada, desde que o tamanho máximo (`aux3`) seja maior que zero.
///   - Se a string de entrada for maior que o limite permitido (considerando o espaço para o terminador nulo), ela é truncada.
///   - Se o endereço de memória exceder os limites, ocorre um erro.
///
/// Exemplos de comportamento para o caso 15:
/// - Se `aux3 == 0`, nada é armazenado.
/// - Se `aux3 == 6` e a entrada for "Hello", armazena "Hello\0".
/// - Se `aux3 == 3` e a entrada for "Hello World", armazena "He\0".

impl StarExecutable for Star {
    fn execute(&mut self) -> Option<(String, Option<StarPosition>)> {
        let instruction_memory_len = match u16::try_from(self.instruction_memory.len()) {
            Ok(len) => len,
            Err(_) => {
                return Some(("StarInstruction memory length exceeds maximum size of 16 bits".to_string(), None))
            }
        };

        'execution_loop: while self.registers.program_counter < instruction_memory_len {
            
            match self.instruction_memory.get_full_instruction_by_program_counter(self.registers.program_counter) {
                Some((ir, ip)) => {
                    self.registers.instruction_register = ir;
                    self.registers.instruction_pointer = ip;
                }
                None => break 'execution_loop,
            }
            
            let instruction_position_option = self
                .position_memory.get_by_program_counter(self.registers.program_counter)
                .cloned();
            // ==== PERFORMANCE DETECTOR ====
            /*
               This part of the code is used to detect non
               state alterable instructions, such as NOPs.
               If the instruction does not alter the state of
               the vm, it will not be decoded and executed.

               This is used to improve performance, as the
               instruction decoder is a costly operation.
            */
            if self.registers.instruction_register == 0b_0000_0000_0000_0000 {
                // NOP instruction, just increment the program counter
                if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                
                continue;
            }

            // ==== INSTRUCTION DECODER ====
            match StarFormat::from_u16(self.registers.instruction_register) {
                StarFormat::Trinity => {
                    match defold_trinity(self.registers.instruction_register) {
                        Some((instruction, reg1, reg2, reg3)) => {
                            match instruction {
                                StarInstruction::Add => {
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);
                                    let (res, is_carry) = reg2_v.overflowing_add(reg3_v);
                                    self.registers.set_general_register_value(reg1, res);
                                    self.registers.carry = if is_carry { 1 } else { 0 };

                                    if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                                }
                                StarInstruction::Sub => {
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);
                                    let (res, is_carry) = reg2_v.overflowing_sub(reg3_v);
                                    self.registers.set_general_register_value(reg1, res);
                                    self.registers.carry = if is_carry { 0xFFFF } else { 0 };

                                    if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                                }

                                StarInstruction::And | StarInstruction::Or | StarInstruction::Xor => {
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);
                                    let res = match instruction {
                                        StarInstruction::And => reg2_v & reg3_v,
                                        StarInstruction::Or => reg2_v | reg3_v,
                                        StarInstruction::Xor => reg2_v ^ reg3_v,
                                        _ => unreachable!(),
                                    };
                                    self.registers.set_general_register_value(reg1, res);

                                    if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                                }

                                StarInstruction::Shl | StarInstruction::Shr => {
                                    let reg2_v: u16 = self.registers.get_general_register_value(reg2);
                                    let reg3_v: u16 = self.registers.get_general_register_value(reg3);

                                    let (res, carry) = match instruction {
                                        StarInstruction::Shl => shift_left_with_carry(reg2_v, reg3_v),
                                        StarInstruction::Shr => shift_right_with_carry(reg2_v, reg3_v),
                                        _ => unreachable!(),
                                    };
                                    self.registers.set_general_register_value(reg1, res);
                                    self.registers.carry = carry;

                                    if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                                }

                                // ==== Branches ====
                                StarInstruction::Beqr
                                | StarInstruction::Bneqr
                                | StarInstruction::Bgtr
                                | StarInstruction::Bltr
                                | StarInstruction::Bgtur
                                | StarInstruction::Bltur => {
                                    let reg1_v = self.registers.get_general_register_value(reg1);
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);

                                    let condition: bool = match instruction {
                                        StarInstruction::Beqr => reg1_v == reg2_v,
                                        StarInstruction::Bneqr => reg1_v != reg2_v,
                                        StarInstruction::Bgtr => 
                                            u16::cast_signed(reg1_v)
                                                > u16::cast_signed(reg2_v)
                                        ,
                                        StarInstruction::Bltr =>
                                            u16::cast_signed(reg1_v)
                                                < u16::cast_signed(reg2_v)
                                        ,
                                        StarInstruction::Bgtur => reg1_v > reg2_v,
                                        StarInstruction::Bltur => reg1_v < reg2_v,
                                        _ => unreachable!(),
                                    };

                                    if condition {
                                        match self.registers.program_counter.checked_add(1) {
                                            Some(ra) => self.registers.return_address = ra,
                                            None => return Some((
                                                "Return address overflow".to_string(),
                                                instruction_position_option,
                                            )),
                                        }

                                        self.registers.program_counter =
                                            self.registers.program_counter.wrapping_add(reg3_v);
                                    } else {
                                        if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                                    }
                                }

                                _ => unimplemented!(),
                            }
                        }

                        None => return Some((
                            "Invalid instruction format for Trinity".to_string(),
                            instruction_position_option,
                        )),
                    }
                }

                StarFormat::Hime => match defold_hime(self.registers.instruction_register) {
                    Some((instruction, reg, imm)) => match instruction {
                        StarInstruction::Lai => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            let regv_low = unsafe { transmute::<u16, (u8, u8)>(reg_v).0 };

                            let new_value = unsafe { transmute::<(u8, u8), u16>((regv_low, imm)) };

                            self.registers.set_general_register_value(reg, new_value);

                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }
                        StarInstruction::Lli => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            let regv_high = unsafe { transmute::<u16, (u8, u8)>(reg_v).1 };

                            let new_value = unsafe { transmute::<(u8, u8), u16>((imm, regv_high)) };

                            self.registers.set_general_register_value(reg, new_value);

                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }
                        _ => unreachable!(),
                    },
                    None => return Some((
                        "Invalid instruction format for Hime".to_string(),
                        instruction_position_option,
                    )),
                },

                StarFormat::Pair => match defold_pair(self.registers.instruction_register) {
                    Some((instruction, reg1, reg2)) => match instruction {
                        StarInstruction::Mulhl => {
                            let reg1_v: u32 = extend_sign_from_u16_to_u32(self.registers.get_general_register_value(reg1));
                            let reg2_v: u32 = extend_sign_from_u16_to_u32(self.registers.get_general_register_value(reg2));

                            let res = reg1_v.wrapping_mul(reg2_v);
                            let (low, high) = unsafe { transmute::<u32, (u16, u16)>(res) };
                            self.registers.high = high;
                            self.registers.low = low;

                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Muluhl => {
                            let reg1_v: u32 = extend_zero_from_u16_to_u32(self.registers.get_general_register_value(reg1));
                            let reg2_v: u32 = extend_zero_from_u16_to_u32(self.registers.get_general_register_value(reg2));

                            let res = reg1_v.wrapping_mul(reg2_v);
                            let (low, high) = unsafe { transmute::<u32, (u16, u16)>(res) };
                            self.registers.high = high;
                            self.registers.low = low;

                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Divhl => {
                            let reg1_v: i16 =
                                 u16::cast_signed(self.registers.get_general_register_value(reg1));
                            let reg2_v: i16 =
                                u16::cast_signed(self.registers.get_general_register_value(reg2));

                            if reg2_v == 0 {
                                self.registers.high = 0xFFFF;
                                self.registers.low = 0xFFFF;
                            } else {
                                let res = reg1_v.wrapping_div(reg2_v);
                                let rem = reg1_v.wrapping_rem(reg2_v);
                                self.registers.high = i16::cast_unsigned(rem);
                                self.registers.low = i16::cast_unsigned(res);
                            }
                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Divuhl => {
                            let reg1_v: u16 = self.registers.get_general_register_value(reg1);
                            let reg2_v: u16 = self.registers.get_general_register_value(reg2);

                            if reg2_v == 0 {
                                self.registers.high = 0xFFFF;
                                self.registers.low = 0xFFFF;
                            } else {
                                let res = reg1_v.wrapping_div(reg2_v);
                                let rem = reg1_v.wrapping_rem(reg2_v);
                                self.registers.high = rem;
                                self.registers.low = res;
                            }
                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Not => {
                            let reg2_v = self.registers.get_general_register_value(reg2);
                            self.registers.set_general_register_value(reg1, !reg2_v);
                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Xlb => {
                            let reg2_v = self.registers.get_general_register_value(reg2);

                            let (low, _) = unsafe { transmute::<u16, (u8, u8)>(reg2_v) };
                            let mut high: u8 = 0b_0000_0000;
                            if reg2_v & 0b_0000_0000_1000_0000 != 0 {
                                high = 0b_1111_1111;
                            }
                            let res = unsafe { transmute::<(u8, u8), u16>((low, high)) };
                            self.registers.set_general_register_value(reg1, res);
                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Lab | StarInstruction::Llb => {
                            let reg1_v = self.registers.get_general_register_value(reg1);
                            let reg2_v = self.registers.get_general_register_value(reg2);

                            let (low, high) = unsafe { transmute::<u16, (u8, u8)>(reg1_v) };
                            match self.data_memory.load(reg2_v) {
                                Ok(value) => {
                                    let v = match instruction {
                                        StarInstruction::Lab => unsafe {
                                            transmute::<(u8, u8), u16>((low, value))
                                        },
                                        StarInstruction::Llb => unsafe {
                                            transmute::<(u8, u8), u16>((value, high))
                                        },
                                        _ => unreachable!(),
                                    };

                                    self.registers.set_general_register_value(reg1, v);
                                }
                                Err(e) => return Some((
                                    e,
                                    instruction_position_option,
                                )),
                            }
                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        StarInstruction::Sab | StarInstruction::Slb => {
                            let reg1_v = self.registers.get_general_register_value(reg1);
                            let reg2_v = self.registers.get_general_register_value(reg2);

                            let (low, high) = unsafe { transmute::<u16, (u8, u8)>(reg1_v) };
                            let value = match instruction {
                                StarInstruction::Sab => high,
                                StarInstruction::Slb => low,
                                _ => unreachable!(),
                            };

                            match self.data_memory.store(reg2_v, value) {
                                Ok(_) => {}
                                Err(e) => return Some((
                                    e,
                                    instruction_position_option,
                                )),
                            }

                            if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));}
                        }

                        _ => unreachable!(),
                    },
                    None => return Some((
                        "Invalid instruction format for Pair".to_string(),
                        instruction_position_option,
                    )),
                },

                StarFormat::Clover => match defold_clover(self.registers.instruction_register) {
                    Some((instruction, reg)) => match instruction {
                        StarInstruction::J => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            match self.registers.program_counter.checked_add(1) {
                                Some(ra) => self.registers.return_address = ra,
                                None => return Some((
                                    "Return address overflow".to_string(),
                                    instruction_position_option,
                                )),
                            }
                            self.registers.program_counter = reg_v;
                        }
                        _ => unreachable!(),
                    },
                    None => return Some((
                        "Invalid instruction format for Clover".to_string(),
                        instruction_position_option,
                    )),
                },

                StarFormat::Ark => {
                    match defold_ark(self.registers.instruction_register) {
                        Some(instruction) => {
                            match instruction {
                                StarInstruction::Mcall => {
                                    let mut interface_option = self.interface.take();

                                    let should_break = if let Some(interface) = interface_option.as_mut() {
                                        interface.as_mut().mcall(self)
                                    } else {
                                        false
                                    };

                                    self.interface = interface_option;

                                    if should_break {
                                        break 'execution_loop;
                                    }

                                    if let Err(err) = self.registers.increment_program_counter() {return Some((err, instruction_position_option));} 
                                }
                                _ => unreachable!(),
                            }
                        }
                        None => return Some((
                            "Invalid instruction format for Ark".to_string(),
                            instruction_position_option,
                        )),
                    }
                }
            }
        }
        io::stdout().flush().unwrap();
        return None;
    }

    

}
