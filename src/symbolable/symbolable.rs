use std::collections::HashMap;

use crate::core::*;
use crate::math::*;
use crate::parseable::*;
use crate::resolveable::*;
use crate::utils::*;

pub trait Symbolable {
    fn get_symbol_table(&self, ast: &mut Ast) -> Result<SymbolTable, (String, Position)>;
}

impl Symbolable for Star {
    fn get_symbol_table(&self, ast: &mut Ast) -> Result<SymbolTable, (String, Position)> {
        let mut symbol_table: SymbolTable = HashMap::new();

        // >>>> DATA MEMORY <<<<
        let mut data_memory_counter: usize = 0;

        for data_camp in ast.data_field.iter() {
            // ==== Process of collecting labels ====
            for label_ptk in data_camp.label_declarations.iter() {
                match label_ptk.token {
                    Token::LabelDeclaration(ref label_name) => {
                        if symbol_table.contains_key(label_name) {
                            return Err((
                                format!("Label '{}' already declared", label_name),
                                label_ptk.position,
                            ));
                        } else {
                            match u16::try_from(data_memory_counter) {
                                Ok(address) => {
                                    symbol_table.insert(label_name.clone(), address);
                                }
                                Err(_) => {
                                    return Err((
                                        format!(
                                            "Label '{}' address exceeds 16 bits",
                                            label_name
                                        ),
                                        label_ptk.position,
                                    ));
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            // ==== Process of moving the data memory counter ====
            match data_camp.directive.token {
                // ==== BYTE DIRECTIVE ====
                Token::Directive(Directive::Byte) => {
                    if let DataCampArg::Multiple(ref ptk_args) = data_camp.arg {
                        for ptk_arg in ptk_args.iter() {
                            match data_memory_counter.checked_add(1) {
                                Some(new_value) => {
                                    if new_value > DATA_MEMORY_SIZE {
                                        return Err((
                                            "Data memory overflow".to_string(),
                                            ptk_arg.position,
                                        ));
                                    } else {
                                        data_memory_counter = new_value;
                                    }
                                }
                                None => {
                                    return Err((
                                        "Data memory overflow".to_string(),
                                        ptk_arg.position,
                                    ))
                                }
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }

                // ==== WORD DIRECTIVE ====
                Token::Directive(Directive::Word) => {
                    if let DataCampArg::Multiple(ref ptk_args) = data_camp.arg {
                        for ptk_arg in ptk_args.iter() {
                            match data_memory_counter.checked_add(2) {
                                Some(new_value) => {
                                    if new_value > DATA_MEMORY_SIZE {
                                        return Err((
                                            "Data memory overflow".to_string(),
                                            ptk_arg.position,
                                        ));
                                    } else {
                                        data_memory_counter = new_value;
                                    }
                                }
                                None => {
                                    return Err((
                                        "Data memory overflow".to_string(),
                                        ptk_arg.position,
                                    ))
                                }
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }

                // ==== SPACE DIRECTIVE ====
                Token::Directive(Directive::Space) => {
                    if let DataCampArg::Unique(ref ptk_arg) = data_camp.arg {
                        match ptk_arg.token {
                            Token::NumberLiteral(ref num_string) => {
                                let num = match u16_from_string(num_string.clone()) {
                                    Ok(n) => n,
                                    Err(err) => {
                                        return Err((err.to_string(), ptk_arg.position));
                                    }
                                };

                                match data_memory_counter.checked_add(num as usize) {
                                    Some(ndmv) => {
                                        if ndmv > DATA_MEMORY_SIZE {
                                            return Err((
                                                "Data memory overflow".to_string(),
                                                ptk_arg.position,
                                            ));
                                        } else {
                                            data_memory_counter = ndmv;
                                        }
                                    }
                                    None => {
                                        return Err((
                                            "Data memory overflow".to_string(),
                                            ptk_arg.position,
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        unreachable!();
                    }
                }

                // ==== STRING AND STRINGZ DIRECTIVES ====
                Token::Directive(Directive::String)
                | Token::Directive(Directive::Stringz) => {
                    if let DataCampArg::Unique(ref ptk_arg) = data_camp.arg {
                        match ptk_arg.token {
                            Token::StringLiteral(ref string_literal) => {
                                let mut string_len = string_literal.len();
                                if data_camp.directive.token == Token::Directive(Directive::Stringz)
                                {
                                    string_len += 1; // null terminator add
                                }

                                let len_u16 = match u16::try_from(string_len) {
                                    Ok(len) => len,
                                    Err(_) => {
                                        return Err((
                                            "String length exceeds 16 bits".to_string(),
                                            ptk_arg.position,
                                        ));
                                    }
                                };

                                match data_memory_counter.checked_add(len_u16 as usize) {
                                    Some(new_value) => {
                                        if new_value > DATA_MEMORY_SIZE {
                                            return Err((
                                                "Data memory overflow".to_string(),
                                                ptk_arg.position,
                                            ));
                                        } else {
                                            data_memory_counter = new_value;
                                        }
                                    }
                                    None => {
                                        return Err((
                                            "Data memory overflow".to_string(),
                                            ptk_arg.position,
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        unreachable!();
                    }
                }

                Token::Directive(Directive::Checkpoint) => {
                    if let DataCampArg::Empty = data_camp.arg {
                        // Nothing to do here
                    } else {
                        unreachable!();
                    }
                }

                _ => unreachable!(),
            }
        }

        // >>>> INSTRUCTION MEMORY <<<<
        for (instruction_index, instruction) in ast.instr_field.iter().enumerate() {
            for label_ptk in instruction.label_declarations.iter() {
                match label_ptk.token {
                    Token::LabelDeclaration(ref label_name) => {
                        if symbol_table.contains_key(label_name) {
                            return Err((
                                format!("Label '{}' already declared", label_name),
                                label_ptk.position,
                            ));
                        } else {
                            match u16::try_from(instruction_index) {
                                Ok(address) => {
                                    symbol_table.insert(label_name.clone(), address);
                                }
                                Err(_) => {
                                    return Err((
                                        format!(
                                            "Label '{}' address exceeds 16 bits",
                                            label_name
                                        ),
                                        label_ptk.position,
                                    ));
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(symbol_table)
    }
}
