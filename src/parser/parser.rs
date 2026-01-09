use super::ast::*;
use super::reader::*;
use super::sequence::*;
use crate::core::*;

pub fn parse(ptokens: &Vec<StarPositionedToken>, custom_sections: &Vec<StarCustomSectionDefinition>) -> Result<StarAst, (String, StarPosition)> {
    let mut ast = StarAst::new(custom_sections);

    let mut section_directive: StarDirective = StarDirective::Instr;
    let mut section_type = StarSectionParsingType::Instr;
    let mut section_name: String = String::new();

    let mut label_declaration_accumulator: Vec<StarPositionedToken> = Vec::new();

    let mut ptk_counter = 0;
    while ptk_counter < ptokens.len() {
        let ptk = match ptokens.get(ptk_counter) {
            Some(ptk) => ptk,
            None => unreachable!(),
        };

        match ptk.token.clone() {
            // ==== Detect the current section ====
            StarToken::StarDirective(StarDirective::Data) => {
                section_directive = StarDirective::Data;
                section_type = StarSectionParsingType::Data;
                section_name = ".data".to_string();

                ptk_counter += 1;
                continue;
            }
            StarToken::StarDirective(StarDirective::Instr) => {
                section_directive = StarDirective::Instr;

                section_type = StarSectionParsingType::Instr;
                section_name = ".instr".to_string();

                ptk_counter += 1;
                continue;
            }
            StarToken::StarDirective(StarDirective::Custom(custom_section_string)) => {
                section_directive = StarDirective::Custom(custom_section_string.clone());

                let custom_section_definition_option = custom_sections
                    .iter()
                    .find(|&csd| csd.name == custom_section_string);

                (section_type, section_name) = match custom_section_definition_option {
                    Some(csd) => (csd.section_type.clone(), csd.name.clone()),
                    None => {
                        return Err((
                            format!("Directive not defined: {}", custom_section_string),
                            ptk.position.clone(),
                        ));
                    }
                };

                ptk_counter += 1;
                continue;
            }

            // ==== Caso não for Star Section ====
            _ => {
                match section_type {
                    // ==== Data Analysis ====
                    StarSectionParsingType::Data => {
                        let data_section: &mut StarDataSection = match section_directive {
                            StarDirective::Data => &mut ast.data_section,

                            StarDirective::Custom(ref custom_directive_string) => ast
                                .custom_sections
                                .iter_mut()
                                .find_map(|(csd_name, cs)| {
                                    if *csd_name == section_name {
                                        match cs {
                                            StarCustomSection::Data(custom_data_section) => {
                                                Some(custom_data_section)
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .ok_or_else(|| {
                                    (
                                        format!(
                                            "Invalid custom section definition: {}",
                                            custom_directive_string
                                        ),
                                        ptk.position.clone(),
                                    )
                                })?,

                            StarDirective::Instr => {
                                return Err((
                                    format!(
                                        "Invalid section type for instruction section: {}",
                                        section_name
                                    ),
                                    ptk.position.clone(),
                                ));
                            }

                            _ => {
                                unreachable!()
                            }
                        };

                        match ptk.token {
                            StarToken::LabelDeclaration(_) => {
                                label_declaration_accumulator.push(ptk.clone());
                                ptk_counter += 1;
                                continue;
                            }

                            StarToken::StarDirective(StarDirective::Byte)
                            | StarToken::StarDirective(StarDirective::Word) => {
                                let data: Vec<StarPositionedToken> =
                                    read_comma_separated_numbers(ptokens, ptk_counter + 1);
                                    
                                let data_len = data.len();
                                if data_len == 0 {
                                    return Err((
                                        "StarDirective expects at least one number".to_string(),
                                        ptk.position.clone(),
                                    ));
                                }

                                data_section.push(StarDataCamp {
                                    label_declarations: label_declaration_accumulator.clone(),
                                    directive: ptk.clone(),
                                    arg: StarDataCampArg::Multiple(data.clone()),
                                });

                                ptk_counter += data_len * 2;
                                label_declaration_accumulator.clear();
                                continue;
                            }

                            StarToken::StarDirective(StarDirective::Space) => {
                                match ptokens.get(ptk_counter + 1) {
                                    Some(next_ptk) => {
                                        if let StarToken::NumberLiteral(_) = next_ptk.token {
                                            data_section.push(StarDataCamp {
                                                label_declarations: label_declaration_accumulator
                                                    .clone(),
                                                directive: ptk.clone(),
                                                arg: StarDataCampArg::Unique(next_ptk.clone()),
                                            });
                                            ptk_counter += 2;
                                            label_declaration_accumulator.clear();
                                            continue;
                                        } else {
                                            return Err((
                                                "StarDirective expects a number".to_string(),
                                                ptk.position.clone(),
                                            ));
                                        }
                                    }
                                    None => {
                                        return Err((
                                            "StarDirective expects a number".to_string(),
                                            ptk.position.clone(),
                                        ));
                                    }
                                };
                            }

                            StarToken::StarDirective(StarDirective::String)
                            | StarToken::StarDirective(StarDirective::Stringz) => {
                                match ptokens.get(ptk_counter + 1) {
                                    Some(next_ptk) => {
                                        if let StarToken::StringLiteral(_) = next_ptk.token {
                                            data_section.push(StarDataCamp {
                                                label_declarations: label_declaration_accumulator
                                                    .clone(),
                                                directive: ptk.clone(),
                                                arg: StarDataCampArg::Unique(next_ptk.clone()),
                                            });
                                            ptk_counter += 2;
                                            label_declaration_accumulator.clear();
                                            continue;
                                        } else {
                                            return Err((
                                                "StarDirective expects a literal string"
                                                    .to_string(),
                                                ptk.position.clone(),
                                            ));
                                        }
                                    }
                                    None => {
                                        return Err((
                                            "StarDirective expects a literal string".to_string(),
                                            ptk.position.clone(),
                                        ));
                                    }
                                };
                            }

                            StarToken::StarDirective(StarDirective::Checkpoint) => {
                                data_section.push(StarDataCamp {
                                    label_declarations: label_declaration_accumulator.clone(),
                                    directive: ptk.clone(),
                                    arg: StarDataCampArg::Empty,
                                });
                                ptk_counter += 1;
                                label_declaration_accumulator.clear();
                                continue;
                            }

                            _ => {
                                return Err((
                                    "Invalid expression in data section".to_string(),
                                    ptk.position.clone(),
                                ));
                            }
                        }
                    }

                    // ==== Instruction Analysis ====
                    StarSectionParsingType::Instr => {
                        let instr_section: &mut StarInstrSection = match section_directive {
                            StarDirective::Instr => &mut ast.instr_section,

                            StarDirective::Custom(ref custom_directive_string) => ast
                                .custom_sections
                                .iter_mut()
                                .find_map(|(csd_name, cs)| {
                                    if *csd_name == section_name {
                                        match cs {
                                            StarCustomSection::Instr(custom_instr_section) => {
                                                Some(custom_instr_section)
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .ok_or_else(|| {
                                    (
                                        format!(
                                            "Invalid custom section definition: {}",
                                            custom_directive_string
                                        ),
                                        ptk.position.clone(),
                                    )
                                })?,

                            StarDirective::Data => {
                                return Err((
                                    format!(
                                        "Invalid section type for data section: {}",
                                        section_name
                                    ),
                                    ptk.position.clone(),
                                ));
                            }

                            _ => {
                                unreachable!();
                            }
                        };

                        match ptk.token.clone() {
                            // ==== Label Declaration ====
                            StarToken::LabelDeclaration(_) => {
                                label_declaration_accumulator.push(ptk.clone());
                                ptk_counter += 1;
                                continue;
                            }

                            StarToken::Instruction(instr) => {
                                match instr {
                                    // ==== READ NONE ====
                                    StarInstruction::Mcall => {
                                        instr_section.push(StarInstrCamp {
                                            label_declarations: label_declaration_accumulator
                                                .clone(),
                                            instruction: ptk.clone(),
                                            sequence: StarSequence::Zero,
                                        });
                                        ptk_counter += 1;
                                        label_declaration_accumulator.clear();
                                        continue;
                                    }

                                    // ==== READ REG ====
                                    StarInstruction::J => {
                                        match read_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 2;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG ====
                                    StarInstruction::Xlb
                                    | StarInstruction::Lab
                                    | StarInstruction::Llb
                                    | StarInstruction::Sab
                                    | StarInstruction::Slb
                                    | StarInstruction::Mulhl
                                    | StarInstruction::Divhl
                                    | StarInstruction::Muluhl
                                    | StarInstruction::Divuhl
                                    | StarInstruction::Not => {
                                        match read_r_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 4;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG IMM ====
                                    StarInstruction::Lai | StarInstruction::Lli => {
                                        match read_r_n_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 4;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG REG ====
                                    StarInstruction::Add
                                    | StarInstruction::Sub
                                    | StarInstruction::And
                                    | StarInstruction::Or
                                    | StarInstruction::Xor
                                    | StarInstruction::Shl
                                    | StarInstruction::Shr
                                    | StarInstruction::Beqr
                                    | StarInstruction::Bneqr
                                    | StarInstruction::Bgtr
                                    | StarInstruction::Bltr
                                    | StarInstruction::Bgtur
                                    | StarInstruction::Bltur => {
                                        match read_r_r_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(three_seq) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence: three_seq,
                                                });
                                                ptk_counter += 6;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }
                                }
                            }

                            StarToken::PseudoInstruction(pseudo_instr) => {
                                match pseudo_instr {
                                    StarPseudoInstruction::Nope | StarPseudoInstruction::Ret => {
                                        instr_section.push(StarInstrCamp {
                                            label_declarations: label_declaration_accumulator
                                                .clone(),
                                            instruction: ptk.clone(),
                                            sequence: StarSequence::Zero,
                                        });
                                        ptk_counter += 1;
                                        label_declaration_accumulator.clear();
                                        continue;
                                    }

                                    StarPseudoInstruction::Inc
                                    | StarPseudoInstruction::Dec
                                    | StarPseudoInstruction::Jr => {
                                        match read_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 2;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ IDENTIFIER ====
                                    StarPseudoInstruction::Ja => {
                                        match read_id_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 2;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG ====
                                    StarPseudoInstruction::Move | StarPseudoInstruction::Swap => {
                                        match read_r_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 4;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    StarPseudoInstruction::Li => {
                                        match read_r_n_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 4;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG IDENTIFIER ====
                                    StarPseudoInstruction::La => {
                                        match read_r_id_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 4;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG REG ====
                                    StarPseudoInstruction::Mul
                                    | StarPseudoInstruction::Div
                                    | StarPseudoInstruction::Mod => {
                                        match read_r_r_r_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(three_seq) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence: three_seq,
                                                });
                                                ptk_counter += 6;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG NUMBER ====
                                    StarPseudoInstruction::Addi
                                    | StarPseudoInstruction::Subi
                                    | StarPseudoInstruction::Andi
                                    | StarPseudoInstruction::Ori
                                    | StarPseudoInstruction::Xori
                                    | StarPseudoInstruction::Shli
                                    | StarPseudoInstruction::Shri
                                    | StarPseudoInstruction::Muli
                                    | StarPseudoInstruction::Divi
                                    | StarPseudoInstruction::Modi => {
                                        match read_r_r_n_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 6;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // READ REG REG SBRACKET NUMBER SBRACKET
                                    StarPseudoInstruction::Lb
                                    | StarPseudoInstruction::Lw
                                    | StarPseudoInstruction::Sb
                                    | StarPseudoInstruction::Sw => {
                                        match read_r_r_br_n_br(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 7;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }

                                    // ==== READ REG REG IDENTIFIER ====
                                    StarPseudoInstruction::Beqa
                                    | StarPseudoInstruction::Bneqa
                                    | StarPseudoInstruction::Bgta
                                    | StarPseudoInstruction::Blta
                                    | StarPseudoInstruction::Bgtua
                                    | StarPseudoInstruction::Bltua => {
                                        match read_r_r_id_sequence(
                                            &ptokens,
                                            ptk_counter + 1,
                                            ptk.position,
                                        ) {
                                            Ok(sequence) => {
                                                instr_section.push(StarInstrCamp {
                                                    label_declarations:
                                                        label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence,
                                                });
                                                ptk_counter += 6;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            }
                                            Err((err_msg, err_pos)) => {
                                                return Err((err_msg, err_pos));
                                            }
                                        }
                                    }
                                }
                            }

                            _ => {
                                return Err((
                                    "Invalid expression in instruction section".to_string(),
                                    ptk.position.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ast)
}

fn read_comma_separated_numbers(
    ptokens: &Vec<StarPositionedToken>,
    start_index: usize,
) -> Vec<StarPositionedToken> {
    let mut numbers = Vec::new();
    let mut index = start_index;

    let Some(ptk) = ptokens.get(index) else {
        return numbers;
    };

    match ptk.token {
        StarToken::NumberLiteral(_) => {
            numbers.push(ptk.clone());
            index += 1;
        }
        _ => return numbers,
    }

    while index + 1 < ptokens.len() {
        let comma = &ptokens[index];
        let next = &ptokens[index + 1];

        match (&comma.token, &next.token) {
            (StarToken::Comma, StarToken::NumberLiteral(_)) => {
                numbers.push(next.clone());
                index += 2;
            }
            _ => break,
        }
    }

    numbers
}

