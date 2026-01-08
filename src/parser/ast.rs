use std::collections::HashMap;

use crate::core::*;
use crate::math::*;
use crate::parser::sequence::*;
use crate::resolver::*;

pub type StarDataSection = Vec<StarDataCamp>;

pub type StarInstrSection = Vec<StarInstrCamp>;

#[derive(Debug, Clone)]
pub enum StarCustomSection {
    Data(StarDataSection),
    Instr(StarInstrSection),
}

pub type StarCustomSections = HashMap<String, StarCustomSection>;


#[derive(Debug, Clone)]
pub struct StarInstrCamp {
    pub label_declarations: Vec<StarPositionedToken>,
    pub instruction: StarPositionedToken,
    pub sequence: StarSequence,
}

#[derive(Debug, Clone)]
pub struct StarDataCamp {
    pub label_declarations: Vec<StarPositionedToken>,
    pub directive: StarPositionedToken,
    pub arg: StarDataCampArg,
}

#[derive(Debug, Clone)]
pub enum StarDataCampArg {
    Empty,
    Unique(StarPositionedToken),
    Multiple(Vec<StarPositionedToken>),
}






#[derive(Debug, Clone)]
pub struct StarAst {
    pub data_section: StarDataSection,
    pub instr_section: StarInstrSection,
    pub custom_sections: StarCustomSections
}

impl StarAst {
    pub fn new(custom_sections: &Vec<StarCustomSectionDefinition>) -> Self {
        let mut ast = StarAst {
            data_section: Vec::new(),
            instr_section: Vec::new(),
            custom_sections: HashMap::new(),
        };

        // cria todas as custom sections vazias no AST
        for csd in custom_sections.iter() {
            let section = match csd.section_type {
                StarSectionParsingType::Data => StarCustomSection::Data(Vec::new()),
                StarSectionParsingType::Instr => StarCustomSection::Instr(Vec::new()),
            };

            ast.custom_sections.insert(csd.name.clone(), section);
        }

        ast
    }

    pub fn get_symbol_table(&mut self) -> Result<StarSymbolTable, (String, StarPosition)> {
        let mut symbol_table: StarSymbolTable = HashMap::new();

        // >>>> DATA MEMORY <<<<
        let mut data_memory_counter: usize = 0;

        for data_camp in self.data_section.iter() {
            // ==== Process of collecting labels ====
            for label_ptk in data_camp.label_declarations.iter() {
                match label_ptk.token {
                    StarToken::LabelDeclaration(ref label_name) => {
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
                StarToken::StarDirective(StarDirective::Byte) => {
                    if let StarDataCampArg::Multiple(ref ptk_args) = data_camp.arg {
                        for ptk_arg in ptk_args.iter() {
                            match data_memory_counter.checked_add(1) {
                                Some(new_value) => {
                                    if new_value > STAR_MEMORY_64KB {
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
                StarToken::StarDirective(StarDirective::Word) => {
                    if let StarDataCampArg::Multiple(ref ptk_args) = data_camp.arg {
                        for ptk_arg in ptk_args.iter() {
                            match data_memory_counter.checked_add(2) {
                                Some(new_value) => {
                                    if new_value > STAR_MEMORY_64KB {
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
                StarToken::StarDirective(StarDirective::Space) => {
                    if let StarDataCampArg::Unique(ref ptk_arg) = data_camp.arg {
                        match ptk_arg.token {
                            StarToken::NumberLiteral(ref num_string) => {
                                let num = match u16_from_string(num_string.clone()) {
                                    Ok(n) => n,
                                    Err(err) => {
                                        return Err((err.to_string(), ptk_arg.position));
                                    }
                                };

                                match data_memory_counter.checked_add(num as usize) {
                                    Some(ndmv) => {
                                        if ndmv > STAR_MEMORY_64KB {
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
                StarToken::StarDirective(StarDirective::String)
                | StarToken::StarDirective(StarDirective::Stringz) => {
                    if let StarDataCampArg::Unique(ref ptk_arg) = data_camp.arg {
                        match ptk_arg.token {
                            StarToken::StringLiteral(ref string_literal) => {
                                let mut string_len = string_literal.len();
                                if data_camp.directive.token == StarToken::StarDirective(StarDirective::Stringz)
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
                                        if new_value > STAR_MEMORY_64KB {
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

                StarToken::StarDirective(StarDirective::Checkpoint) => {
                    if let StarDataCampArg::Empty = data_camp.arg {
                        // Nothing to do here
                    } else {
                        unreachable!();
                    }
                }

                _ => unreachable!(),
            }
        }

        // >>>> INSTRUCTION MEMORY <<<<
        for (instruction_index, instruction) in self.instr_section.iter().enumerate() {
            for label_ptk in instruction.label_declarations.iter() {
                match label_ptk.token {
                    StarToken::LabelDeclaration(ref label_name) => {
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
