use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::mem::transmute;

use crate::core::*;
use crate::generateable::*;
use crate::resolveable::*;
use crate::scannable::*;
use crate::parseable::*;

use crate::math::*;

pub struct Star {
    pub file_table: HashMap<usize, String>,
    pub data_memory: StarDataMemory,
    pub instruction_memory: StarInstructionMemory,
    pub position_memory: StarPositionMemory,
    pub registers: StarRegisters,

    pub interface: Option<Box<dyn StarInterface>>,
}

impl Star {
    pub fn new() -> Self {
        let mut data_memory = [0; DATA_MEMORY_SIZE];
        for b in data_memory.iter_mut() {
            *b = rand::random::<u8>();
        }
        Self {
            file_table: HashMap::new(),
            data_memory: data_memory,
            instruction_memory: Vec::new(),
            position_memory: Vec::new(),
            registers: StarRegisters::new(),
            interface: None,
        }
    }

    pub fn load_from_assembly(&mut self, source: &String) -> Result<(StarSymbolTable, usize), (String, Option<StarPosition>)> {
        match self.scan(source) {
            Ok(ptokens) => {
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok(okay) => Ok(okay),
                    Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                }
            }
            Err(e) => return Err(e),
        }
    }

    pub fn load_from_assembly_file(&mut self, file_path: &str) -> Result<(StarSymbolTable, usize), (String, Option<StarPosition>)> {        
        
        match self.scan_file(file_path) {
            Ok(ptokens) => {
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok(okay) => Ok(okay),
                    Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                }
            }
            Err(e) => return Err(e),
        }
    }

    fn process_positioned_tokens_from_assembly(&mut self, ptokens: Vec<StarPositionedToken>) ->Result<(StarSymbolTable, usize), (String, StarPosition)> {
        match self.parse(&ptokens) {
            Ok(mut ast) => {
                match self.resolve(&mut ast) {
                    Ok(symbol_table) => {
                        match self.generate(&ast) {
                            Ok(data_section_size) => {
                                return Ok((symbol_table, data_section_size));
                            }
                            Err((e_string, e_position)) => return Err((e_string, e_position)),
                        }
                    }
                    Err((e_string, e_position)) => return Err((e_string, e_position)),
                }
            }
            Err((e_string, e_position)) => return Err((e_string, e_position)),
        }
    }


    pub fn load_from_binary_file(&mut self, file_path: &str) -> Result<usize, (String, Option<StarPosition>)> {
        let file_path_string = file_path.to_string();
        let absolute_file_path: String = match fs::canonicalize(file_path_string.clone()) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => 
                return Err((
                    format!("Failed to read binary file '{}'", file_path_string),
                    None,
                )),
        };

        let file_content = match fs::read_to_string(&absolute_file_path) {
            Ok(content) => content.replace("\r", ""),
            Err(_) => {
                return Err((
                    format!("Failed to read binary file '{}'", absolute_file_path),
                    None,
                ))
            }
        };

        return self.load_from_binary(&file_content);
    }

    pub fn load_from_binary(&mut self, source: &String) -> Result<usize, (String, Option<StarPosition>)> {
        
        let id_option: Option<usize> = None;
        
        let mut section = ".instr".to_string();
        let mut actual_line: usize = 1;
        let mut actual_column: usize = 1;

        let mut data_memory_vector: Vec<u8> = Vec::new();

        let mut tkn_line: usize = 1;
        let mut tkn_column: usize = 1;

        let mut line_has_identation = false;

        let mut accumulator = String::new();
        let mut chars = source.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\n' | '\t' | ' ' => {
                    if ch == '\n' {
                        actual_line += 1;
                        actual_column = 1;
                        line_has_identation = false;
                    } else if ch == '\t' {
                        line_has_identation = true;
                        actual_column += 1;
                    } else {
                        // ' '
                        actual_column += 1;
                    }

                    if accumulator == ".instr" {
                        section = ".instr".to_string();
                        accumulator.clear();
                    } else if accumulator == ".data" {
                        section = ".data".to_string();
                        accumulator.clear();
                    } else if !accumulator.is_empty() {
                        match section.as_str() {
                            ".instr" => {
                                let mut aux = "0b".to_string();
                                aux.push_str(&accumulator);

                                match u8_from_string(aux) {
                                    Ok(v) => {
                                        self.instruction_memory.push(v);
                                    }
                                    Err(_) => {
                                        return Err((
                                            "Invalid binary instruction".to_string(),
                                            Some(StarPosition::new(
                                                id_option,
                                                tkn_line,
                                                Some(tkn_column),
                                            )),
                                        ));
                                    }
                                }

                                if self.instruction_memory.len() % 2 == 0 {
                                    self.position_memory.push(StarPosition::new(
                                        id_option,
                                        tkn_line,
                                        if line_has_identation {
                                            Some(actual_column)
                                        } else {
                                            None
                                        },
                                    ))
                                }
                            }

                            ".data" => {
                                let mut aux = "0b".to_string();
                                aux.push_str(&accumulator);

                                match u8_from_string(aux) {
                                    Ok(v) => {
                                        data_memory_vector.push(v);
                                    }
                                    Err(_) => {
                                        return Err((
                                            "Invalid binary data".to_string(),
                                            Some(StarPosition::new(
                                                id_option,
                                                tkn_line,
                                                Some(tkn_column),
                                            )),
                                        ));
                                    }
                                }
                            }

                            _ => {
                                return Err((
                                    "Unknown section".to_string(),
                                    Some(StarPosition::new(id_option, tkn_line, Some(tkn_column))),
                                ));
                            }
                        }

                        accumulator.clear();
                    }

                    continue;
                }

                _ => {
                    if accumulator.is_empty() {
                        tkn_line = actual_line;
                        tkn_column = actual_column;
                    }
                    accumulator.push(ch);
                    actual_column += 1;
                }
            }
        }

        // Se o arquivo não termina com whitespace, ainda pode ter token pendente.
        if !accumulator.is_empty() {
            match section.as_str() {
                ".instr" => {
                    let mut aux = "0b".to_string();
                    aux.push_str(&accumulator);

                    let v = match u8_from_string(aux) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err((
                                "Invalid binary instruction".to_string(),
                                Some(StarPosition::new(id_option, tkn_line, Some(tkn_column))),
                            ))
                        }
                    };

                    self.instruction_memory.push(v);

                    if self.instruction_memory.len() % 2 == 0 {
                        self.position_memory.push(StarPosition::new(
                            id_option,
                            tkn_line,
                            if line_has_identation {
                                Some(actual_column)
                            } else {
                                None
                            },
                        ))
                    }
                }

                ".data" => {
                    let mut aux = "0b".to_string();
                    aux.push_str(&accumulator);

                    let v = match u8_from_string(aux) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err((
                                "Invalid binary data".to_string(),
                                Some(StarPosition::new(id_option, tkn_line, Some(tkn_column))),
                            ))
                        }
                    };

                    data_memory_vector.push(v);
                }

                _ => {
                    return Err((
                        "Unknown section".to_string(),
                        Some(StarPosition::new(id_option, tkn_line, Some(tkn_column))),
                    ));
                }
            }
        }

        for (i, b) in data_memory_vector.iter().enumerate() {
            match self.data_memory.get_mut(i) {
                Some(memory_byte) => *memory_byte = *b,
                None => return Err(("Data memory out of bounds".to_string(), None)),
            }
        }

        Ok(data_memory_vector.len())
    }

    pub fn save_binary(&self, file_path: &str, data_section_size: usize) -> Result<(), String> {
        let mut file = match File::create(file_path.to_string()) {
            Ok(f) => f,
            Err(_) => return Err("Failed to create binary file".to_string()),
        };

        if file.write_all(".instr".as_bytes()).is_err() {
            return Err("Failed to write on binary file".to_string());
        }

        for (index, instr) in self.instruction_memory.iter().enumerate() {
            let mut output = format!("{:08b} ", instr);
            if index % 2 == 0 {
                output = format!("\n{}", output);
            }

            if file.write_all(output.as_bytes()).is_err() {
                return Err("Failed to write on binary file".to_string());
            }
        }

        if file.write_all("\n.data".as_bytes()).is_err() {
            return Err("Failed to write on binary file".to_string());
        }

        for byte_index in 0..data_section_size {
            let byte = match self.data_memory.get(byte_index) {
                Some(b) => *b,
                None => return Err("Data memory out of bounds".to_string()),
            };

            let output = format!("\n{:08b} ", byte);
            if file.write_all(output.as_bytes()).is_err() {
                return Err("Failed to write on binary file".to_string());
            }
        }

        Ok(())
    }
    
    pub fn get_file_id_by_path(&self, file_path: &String) -> Option<usize> {
        self.file_table.iter().find_map(|(id, path)| if path == file_path { Some(*id) } else { None })
    }

    pub fn get_file_name(&self, file_id: usize) -> String {
        match self.file_table.get(&file_id) {
            Some(name) => name.clone(),
            None => "Unknown file".to_string(),
        }
    }

    pub fn set_interface(&mut self, interface: Box<dyn StarInterface>) {
        self.interface = Some(interface);
    }

    pub fn take_interface(&mut self) -> Option<Box<dyn StarInterface>> {
        self.interface.take()
    }

    pub fn take_interface_mut(&mut self) -> Option<&mut Box<dyn StarInterface>> {
        self.interface.as_mut().take()
    }
    pub fn execute(&mut self) -> Option<(String, Option<StarPosition>)> {
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