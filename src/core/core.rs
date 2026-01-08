use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::mem::transmute;

use crate::math::u8_from_string;

use crate::core::*;
use crate::generation::*;
use crate::parser::*;
use crate::resolver::*;
use crate::scanner::*;

use crate::math::*;

pub struct Star {
    pub file_table: HashMap<usize, String>,
    pub memories: StarMemories,
    pub registers: StarRegisters,
    pub custom_sections: Vec<StarCustomSectionDefinition>,

    pub interface: Option<Box<dyn StarInterface>>,
}

impl Star {
    /// Creates a new `Star` virtual machine instance with fresh memories, randomized registers,
    /// no attached interface, and no custom sections.
    ///
    /// # Notes
    /// - Data/instruction/position memories start empty/new.
    /// - Registers are randomized (useful for catching uninitialized-dependence bugs in programs).
    /// - Call `reset()` if you want to re-randomize and clear runtime state later.
    pub fn default() -> Self {
        Self {
            file_table: HashMap::new(),
            memories: StarMemories::new(),
            registers: StarRegisters::new_randomized(),
            interface: None,
            custom_sections: Vec::new(),
        }
    }

    /// Registers a **custom section directive** that the assembler/binary loader will recognize.
    ///
    /// This enables extra `.something` sections beyond `.instr` and `.data`.
    ///
    /// # Parameters
    /// - `section_name`: The directive name (e.g. `.foo`). Must be a valid custom directive token.
    /// - `parsing_type`: Whether this section behaves like instruction bytes or data bytes.
    ///
    /// # Errors
    /// Returns `Err` if `section_name` is not accepted as a custom directive name.
    ///
    /// # Side effects
    /// Appends a new `StarCustomSectionDefinition` into `self.custom_sections`.
    pub fn add_section(
        &mut self,
        section_name: &str,
        parsing_type: StarSectionParsingType,
    ) -> Result<(), String> {
        match StarToken::from_string(section_name.to_string()) {
            Ok(StarToken::StarDirective(StarDirective::Custom(_))) => {}
            _ => {
                return Err(
                    "This section name cannot be a custom directive, please use another name"
                        .to_string(),
                );
            }
        }

        self.custom_sections.push(StarCustomSectionDefinition::new(
            section_name.to_string(),
            parsing_type,
        ));

        Ok(())
    }

    /// Convenience wrapper for `add_section(section_name, StarSectionParsingType::Data)`.
    ///
    /// Use this to create custom sections that behave like `.data` (byte-per-token).
    ///
    /// # Errors
    /// Forwards any error from `add_section`.

    pub fn add_data_section(&mut self, section_name: &str) -> Result<(), String> {
        self.add_section(section_name, StarSectionParsingType::Data)
    }

    /// Convenience wrapper for `add_section(section_name, StarSectionParsingType::Instr)`.
    ///
    /// Use this to create custom sections that behave like `.instr` (two bytes per instruction word).
    ///
    /// # Errors
    /// Forwards any error from `add_section`.
    pub fn add_instr_section(&mut self, section_name: &str) -> Result<(), String> {
        self.add_section(section_name, StarSectionParsingType::Instr)
    }

    /// Resets the VM to a clean execution/loading state.
    ///
    /// This will:
    /// - recreate memories (data/instruction/position),
    /// - clear the file table,
    /// - randomize registers again.
    ///
    /// # Notes
    /// Does **not** remove custom sections (they remain registered).
    /// Interface is left as-is unless overwritten elsewhere.
    pub fn reset(&mut self) {
        self.memories = StarMemories::new();
        self.file_table.clear();
        self.registers = StarRegisters::new_randomized();
    }

    /// Loads a program from an **assembly source string** into the VM memories.
    ///
    /// Pipeline:
    /// 1) `reset()` the VM
    /// 2) scan the source into positioned tokens + file table
    /// 3) parse tokens into AST (considering `custom_sections`)
    /// 4) resolve symbols
    /// 5) generate bytes (data, instructions, positions, and custom memories)
    /// 6) commit main data/instr/pos into VM memories
    ///
    /// # Returns
    /// - `StarSymbolTable` for resolved symbols
    /// - custom memories map (bytes) for custom sections
    /// - custom positions map (source positions) for custom instruction sections
    ///
    /// # Errors
    /// Returns `(message, position)` where position may be `None` for non-source-specific failures.
    pub fn load_memory_from_assembly(
        &mut self,
        source: &String,
    ) -> Result<
        (
            StarSymbolTable,
            StarCustomMemoriesHashMap,
            StarCustomMemoryPositionsHashMap,
        ),
        (String, Option<StarPosition>),
    > {
        self.reset();
        match scan(source) {
            Ok((ptokens, file_table)) => {
                self.file_table = file_table;
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok((dm, im, pm, cm, cpm, st)) => {
                        if let Err(e) =
                            self.load_data_instructions_and_position_memory(&dm, &im, &pm)
                        {
                            return Err((e, None));
                        }
                        Ok((st, cm, cpm))
                    }
                    Err((e_string, e_position)) => {
                        return Err((
                            e_string,
                            e_position,
                        ));
                    }
                }
            }
            Err((e_string, e_position)) => {
                return Err((
                    e_string,
                    e_position,
                ));
            }
        }
    }

    /// Loads a program from an **assembly file path** into the VM memories.
    ///
    /// Same pipeline as `load_memory_from_assembly`, but the source is read via `scan_file`.
    ///
    /// # Returns
    /// - `StarSymbolTable` for resolved symbols
    /// - custom memories map (bytes) for custom sections
    /// - custom positions map (source positions) for custom instruction sections
    ///
    /// # Errors
    /// Returns `(message, position)` where position may be `None` for non-source-specific failures.

    pub fn load_memory_from_assembly_file(
        &mut self,
        file_path: &String,
    ) -> Result<
        (
            StarSymbolTable,
            StarCustomMemoriesHashMap,
            StarCustomMemoryPositionsHashMap,
        ),
        (String, Option<StarPosition>),
    > {
        self.reset();
        match scan_file(file_path) {
            Ok((ptokens, file_table)) => {
                self.file_table = file_table;
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok((dm, im, pm, cm, cpm, st)) => {
                        if let Err(e) =
                            self.load_data_instructions_and_position_memory(&dm, &im, &pm)
                        {
                            return Err((e, None));
                        }
                        Ok((st, cm, cpm))
                    }
                    Err((e_string, e_position)) => {
                        return Err((
                            e_string,
                            e_position,
                        ));
                    }
                }
            }
            Err((e_string, e_position)) => {
                return Err((
                    e_string,
                    e_position,
                ));
            }
        }
    }

    fn process_positioned_tokens_from_assembly(
        &mut self,
        ptokens: Vec<StarPositionedToken>,
    ) -> Result<
        (
            StarDataMemoryBytes,
            StarInstructionMemoryBytes,
            StarPositionMemoryBytes,
            StarCustomMemoriesHashMap,
            StarCustomMemoryPositionsHashMap,
            StarSymbolTable,
        ),
        (String, Option<StarPosition>),
    > {
        match parse(&ptokens, &self.custom_sections) {
            Ok(mut ast) => match resolve(&mut ast) {
                Ok(symbol_table) => match generate_bytes_from_ast(&ast) {
                    Ok((dm, im, pm, cm, cpm)) => Ok((dm, im, pm, cm, cpm, symbol_table)),
                    Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                },
                Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
            },
            Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
        }
    }

    /// Looks up the original file name recorded in `file_table` by its numeric id.
    ///
    /// This is typically used for error reporting and for reconstructing file paths in positions.
    ///
    /// # Returns
    /// `Some(file_name)` if the id exists; otherwise `None`.

    pub fn get_file_name_by_id(&self, file_id: usize) -> Option<String> {
        self.file_table.get(&file_id).cloned()
    }

    /// Attaches a system-call interface to this VM.
    ///
    /// The interface may be invoked by the `Mcall` instruction during execution.
    ///
    /// # Parameters
    /// - `interface`: Implementation of `StarInterface` to handle machine calls.
    ///
    /// # Notes
    /// Overwrites any previously attached interface.
    pub fn set_interface(&mut self, interface: Box<dyn StarInterface>) {
        self.interface = Some(interface);
    }

    /// Returns a mutable reference to the optional system-call interface.
    ///
    /// # Use cases
    /// - swap/replace the interface
    /// - temporarily `take()` it to avoid borrow conflicts (as done in `execute`)
    pub fn get_interface_mut(&mut self) -> &mut Option<Box<dyn StarInterface>> {
        &mut self.interface
    }

    /// Returns an immutable reference to the optional system-call interface.
    ///
    /// # Notes
    /// Signature takes `&mut self` in this version; it can be changed to `&self` if desired.

    pub fn get_interface(&mut self) -> &Option<Box<dyn StarInterface>> {
        &self.interface
    }

    fn load_data_instructions_and_position_memory(
        &mut self,
        data_memory: &Vec<u8>,
        instruction_memory: &Vec<u8>,
        position_memory: &Vec<StarPosition>,
    ) -> Result<(), String> {
        for (i, d) in data_memory.iter().enumerate() {
            match self.memories.data_memory.get_mut(i) {
                Some(byte) => {
                    *byte = d.clone();
                }
                None => return Err("Data memory overflow".to_string()),
            }
        }

        self.memories.instruction_memory = instruction_memory.clone();
        self.memories.position_memory = position_memory.clone();

        Ok(())
    }

    /// Executes the currently loaded program until completion.
    ///
    /// Execution stops when:
    /// - `program_counter` reaches the end of instruction memory, or
    /// - an `Mcall` causes the attached interface to request termination.
    ///
    /// # Behavior
    /// - Fetches a 16-bit instruction by `program_counter`
    /// - Reads the associated source position (when available)
    /// - Fast-path for NOP (`0x0000`)
    /// - Decodes instruction format and executes the instruction semantics
    ///
    /// # Returns
    /// `Ok(())` on normal termination.
    ///
    /// # Errors
    /// Returns `(message, position)` where position may be the source position of the current instruction.

    pub fn execute(&mut self) -> Result<(), (String, Option<StarPosition>)> {
        let instruction_memory_len = match u16::try_from(self.memories.instruction_memory.len()) {
            Ok(len) => len,
            Err(_) => {
                return Err((
                    "StarInstruction memory length exceeds maximum size of 16 bits".to_string(),
                    None,
                ));
            }
        };

        'execution_loop: while self.registers.program_counter < instruction_memory_len {
            match self
                .memories
                .instruction_memory
                .get_full_instruction_by_program_counter(self.registers.program_counter)
            {
                Some((ir, ip)) => {
                    self.registers.instruction_register = ir;
                    self.registers.instruction_pointer = ip;
                }
                None => break 'execution_loop,
            }

            let instruction_position_option = self
                .memories
                .position_memory
                .get_by_program_counter(self.registers.program_counter)
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
                if let Err(err) = self.registers.increment_program_counter() {
                    return Err((err, instruction_position_option));
                }

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

                                    if let Err(err) = self.registers.increment_program_counter() {
                                        return Err((err, instruction_position_option));
                                    }
                                }
                                StarInstruction::Sub => {
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);
                                    let (res, is_carry) = reg2_v.overflowing_sub(reg3_v);
                                    self.registers.set_general_register_value(reg1, res);
                                    self.registers.carry = if is_carry { 0xFFFF } else { 0 };

                                    if let Err(err) = self.registers.increment_program_counter() {
                                        return Err((err, instruction_position_option));
                                    }
                                }

                                StarInstruction::And
                                | StarInstruction::Or
                                | StarInstruction::Xor => {
                                    let reg2_v = self.registers.get_general_register_value(reg2);
                                    let reg3_v = self.registers.get_general_register_value(reg3);
                                    let res = match instruction {
                                        StarInstruction::And => reg2_v & reg3_v,
                                        StarInstruction::Or => reg2_v | reg3_v,
                                        StarInstruction::Xor => reg2_v ^ reg3_v,
                                        _ => unreachable!(),
                                    };
                                    self.registers.set_general_register_value(reg1, res);

                                    if let Err(err) = self.registers.increment_program_counter() {
                                        return Err((err, instruction_position_option));
                                    }
                                }

                                StarInstruction::Shl | StarInstruction::Shr => {
                                    let reg2_v: u16 =
                                        self.registers.get_general_register_value(reg2);
                                    let reg3_v: u16 =
                                        self.registers.get_general_register_value(reg3);

                                    let (res, carry) = match instruction {
                                        StarInstruction::Shl => {
                                            shift_left_with_carry(reg2_v, reg3_v)
                                        }
                                        StarInstruction::Shr => {
                                            shift_right_with_carry(reg2_v, reg3_v)
                                        }
                                        _ => unreachable!(),
                                    };
                                    self.registers.set_general_register_value(reg1, res);
                                    self.registers.carry = carry;

                                    if let Err(err) = self.registers.increment_program_counter() {
                                        return Err((err, instruction_position_option));
                                    }
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
                                        StarInstruction::Bgtr => {
                                            u16::cast_signed(reg1_v) > u16::cast_signed(reg2_v)
                                        }
                                        StarInstruction::Bltr => {
                                            u16::cast_signed(reg1_v) < u16::cast_signed(reg2_v)
                                        }
                                        StarInstruction::Bgtur => reg1_v > reg2_v,
                                        StarInstruction::Bltur => reg1_v < reg2_v,
                                        _ => unreachable!(),
                                    };

                                    if condition {
                                        match self.registers.program_counter.checked_add(1) {
                                            Some(ra) => self.registers.return_address = ra,
                                            None => {
                                                return Err((
                                                    "Return address overflow".to_string(),
                                                    instruction_position_option,
                                                ));
                                            }
                                        }

                                        self.registers.program_counter =
                                            self.registers.program_counter.wrapping_add(reg3_v);
                                    } else {
                                        if let Err(err) = self.registers.increment_program_counter()
                                        {
                                            return Err((err, instruction_position_option));
                                        }
                                    }
                                }

                                _ => unimplemented!(),
                            }
                        }

                        None => {
                            return Err((
                                "Invalid instruction format for Trinity".to_string(),
                                instruction_position_option,
                            ));
                        }
                    }
                }

                StarFormat::Hime => match defold_hime(self.registers.instruction_register) {
                    Some((instruction, reg, imm)) => match instruction {
                        StarInstruction::Lai => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            let regv_low = unsafe { transmute::<u16, (u8, u8)>(reg_v).0 };

                            let new_value = unsafe { transmute::<(u8, u8), u16>((regv_low, imm)) };

                            self.registers.set_general_register_value(reg, new_value);

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }
                        StarInstruction::Lli => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            let regv_high = unsafe { transmute::<u16, (u8, u8)>(reg_v).1 };

                            let new_value = unsafe { transmute::<(u8, u8), u16>((imm, regv_high)) };

                            self.registers.set_general_register_value(reg, new_value);

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }
                        _ => unreachable!(),
                    },
                    None => {
                        return Err((
                            "Invalid instruction format for Hime".to_string(),
                            instruction_position_option,
                        ));
                    }
                },

                StarFormat::Pair => match defold_pair(self.registers.instruction_register) {
                    Some((instruction, reg1, reg2)) => match instruction {
                        StarInstruction::Mulhl => {
                            let reg1_v: u32 = extend_sign_from_u16_to_u32(
                                self.registers.get_general_register_value(reg1),
                            );
                            let reg2_v: u32 = extend_sign_from_u16_to_u32(
                                self.registers.get_general_register_value(reg2),
                            );

                            let res = reg1_v.wrapping_mul(reg2_v);
                            let (low, high) = unsafe { transmute::<u32, (u16, u16)>(res) };
                            self.registers.high = high;
                            self.registers.low = low;

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }

                        StarInstruction::Muluhl => {
                            let reg1_v: u32 = extend_zero_from_u16_to_u32(
                                self.registers.get_general_register_value(reg1),
                            );
                            let reg2_v: u32 = extend_zero_from_u16_to_u32(
                                self.registers.get_general_register_value(reg2),
                            );

                            let res = reg1_v.wrapping_mul(reg2_v);
                            let (low, high) = unsafe { transmute::<u32, (u16, u16)>(res) };
                            self.registers.high = high;
                            self.registers.low = low;

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
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
                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
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
                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }

                        StarInstruction::Not => {
                            let reg2_v = self.registers.get_general_register_value(reg2);
                            self.registers.set_general_register_value(reg1, !reg2_v);
                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
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
                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }

                        StarInstruction::Lab | StarInstruction::Llb => {
                            let reg1_v = self.registers.get_general_register_value(reg1);
                            let reg2_v = self.registers.get_general_register_value(reg2);

                            let (low, high) = unsafe { transmute::<u16, (u8, u8)>(reg1_v) };
                            match self.memories.data_memory.load(reg2_v) {
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
                                Err(e) => return Err((e, instruction_position_option)),
                            }
                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
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

                            match self.memories.data_memory.store(reg2_v, value) {
                                Ok(_) => {}
                                Err(e) => return Err((e, instruction_position_option)),
                            }

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }

                        _ => unreachable!(),
                    },
                    None => {
                        return Err((
                            "Invalid instruction format for Pair".to_string(),
                            instruction_position_option,
                        ));
                    }
                },

                StarFormat::Clover => match defold_clover(self.registers.instruction_register) {
                    Some((instruction, reg)) => match instruction {
                        StarInstruction::J => {
                            let reg_v = self.registers.get_general_register_value(reg);
                            match self.registers.program_counter.checked_add(1) {
                                Some(ra) => self.registers.return_address = ra,
                                None => {
                                    return Err((
                                        "Return address overflow".to_string(),
                                        instruction_position_option,
                                    ));
                                }
                            }
                            self.registers.program_counter = reg_v;
                        }
                        _ => unreachable!(),
                    },
                    None => {
                        return Err((
                            "Invalid instruction format for Clover".to_string(),
                            instruction_position_option,
                        ));
                    }
                },

                StarFormat::Ark => match defold_ark(self.registers.instruction_register) {
                    Some(instruction) => match instruction {
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

                            if let Err(err) = self.registers.increment_program_counter() {
                                return Err((err, instruction_position_option));
                            }
                        }
                        _ => unreachable!(),
                    },
                    None => {
                        return Err((
                            "Invalid instruction format for Ark".to_string(),
                            instruction_position_option,
                        ));
                    }
                },
            }
        }
        io::stdout().flush().unwrap();
        return Ok(());
    }

    /// Loads VM memories from a **textual binary format** (your `.instr`/`.data` style with `{:08b}` tokens).
    ///
    /// This loader:
    /// - resets the VM
    /// - recognizes `.instr`, `.data`, and any registered `custom_sections`
    /// - reads tokens, tracking line/column and indentation-based column behavior
    /// - commits bytes into main instruction/data memories and custom memories
    ///
    /// # Parameters
    /// - `source`: The textual binary content (not raw bytes).
    ///
    /// # Returns
    /// - custom memories map (bytes) for custom sections
    /// - custom positions map (source positions) for custom instruction sections
    ///
    /// # Errors
    /// Returns `(message, position)` for malformed tokens, unknown sections, or memory overflow.
    pub fn load_memory_from_binary(
        &mut self,
        source: &String,
    ) -> Result<
        (StarCustomMemoriesHashMap, StarCustomMemoryPositionsHashMap),
        (String, Option<StarPosition>),
    > {
        self.reset();

        let mut section_types: HashMap<String, StarSectionParsingType> = HashMap::new();
        section_types.insert(".instr".to_string(), StarSectionParsingType::Instr);
        section_types.insert(".data".to_string(), StarSectionParsingType::Data);

        for cs in self.custom_sections.iter() {
            section_types.insert(cs.name.clone(), cs.section_type.clone());
        }

        let mut main_instruction_memory: Vec<u8> = Vec::new();
        let mut main_position_memory: Vec<StarPosition> = Vec::new();
        let mut main_data_bytes: Vec<u8> = Vec::new();

        let mut custom_memories: StarCustomMemoriesHashMap = StarCustomMemoriesHashMap::new();
        let mut custom_positions: StarCustomMemoryPositionsHashMap =
            StarCustomMemoryPositionsHashMap::new();

        let mut section: String = ".instr".to_string();

        let mut actual_line: usize = 1;
        let mut actual_column: usize = 1;

        let mut token_line: usize = 1;
        let mut token_column: usize = 1;

        let mut line_has_indentation: bool = false;

        let mut accumulator: String = String::new();
        let binding = source.replace("\r", "");
        let mut chars = binding.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\n' | '\t' | ' ' => {
                    if ch == '\n' {
                        actual_line += 1;
                        actual_column = 1;
                        line_has_indentation = false;
                    } else if ch == '\t' {
                        line_has_indentation = true;
                        actual_column += 1;
                    } else {
                        actual_column += 1;
                    }

                    if !accumulator.is_empty() {
                        let tk = accumulator.clone();
                        accumulator.clear();

                        match self.binary_commit_token(
                            &tk,
                            token_line,
                            token_column,
                            actual_column,
                            line_has_indentation,
                            &section_types,
                            &mut section,
                            &mut main_instruction_memory,
                            &mut main_position_memory,
                            &mut main_data_bytes,
                            &mut custom_memories,
                            &mut custom_positions,
                        ) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                    }

                    continue;
                }
                _ => {
                    if accumulator.is_empty() {
                        token_line = actual_line;
                        token_column = actual_column;
                    }
                    accumulator.push(ch);
                    actual_column += 1;
                }
            }
        }

        if !accumulator.is_empty() {
            let tk = accumulator.clone();

            match self.binary_commit_token(
                &tk,
                token_line,
                token_column,
                actual_column,
                line_has_indentation,
                &section_types,
                &mut section,
                &mut main_instruction_memory,
                &mut main_position_memory,
                &mut main_data_bytes,
                &mut custom_memories,
                &mut custom_positions,
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        for (i, b) in main_data_bytes.iter().enumerate() {
            match self.memories.data_memory.get_mut(i) {
                Some(cell) => *cell = *b,
                None => return Err(("Data memory out of bounds".to_string(), None)),
            }
        }

        self.memories.instruction_memory = main_instruction_memory;
        self.memories.position_memory = main_position_memory;

        Ok((custom_memories, custom_positions))
    }

    /// Loads VM memories from a **binary text file** (the same format accepted by `load_memory_from_binary`).
    ///
    /// The path is canonicalized and the file is read as text (CRLF normalized).
    ///
    /// # Errors
    /// Returns `(message, position)` where position is usually `None` for file IO failures.

    pub fn load_memory_from_binary_file(
        &mut self,
        file_path: &String,
    ) -> Result<
        (StarCustomMemoriesHashMap, StarCustomMemoryPositionsHashMap),
        (String, Option<StarPosition>),
    > {
        self.reset();

        let absolute_file_path: String = match fs::canonicalize(file_path.clone()) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => return Err((format!("Failed to read binary file '{}'", file_path), None)),
        };

        let file_content = match fs::read_to_string(&absolute_file_path) {
            Ok(content) => content.replace("\r", ""),
            Err(_) => {
                return Err((
                    format!("Failed to read binary file '{}'", absolute_file_path),
                    None,
                ));
            }
        };

        self.load_memory_from_binary(&file_content)
    }

    /// Compiles assembly source into the **textual binary format** used by `load_memory_from_binary`.
    ///
    /// Pipeline:
    /// - reset VM
    /// - scan → parse → resolve → generate bytes
    /// - serialize `.instr`, custom instr sections, `.data`, custom data sections
    ///
    /// # Returns
    /// A `String` containing the binary text representation.
    ///
    /// # Errors
    /// Returns `(message, position)` on scan/parse/resolve/generate failures.
    pub fn get_binary_from_assembly(
        &mut self,
        source: &String,
    ) -> Result<String, (String, Option<StarPosition>)> {
        self.reset();

        match scan(source) {
            Ok((ptokens, file_table)) => {
                self.file_table = file_table;
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok((dm, im, _pm, cm, _cpm, _st)) => {
                        Ok(self.binary_serialize_text(&dm, &im, &cm))
                    }
                    Err((e_string, e_position)) => Err((
                        e_string, 
                        e_position,
                    )),
                }
            }
            Err((e_string, e_position)) => Err((
                e_string,
                e_position,
            )),
        }
    }

    /// Compiles an assembly file into the **textual binary format** used by `load_memory_from_binary`.
    ///
    /// Same as `get_binary_from_assembly`, but reads tokens from a file via `scan_file`.
    ///
    /// # Returns
    /// A `String` containing the binary text representation.
    ///
    /// # Errors
    /// Returns `(message, position)` on scan/parse/resolve/generate failures.
    pub fn get_binary_from_assembly_file(
        &mut self,
        file_path: &String,
    ) -> Result<String, (String, Option<StarPosition>)> {
        self.reset();

        match scan_file(file_path) {
            Ok((ptokens, file_table)) => {
                self.file_table = file_table;
                match self.process_positioned_tokens_from_assembly(ptokens) {
                    Ok((dm, im, _pm, cm, _cpm, _st)) => {
                        Ok(self.binary_serialize_text(&dm, &im, &cm))
                    }
                    Err((e_string, e_position)) => Err((
                        e_string,
                        e_position,
                    )),
                }
            }
            Err((e_string, e_position)) => Err((
                e_string,
                e_position,
            )),
        }
    }

    /// Compiles assembly source and writes the resulting **binary text** to a destination file.
    ///
    /// # Parameters
    /// - `source`: assembly source code
    /// - `dest_file_path`: output file path for the binary text
    ///
    /// # Errors
    /// Forwards errors from compilation or file writing.
    pub fn save_binary_on_file_from_assembly(
        &mut self,
        source: &String,
        dest_file_path: &String,
    ) -> Result<(), (String, Option<StarPosition>)> {
        let text = match self.get_binary_from_assembly(source) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

        match self.binary_write_string_on_file(dest_file_path, &text) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Compiles an assembly file and writes the resulting **binary text** to a destination file.
    ///
    /// # Parameters
    /// - `source_file_path`: path to the assembly input
    /// - `dest_file_path`: output file path for the binary text
    ///
    /// # Errors
    /// Forwards errors from compilation or file writing.
    pub fn save_binary_on_file_from_assembly_file(
        &mut self,
        source_file_path: &String,
        dest_file_path: &String,
    ) -> Result<(), (String, Option<StarPosition>)> {
        let text = match self.get_binary_from_assembly_file(source_file_path) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

        match self.binary_write_string_on_file(dest_file_path, &text) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn binary_write_string_on_file(
        &self,
        file_path: &String,
        content: &String,
    ) -> Result<(), (String, Option<StarPosition>)> {
        let mut file = match File::create(file_path.clone()) {
            Ok(f) => f,
            Err(_) => {
                return Err((
                    format!("Failed to create binary file '{}'", file_path),
                    None,
                ));
            }
        };

        if file.write_all(content.as_bytes()).is_err() {
            return Err(("Failed to write on binary file".to_string(), None));
        }

        Ok(())
    }

    fn binary_commit_token(
        &self,
        token_raw: &str,
        token_line: usize,
        token_column: usize,
        actual_column_now: usize,
        line_has_indentation: bool,
        section_types: &HashMap<String, StarSectionParsingType>,
        section: &mut String,
        main_instruction_memory: &mut Vec<u8>,
        main_position_memory: &mut Vec<StarPosition>,
        main_data_bytes: &mut Vec<u8>,
        custom_memories: &mut StarCustomMemoriesHashMap,
        custom_positions: &mut StarCustomMemoryPositionsHashMap,
    ) -> Result<(), (String, Option<StarPosition>)> {
        let token = self.binary_sanitize_token(token_raw);

        if token.is_empty() {
            return Ok(());
        }

        if token.starts_with('.') {
            if section_types.contains_key(&token) {
                *section = token;
                return Ok(());
            }
            return Err((
                "Unknown section".to_string(),
                Some(StarPosition::new(None, token_line, Some(token_column))),
            ));
        }

        let section_type = match section_types.get(section) {
            Some(t) => t,
            None => {
                return Err((
                    "Unknown section".to_string(),
                    Some(StarPosition::new(None, token_line, Some(token_column))),
                ));
            }
        };

        match section_type {
            StarSectionParsingType::Instr => {
                let b = match self.binary_parse_u8_token(
                    &token,
                    "Invalid binary instruction",
                    token_line,
                    token_column,
                ) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                let col_for_pos = if line_has_indentation {
                    Some(actual_column_now)
                } else {
                    None
                };

                self.binary_push_instr_byte(
                    section,
                    b,
                    token_line,
                    col_for_pos,
                    main_instruction_memory,
                    main_position_memory,
                    custom_memories,
                    custom_positions,
                );

                Ok(())
            }
            StarSectionParsingType::Data => {
                let b = match self.binary_parse_u8_token(
                    &token,
                    "Invalid binary data",
                    token_line,
                    token_column,
                ) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                self.binary_push_data_byte(section, b, main_data_bytes, custom_memories);

                Ok(())
            }
        }
    }

    fn binary_sanitize_token(&self, token: &str) -> String {
        token.trim().trim_start_matches('\u{feff}').to_string()
    }

    fn binary_parse_u8_token(
        &self,
        token: &str,
        message: &str,
        line: usize,
        column: usize,
    ) -> Result<u8, (String, Option<StarPosition>)> {
        if token.len() != 8 {
            return Err((
                message.to_string(),
                Some(StarPosition::new(None, line, Some(column))),
            ));
        }

        let mut aux = "0b".to_string();
        aux.push_str(token);

        match u8_from_string(aux) {
            Ok(v) => Ok(v),
            Err(_) => Err((
                message.to_string(),
                Some(StarPosition::new(None, line, Some(column))),
            )),
        }
    }

    fn binary_push_instr_byte(
        &self,
        section: &str,
        byte: u8,
        token_line: usize,
        col_for_pos: Option<usize>,
        main_instruction_memory: &mut Vec<u8>,
        main_position_memory: &mut Vec<StarPosition>,
        custom_memories: &mut StarCustomMemoriesHashMap,
        custom_positions: &mut StarCustomMemoryPositionsHashMap,
    ) {
        if section == ".instr" {
            main_instruction_memory.push(byte);

            if main_instruction_memory.len() % 2 == 0 {
                main_position_memory.push(StarPosition::new(None, token_line, col_for_pos));
            }

            return;
        }

        let entry = custom_memories
            .entry(section.to_string())
            .or_insert_with(Vec::new);
        entry.push(byte);

        if entry.len() % 2 == 0 {
            let pentry = custom_positions
                .entry(section.to_string())
                .or_insert_with(Vec::new);
            pentry.push(StarPosition::new(None, token_line, col_for_pos));
        }
    }

    fn binary_push_data_byte(
        &self,
        section: &str,
        byte: u8,
        main_data_bytes: &mut Vec<u8>,
        custom_memories: &mut StarCustomMemoriesHashMap,
    ) {
        if section == ".data" {
            main_data_bytes.push(byte);
            return;
        }

        let entry = custom_memories
            .entry(section.to_string())
            .or_insert_with(Vec::new);
        entry.push(byte);
    }

    fn binary_serialize_text(
        &self,
        data_memory_bytes: &Vec<u8>,
        instruction_memory_bytes: &Vec<u8>,
        custom_memories: &StarCustomMemoriesHashMap,
    ) -> String {
        let mut out: String = String::new();

        out.push_str(".instr\n");
        self.binary_write_instr_bytes(&mut out, instruction_memory_bytes);

        for cs in self.custom_sections.iter() {
            if cs.section_type != StarSectionParsingType::Instr {
                continue;
            }

            match custom_memories.get(&cs.name) {
                Some(bytes) => {
                    out.push_str(&format!("{}\n", cs.name));
                    self.binary_write_instr_bytes(&mut out, bytes);
                }
                None => {}
            }
        }

        out.push_str(".data\n");
        self.binary_write_data_bytes(&mut out, data_memory_bytes);

        for cs in self.custom_sections.iter() {
            if cs.section_type != StarSectionParsingType::Data {
                continue;
            }

            match custom_memories.get(&cs.name) {
                Some(bytes) => {
                    out.push_str(&format!("{}\n", cs.name));
                    self.binary_write_data_bytes(&mut out, bytes);
                }
                None => {}
            }
        }

        out
    }

    fn binary_write_instr_bytes(&self, out: &mut String, bytes: &Vec<u8>) {
        let mut i: usize = 0;
        while i < bytes.len() {
            let b1 = bytes[i];
            let b2 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
            out.push_str(&format!("{:08b} {:08b} \n", b1, b2));
            i += 2;
        }
    }

    fn binary_write_data_bytes(&self, out: &mut String, bytes: &Vec<u8>) {
        for b in bytes.iter() {
            out.push_str(&format!("{:08b} \n", b));
        }
    }
}
