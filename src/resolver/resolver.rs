use std::collections::HashMap;

use crate::core::*;
use crate::math::*;
use crate::parser::*;

/// Represents the symbol table produced during assembly processing.
///
/// This type maps symbol names (such as labels or named constants defined
/// in the source program) to their resolved 16-bit addresses or values.
///
/// # Semantics
/// - The key (`String`) is the symbolic identifier as written in the source.
/// - The value (`u16`) is the resolved address or constant value after
///   parsing and resolution.
///
/// # Usage
/// The symbol table is typically returned after assembling a program and can be
/// used for debugging, introspection, or tooling that needs to correlate
/// symbolic names with their final numeric locations.
pub type StarSymbolTable = HashMap<String, u16>;

pub fn resolve(ast: &mut StarAst) -> Result<StarSymbolTable, (String, StarPosition)> {
    resolve_space(ast);
    match ast.get_symbol_table() {
        Ok(symbol_table) => {
            if let Some(err) = resolve_pseudo_instructions(ast, &symbol_table) {
                return Err(err);
            }
            return Ok(symbol_table);
        }
        Err(e) => return Err(e),
    }
}

fn resolve_space(ast: &mut StarAst) {
    let mut instr_field_len = ast.instr_section.len();
    let mut instr_counter: usize = 0;

    while instr_counter < instr_field_len {
        let instr_camp = match ast.instr_section.get(instr_counter) {
            Some(camp) => camp,
            None => break,
        };

        let zero_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::Zero),
            position: instr_camp.instruction.position,
        };

        let nope_camp = StarInstrCamp {
            label_declarations: Vec::new(),
            instruction: StarPositionedToken {
                token: StarToken::Instruction(StarInstruction::Add),
                position: instr_camp.instruction.position,
            },
            sequence: StarSequence::Three(zero_reg.clone(), zero_reg.clone(), zero_reg.clone()),
        };

        match instr_camp.instruction.token.clone() {
            StarToken::PseudoInstruction(pseudo_instruction) => {
                match pseudo_instruction {
                    // ==== +0 ====
                    StarPseudoInstruction::Nope
                    | StarPseudoInstruction::Move
                    | StarPseudoInstruction::Jr
                    | StarPseudoInstruction::Ret => {}

                    // ==== +1 ====
                    StarPseudoInstruction::Li
                    | StarPseudoInstruction::La
                    | StarPseudoInstruction::Mul
                    | StarPseudoInstruction::Div
                    | StarPseudoInstruction::Mod => {
                        ast.instr_section
                            .insert(instr_counter + 1, nope_camp.clone());
                    }

                    // ==== +2 ====
                    StarPseudoInstruction::Swap
                    | StarPseudoInstruction::Addi
                    | StarPseudoInstruction::Subi
                    | StarPseudoInstruction::Andi
                    | StarPseudoInstruction::Ori
                    | StarPseudoInstruction::Xori
                    | StarPseudoInstruction::Shli
                    | StarPseudoInstruction::Shri
                    | StarPseudoInstruction::Beqa
                    | StarPseudoInstruction::Bneqa
                    | StarPseudoInstruction::Blta
                    | StarPseudoInstruction::Bgta
                    | StarPseudoInstruction::Bltua
                    | StarPseudoInstruction::Bgtua
                    | StarPseudoInstruction::Inc
                    | StarPseudoInstruction::Dec
                    | StarPseudoInstruction::Ja => {
                        for _ in 0..2 {
                            ast.instr_section
                                .insert(instr_counter + 1, nope_camp.clone());
                        }
                    }

                    // ==== +3 ====
                    StarPseudoInstruction::Sb
                    | StarPseudoInstruction::Muli
                    | StarPseudoInstruction::Divi
                    | StarPseudoInstruction::Modi => {
                        for _ in 0..3 {
                            ast.instr_section
                                .insert(instr_counter + 1, nope_camp.clone());
                        }
                    }

                    // ==== +4 ====
                    StarPseudoInstruction::Lb => {
                        for _ in 0..4 {
                            ast.instr_section
                                .insert(instr_counter + 1, nope_camp.clone());
                        }
                    }

                    // ==== +7 ====
                    StarPseudoInstruction::Lw | StarPseudoInstruction::Sw => {
                        for _ in 0..7 {
                            ast.instr_section
                                .insert(instr_counter + 1, nope_camp.clone());
                        }
                    }
                }

                instr_counter += 1;
                instr_field_len = ast.instr_section.len();
                continue;
            }
            _ => {
                instr_counter += 1;
                instr_field_len = ast.instr_section.len();
                continue;
            }
        }
    }
}

fn resolve_pseudo_instructions(
    ast: &mut StarAst,
    symbol_table: &StarSymbolTable,
) -> Option<(String, StarPosition)> {
    let mut instr_counter: usize = 0;

    while instr_counter < ast.instr_section.len() {
        let instr_camp = match ast.instr_section.get_mut(instr_counter) {
            Some(camp) => camp,
            None => break,
        };

        let zero_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::Zero),
            position: instr_camp.instruction.position,
        };
        let low_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::Low),
            position: instr_camp.instruction.position,
        };
        let high_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::High),
            position: instr_camp.instruction.position,
        };

        let aux1_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::Aux1),
            position: instr_camp.instruction.position,
        };
        let aux2_reg = StarPositionedToken {
            token: StarToken::GeneralRegister(StarGeneralRegister::Aux2),
            position: instr_camp.instruction.position,
        };

        match instr_camp.instruction.token.clone() {
            StarToken::PseudoInstruction(pseudo_instruction) => {
                match pseudo_instruction {
                    // >>>> 1 <<<<
                    // ==== NOPE ====
                    StarPseudoInstruction::Nope => {
                        instr_camp.instruction.token = StarToken::Instruction(StarInstruction::Add);
                        instr_camp.sequence = StarSequence::Three(
                            zero_reg.clone(),
                            zero_reg.clone(),
                            zero_reg.clone(),
                        )
                    }

                    // ==== MOVE ====
                    StarPseudoInstruction::Move => {
                        instr_camp.instruction.token = StarToken::Instruction(StarInstruction::Add);
                        if let StarSequence::Two(arg1, arg2) = instr_camp.sequence.clone() {
                            instr_camp.sequence =
                                StarSequence::Three(arg1.clone(), zero_reg.clone(), arg2.clone())
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== JUMP RELATIVE ====
                    StarPseudoInstruction::Jr => {
                        instr_camp.instruction.token =
                            StarToken::Instruction(StarInstruction::Beqr);
                        if let StarSequence::One(arg) = instr_camp.sequence.clone() {
                            instr_camp.sequence = StarSequence::Three(
                                zero_reg.clone(),
                                zero_reg.clone(),
                                arg.clone(),
                            );
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== RETURN ====
                    StarPseudoInstruction::Ret => {
                        instr_camp.instruction.token = StarToken::Instruction(StarInstruction::J);
                        instr_camp.sequence = StarSequence::One(StarPositionedToken {
                            token: StarToken::GeneralRegister(StarGeneralRegister::ReturnAddress),
                            position: instr_camp.instruction.position.clone(),
                        });
                    }

                    // >>>> 2 <<<<
                    // ==== LOAD IMMEDIATE ====
                    StarPseudoInstruction::Li => {
                        if let StarSequence::Two(reg_ptk, imm_ptk) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(imm_string) = imm_ptk.token.clone() {
                                let num = match u16_from_string(imm_string) {
                                    Ok(n) => n,
                                    Err(e) => {
                                        return Some((e.to_string(), imm_ptk.position.clone()));
                                    }
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        reg_ptk.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: imm_ptk.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        reg_ptk.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: imm_ptk.position.clone(),
                                        },
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== LOAD ADDRESS ====
                    StarPseudoInstruction::La => {
                        if let StarSequence::Two(reg_ptk, address_ptk) = instr_camp.sequence.clone()
                        {
                            if let StarToken::Identifier(address_string) = address_ptk.token.clone()
                            {
                                let num = match symbol_table.get(&address_string) {
                                    Some(n) => *n,
                                    None => {
                                        return Some((
                                            "Address not found".to_string(),
                                            address_ptk.position.clone(),
                                        ));
                                    }
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        reg_ptk.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: address_ptk.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        reg_ptk.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: address_ptk.position.clone(),
                                        },
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== MULTIPLY ====
                    StarPseudoInstruction::Mul => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            let new_camp_1 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Mulhl),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Two(arg2.clone(), arg3.clone()),
                            };

                            let new_camp_2 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg1.clone(),
                                    zero_reg.clone(),
                                    low_reg.clone(),
                                ),
                            };

                            ast.instr_section[instr_counter] = new_camp_1;
                            ast.instr_section[instr_counter + 1] = new_camp_2;
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== DIVIDE ====
                    StarPseudoInstruction::Div => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            let new_camp_1 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Divhl),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Two(arg2.clone(), arg3.clone()),
                            };

                            let new_camp_2 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg1.clone(),
                                    zero_reg.clone(),
                                    low_reg.clone(),
                                ),
                            };

                            ast.instr_section[instr_counter] = new_camp_1;
                            ast.instr_section[instr_counter + 1] = new_camp_2;
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== MODULO ====
                    StarPseudoInstruction::Mod => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            let new_camp_1 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Divhl),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Two(arg2.clone(), arg3.clone()),
                            };

                            let new_camp_2 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg1.clone(),
                                    zero_reg.clone(),
                                    high_reg.clone(),
                                ),
                            };

                            ast.instr_section[instr_counter] = new_camp_1;
                            ast.instr_section[instr_counter + 1] = new_camp_2;
                        } else {
                            unreachable!()
                        }
                    }

                    // >>>> 3 <<<<
                    // ==== SWAP ====
                    StarPseudoInstruction::Swap => {
                        if let StarSequence::Two(arg1, arg2) = instr_camp.sequence.clone() {
                            let new_camp_1 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    aux1_reg.clone(),
                                    zero_reg.clone(),
                                    arg1.clone(),
                                ),
                            };

                            let new_camp_2 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg1.clone(),
                                    zero_reg.clone(),
                                    arg2.clone(),
                                ),
                            };

                            let new_camp_3 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Add),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg2.clone(),
                                    zero_reg.clone(),
                                    aux1_reg.clone(),
                                ),
                            };

                            ast.instr_section[instr_counter] = new_camp_1;
                            ast.instr_section[instr_counter + 1] = new_camp_2;
                            ast.instr_section[instr_counter + 2] = new_camp_3;
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== OPERATIONS IMMEDIATE ====
                    StarPseudoInstruction::Addi
                    | StarPseudoInstruction::Subi
                    | StarPseudoInstruction::Andi
                    | StarPseudoInstruction::Ori
                    | StarPseudoInstruction::Xori
                    | StarPseudoInstruction::Shli
                    | StarPseudoInstruction::Shri => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(arg3_string) = arg3.token.clone() {
                                let num = match u16_from_string(arg3_string) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let operation = match instr_camp.instruction.token {
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Addi) => {
                                        StarInstruction::Add
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Subi) => {
                                        StarInstruction::Sub
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Andi) => {
                                        StarInstruction::And
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Ori) => {
                                        StarInstruction::Or
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Xori) => {
                                        StarInstruction::Xor
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Shli) => {
                                        StarInstruction::Shl
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Shri) => {
                                        StarInstruction::Shr
                                    }
                                    _ => unreachable!(),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(operation),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        arg1.clone(),
                                        arg2.clone(),
                                        aux1_reg.clone(),
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== INC DEC ====
                    StarPseudoInstruction::Inc | StarPseudoInstruction::Dec => {
                        if let StarSequence::One(arg) = instr_camp.sequence.clone() {
                            let new_camp_1 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Lli),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Two(
                                    aux1_reg.clone(),
                                    StarPositionedToken {
                                        token: StarToken::NumberLiteral("0x01".to_string()),
                                        position: arg.position.clone(),
                                    },
                                ),
                            };

                            let new_camp_2 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(StarInstruction::Lai),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Two(
                                    aux1_reg.clone(),
                                    StarPositionedToken {
                                        token: StarToken::NumberLiteral("0x00".to_string()),
                                        position: arg.position.clone(),
                                    },
                                ),
                            };

                            let op = match instr_camp.instruction.token {
                                StarToken::PseudoInstruction(StarPseudoInstruction::Inc) => {
                                    StarInstruction::Add
                                }
                                StarToken::PseudoInstruction(StarPseudoInstruction::Dec) => {
                                    StarInstruction::Sub
                                }
                                _ => unreachable!(),
                            };

                            let new_camp_3 = StarInstrCamp {
                                label_declarations: Vec::new(),
                                instruction: StarPositionedToken {
                                    token: StarToken::Instruction(op),
                                    position: instr_camp.instruction.position.clone(),
                                },
                                sequence: StarSequence::Three(
                                    arg.clone(),
                                    arg.clone(),
                                    aux1_reg.clone(),
                                ),
                            };

                            ast.instr_section[instr_counter] = new_camp_1;
                            ast.instr_section[instr_counter + 1] = new_camp_2;
                            ast.instr_section[instr_counter + 2] = new_camp_3;
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== JUMP ABSOLUTE ====
                    StarPseudoInstruction::Ja => {
                        if let StarSequence::One(arg) = instr_camp.sequence.clone() {
                            if let StarToken::Identifier(label) = arg.token.clone() {
                                let address = match symbol_table.get(&label) {
                                    Some(addr) => *addr,
                                    None => {
                                        return Some((
                                            "Label not found".to_string(),
                                            arg.position.clone(),
                                        ));
                                    }
                                };

                                let (address_low, address_high) = split_u16_to_strings(address);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(address_low),
                                            position: arg.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(address_high),
                                            position: arg.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::J),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::One(aux1_reg.clone()),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }

                    // >>>> 6 <<<<
                    // ==== STORE BYTE ====
                    StarPseudoInstruction::Sb => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            // sb $rd, $rs[offset]
                            if let StarToken::NumberLiteral(offset_str) = arg3.token.clone() {
                                let offset = match u16_from_string(offset_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (offset_low, offset_high) = split_u16_to_strings(offset);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                // add $aux1, $aux1, $rs
                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        aux1_reg.clone(),
                                        aux1_reg.clone(),
                                        arg2.clone(),
                                    ),
                                };

                                // slb $rd, $aux1
                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Slb),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg1.clone(), aux1_reg.clone()),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== MULTIPLY IMMEDIATE ====
                    StarPseudoInstruction::Muli => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(imm_str) = arg3.token.clone() {
                                let num = match u16_from_string(imm_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Mulhl),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg2.clone(), aux1_reg.clone()),
                                };

                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        arg1.clone(),
                                        zero_reg.clone(),
                                        low_reg.clone(),
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== DIVIDE IMMEDIATE ====
                    StarPseudoInstruction::Divi => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(imm_str) = arg3.token.clone() {
                                let num = match u16_from_string(imm_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Divhl),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg2.clone(), aux1_reg.clone()),
                                };

                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        arg1.clone(),
                                        zero_reg.clone(),
                                        low_reg.clone(),
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== MODULO IMMEDIATE ====
                    StarPseudoInstruction::Modi => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(imm_str) = arg3.token.clone() {
                                let num = match u16_from_string(imm_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (num_low, num_high) = split_u16_to_strings(num);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(num_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Divhl),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg2.clone(), aux1_reg.clone()),
                                };

                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        arg1.clone(),
                                        zero_reg.clone(),
                                        high_reg.clone(),
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== LOAD BYTE (5 instructions) ====
                    StarPseudoInstruction::Lb => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            // lb $rd, $rs[offset]
                            if let StarToken::NumberLiteral(offset_str) = arg3.token.clone() {
                                let offset = match u16_from_string(offset_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (offset_low, offset_high) = split_u16_to_strings(offset);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };
                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                // add $aux1, $aux1, $rs
                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        aux1_reg.clone(),
                                        aux1_reg.clone(),
                                        arg2.clone(),
                                    ),
                                };

                                // llb $rd, $aux1
                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Llb),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg1.clone(), aux1_reg.clone()),
                                };

                                // xlb $rd, $rd
                                let new_camp_5 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Xlb),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg1.clone(), arg1.clone()),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                                ast.instr_section[instr_counter + 4] = new_camp_5;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // ==== BRANCH INSTRUCTIONS (3 instructions)====
                    StarPseudoInstruction::Beqa
                    | StarPseudoInstruction::Bneqa
                    | StarPseudoInstruction::Blta
                    | StarPseudoInstruction::Bgta
                    | StarPseudoInstruction::Bltua
                    | StarPseudoInstruction::Bgtua => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::Identifier(label) = arg3.token.clone() {
                                let address = match symbol_table.get(&label) {
                                    Some(addr) => *addr,
                                    None => {
                                        return Some((
                                            "Label not found".to_string(),
                                            arg3.position.clone(),
                                        ));
                                    }
                                };

                                let relative_target_address =
                                    address.wrapping_sub(instr_counter as u16).wrapping_sub(2);

                                let (relative_target_address_low, relative_target_address_high) =
                                    split_u16_to_strings(relative_target_address);

                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(
                                                relative_target_address_low,
                                            ),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(
                                                relative_target_address_high,
                                            ),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let branch_instr = match instr_camp.instruction.token {
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Beqa) => {
                                        StarInstruction::Beqr
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Bneqa) => {
                                        StarInstruction::Bneqr
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Blta) => {
                                        StarInstruction::Bltr
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Bgta) => {
                                        StarInstruction::Bgtr
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Bltua) => {
                                        StarInstruction::Bltur
                                    }
                                    StarToken::PseudoInstruction(StarPseudoInstruction::Bgtua) => {
                                        StarInstruction::Bgtur
                                    }
                                    _ => unreachable!(),
                                };

                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(branch_instr),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        arg1.clone(),
                                        arg2.clone(),
                                        aux1_reg.clone(),
                                    ),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    // >>>> 10 <<<<
                    // ==== LOAD WORD and STORE WORD ====
                    StarPseudoInstruction::Lw | StarPseudoInstruction::Sw => {
                        if let StarSequence::Three(arg1, arg2, arg3) = instr_camp.sequence.clone() {
                            if let StarToken::NumberLiteral(offset_str) = arg3.token.clone() {
                                // lw $rd, $rs[offset]
                                let offset = match u16_from_string(offset_str) {
                                    Ok(n) => n,
                                    Err(e) => return Some((e.to_string(), arg3.position.clone())),
                                };

                                let (offset_low, offset_high) = split_u16_to_strings(offset);

                                // la $aux1, offset
                                let new_camp_1 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_low),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_2 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux1_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral(offset_high),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                // add $aux1, $aux1, $rs
                                let new_camp_3 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        aux1_reg.clone(),
                                        aux1_reg.clone(),
                                        arg2.clone(),
                                    ),
                                };

                                // lab $rd, $aux1 or sab $rd, $aux1
                                let new_camp_4 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(
                                            match instr_camp.instruction.token {
                                                StarToken::PseudoInstruction(
                                                    StarPseudoInstruction::Lw,
                                                ) => StarInstruction::Lab,
                                                StarToken::PseudoInstruction(
                                                    StarPseudoInstruction::Sw,
                                                ) => StarInstruction::Sab,
                                                _ => unreachable!(),
                                            },
                                        ),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg1.clone(), aux1_reg.clone()),
                                };

                                // la $aux2, 0x0001
                                let new_camp_5 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lli),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux2_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral("0x01".to_string()),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                let new_camp_6 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Lai),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(
                                        aux2_reg.clone(),
                                        StarPositionedToken {
                                            token: StarToken::NumberLiteral("0x00".to_string()),
                                            position: arg3.position.clone(),
                                        },
                                    ),
                                };

                                // add $aux1, $aux1, $aux2
                                let new_camp_7 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(StarInstruction::Add),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Three(
                                        aux1_reg.clone(),
                                        aux1_reg.clone(),
                                        aux2_reg.clone(),
                                    ),
                                };

                                // llb $rd, $aux1 or slb $rd, $aux1
                                let new_camp_8 = StarInstrCamp {
                                    label_declarations: Vec::new(),
                                    instruction: StarPositionedToken {
                                        token: StarToken::Instruction(
                                            match instr_camp.instruction.token {
                                                StarToken::PseudoInstruction(
                                                    StarPseudoInstruction::Lw,
                                                ) => StarInstruction::Llb,
                                                StarToken::PseudoInstruction(
                                                    StarPseudoInstruction::Sw,
                                                ) => StarInstruction::Slb,
                                                _ => unreachable!(),
                                            },
                                        ),
                                        position: instr_camp.instruction.position.clone(),
                                    },
                                    sequence: StarSequence::Two(arg1.clone(), aux1_reg.clone()),
                                };

                                ast.instr_section[instr_counter] = new_camp_1;
                                ast.instr_section[instr_counter + 1] = new_camp_2;
                                ast.instr_section[instr_counter + 2] = new_camp_3;
                                ast.instr_section[instr_counter + 3] = new_camp_4;
                                ast.instr_section[instr_counter + 4] = new_camp_5;
                                ast.instr_section[instr_counter + 5] = new_camp_6;
                                ast.instr_section[instr_counter + 6] = new_camp_7;
                                ast.instr_section[instr_counter + 7] = new_camp_8;
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                }

                instr_counter += 1;
                continue;
            }
            _ => {
                instr_counter += 1;
                continue;
            }
        }
    }

    None
}
