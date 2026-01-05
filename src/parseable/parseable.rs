use crate::core::*;
use super::ast::*;
use super::reader::*;
use super::sequence::*;

pub trait StarParseable {
    fn parse(&self, ptokens: &Vec<StarPositionedToken>) -> Result<Ast, (String, StarPosition)>;
    fn read_comma_separated_numbers(&self, ptokens: &Vec<StarPositionedToken>, start_index: usize) -> Vec<StarPositionedToken>;
}

impl StarParseable for Star {
    fn parse(&self, ptokens: &Vec<StarPositionedToken>) -> Result<Ast, (String, StarPosition)> {
        let mut ast = Ast {
            data_field: Vec::new(),
            instr_field: Vec::new(),
        };

        let mut field: StarDirective = StarDirective::Instr;
        let mut label_declaration_accumulator: Vec<StarPositionedToken> = Vec::new();

        let mut ptk_counter = 0;
        while ptk_counter < ptokens.len() {
            let ptk = match ptokens.get(ptk_counter) {
                Some(ptk) => ptk,
                None => unreachable!(),
            };

            match ptk.token {
                // ==== Detect the current field ====
                StarToken::StarDirective(StarDirective::Data) => {
                    field = StarDirective::Data;
                    ptk_counter += 1;
                    continue;
                }
                StarToken::StarDirective(StarDirective::Instr) => {
                    field = StarDirective::Instr;
                    ptk_counter += 1;
                    continue;
                }

                // ==== Case not is a Data or Instr StarDirective ====
                _ => {
                    match field {
                        // ==== Data Field ====
                        StarDirective::Data => {
                            match ptk.token {
                                StarToken::LabelDeclaration(_) => {
                                    label_declaration_accumulator.push(ptk.clone());
                                    ptk_counter += 1;
                                    continue;
                                }

                                StarToken::StarDirective(StarDirective::Byte)
                                | StarToken::StarDirective(StarDirective::Word) => {
                                    let data: Vec<StarPositionedToken> =
                                        self.read_comma_separated_numbers(ptokens, ptk_counter + 1);

                                    let data_len = data.len();
                                    if data_len == 0 {
                                        return Err((
                                            "StarDirective expects at least one number".to_string(),
                                            ptk.position.clone(),
                                        ));
                                    }

                                    ast.data_field.push(
                                        DataCamp {
                                            label_declarations: label_declaration_accumulator.clone(),
                                            directive: ptk.clone(),
                                            arg: DataCampArg::Multiple(data.clone()),
                                        }
                                    );

                                    ptk_counter += data_len * 2;
                                    label_declaration_accumulator.clear();
                                    continue;
                                }

                                StarToken::StarDirective(StarDirective::Space) => {
                                    match ptokens.get(ptk_counter + 1) {
                                        Some(next_ptk) => {
                                            if let StarToken::NumberLiteral(_) = next_ptk.token {
                                                ast.data_field.push(
                                                    DataCamp {
                                                        label_declarations: label_declaration_accumulator.clone(),
                                                        directive: ptk.clone(),
                                                        arg: DataCampArg::Unique(next_ptk.clone()),
                                                    }
                                                );
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
                                                ast.data_field.push(
                                                    DataCamp {
                                                        label_declarations: label_declaration_accumulator.clone(),
                                                        directive: ptk.clone(),
                                                        arg: DataCampArg::Unique(next_ptk.clone()),
                                                    }
                                                );
                                                ptk_counter += 2;
                                                label_declaration_accumulator.clear();
                                                continue;
                                            } else {
                                                return Err((
                                                    "StarDirective expects a literal string".to_string(),
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
                                    ast.data_field.push(
                                        DataCamp {
                                            label_declarations: label_declaration_accumulator.clone(),
                                            directive: ptk.clone(),
                                            arg: DataCampArg::Empty,
                                        }
                                    );
                                    ptk_counter += 1;
                                    label_declaration_accumulator.clear();
                                    continue;
                                }

                                _ => {
                                    return Err((
                                        "Invalid expression in data field".to_string(),
                                        ptk.position.clone(),
                                    ));
                                }
                            }
                        }

                        // ==== StarInstruction Field ====
                        StarDirective::Instr => {
                            match ptk.token.clone() {
                                // ==== Label Declaration ====
                                StarToken::LabelDeclaration(_) => {
                                    label_declaration_accumulator.push(ptk.clone());
                                    ptk_counter += 1;
                                    continue;
                                }

                                StarToken::StarInstruction(instr) => {
                                    match instr {
                                        // ==== READ NONE ====
                                        StarInstruction::Mcall => {
                                            ast.instr_field.push(
                                                InstrCamp {
                                                    label_declarations: label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence: StarSequence::Zero,
                                                }
                                            );
                                            ptk_counter += 1;
                                            label_declaration_accumulator.clear();
                                            continue;
                                        }

                                        // ==== READ REG ====
                                        StarInstruction::J => {
                                            match read_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                        StarInstruction::Lai
                                        | StarInstruction::Lli => {
                                            match read_r_n_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_r_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(three_seq) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence: three_seq,
                                                        }
                                                    );
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

                                StarToken::StarPseudoInstruction(pseudo_instr) => {
                                    match pseudo_instr {
                                        StarPseudoInstruction::Nope
                                        | StarPseudoInstruction::Ret => {
                                            ast.instr_field.push(
                                                InstrCamp {
                                                    label_declarations: label_declaration_accumulator.clone(),
                                                    instruction: ptk.clone(),
                                                    sequence: StarSequence::Zero,
                                                }
                                            );
                                            ptk_counter += 1;
                                            label_declaration_accumulator.clear();
                                            continue;
                                        }

                                        StarPseudoInstruction::Inc
                                        | StarPseudoInstruction::Dec
                                        | StarPseudoInstruction::Jr => {
                                            match read_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_id_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                        StarPseudoInstruction::Move
                                        | StarPseudoInstruction::Swap => {
                                            match read_r_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_n_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_id_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_r_r_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(three_seq) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence: three_seq,
                                                        }
                                                    );
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
                                            match read_r_r_n_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_r_br_n_br(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                            match read_r_r_id_sequence(&ptokens, ptk_counter + 1, ptk.position) {
                                                Ok(sequence) => {
                                                    ast.instr_field.push(
                                                        InstrCamp {
                                                            label_declarations: label_declaration_accumulator.clone(),
                                                            instruction: ptk.clone(),
                                                            sequence,
                                                        }
                                                    );
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
                                        "Invalid expression in instruction field".to_string(),
                                        ptk.position.clone(),
                                    ));
                                }
                            }
                        }

                        _ => unreachable!(),
                    }
                }
            }
        }

        Ok(ast)
    }

    fn read_comma_separated_numbers(&self, ptokens: &Vec<StarPositionedToken>, start_index: usize) -> Vec<StarPositionedToken> {
        let mut numbers: Vec<StarPositionedToken> = Vec::new();
        let mut index: usize = start_index;

        while index < ptokens.len() {
            let ptk = match ptokens.get(index) {
                Some(ptk) => ptk,
                None => break,
            };
            match ptk.token {
                StarToken::NumberLiteral(_) => {
                    numbers.push(ptk.clone());
                    index += 1;
                }
                StarToken::Comma => {
                    index += 1;
                }
                _ => break,
            }
        }

        numbers
    }
}
