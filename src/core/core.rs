use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use crate::math::u8_from_string;
use crate::utils::*;
use crate::core::*;
use crate::generateable::*;
use crate::resolveable::*;
use crate::scannable::*;
use crate::parseable::*;

pub const DATA_MEMORY_SIZE: usize = 65536;

pub struct Star {
    pub file_table: HashMap<usize, String>,
    pub data_memory: [u8; DATA_MEMORY_SIZE],
    pub instruction_memory: Vec<u8>,
    pub position_memory: Vec<Position>,
    pub registers: Registers,

    pub interface: Option<Box<dyn Interface>>,
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
            registers: Registers::new(),
            interface: None,
        }
    }

    pub fn load_from_assembly_file(&mut self, file_path: &str) -> Result<(SymbolTable, usize), (String, Option<Position>)> {        
        
        match self.scan_file(file_path) {
            Ok(ptokens) => {
                match self.parse(&ptokens) {
                    Ok(mut ast) => {
                        match self.resolve(&mut ast) {
                            Ok(symbol_table) => {
                                match self.generate(&ast) {
                                    Ok(data_section_size) => {
                                        return Ok((symbol_table, data_section_size));
                                    }
                                    Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                                }
                            }
                            Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                        }
                    }
                    Err((e_string, e_position)) => return Err((e_string, Some(e_position))),
                }
            }
            Err(e) => return Err(e),
        }
    }

    pub fn load_from_binary(&mut self, file_path: &String) -> Option<(String, Option<Position>)> {
        use std::fs;

        let absolute_file_path: String = match fs::canonicalize(file_path) {
            Ok(path) => path.to_str().unwrap_or(file_path).to_string(),
            Err(_) => {
                return Some((
                    format!("Failed to read binary file '{}'", file_path),
                    None,
                ));
            }
        };

        self.file_table
            .insert(self.file_table.len(), absolute_file_path.clone());

        let id_option = match self.get_file_id_by_path(&absolute_file_path) {
            Some(id) => Some(id),
            None => {
                return Some((
                    format!("File '{}' not found in file table", absolute_file_path),
                    None,
                ))
            }
        };

        let file_content = match fs::read_to_string(&absolute_file_path) {
            Ok(content) => content.replace("\r", ""),
            Err(_) => {
                return Some((
                    format!("Failed to read binary file '{}'", absolute_file_path),
                    None,
                ))
            }
        };

        let mut section = ".instr".to_string();

        let mut actual_line: usize = 1;
        let mut actual_column: usize = 1;

        let mut data_memory_vector: Vec<u8> = Vec::new();

        let mut tkn_line: usize = 1;
        let mut tkn_column: usize = 1;

        let mut line_has_identation = false;

        let mut accumulator = String::new();
        let mut chars = file_content.chars().peekable();

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
                                        return Some((
                                            "Invalid binary instruction".to_string(),
                                            Some(Position::new(
                                                id_option,
                                                tkn_line,
                                                Some(tkn_column),
                                            )),
                                        ));
                                    }
                                }

                                if self.instruction_memory.len() % 2 == 0 {
                                    self.position_memory.push(Position::new(
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
                                        return Some((
                                            "Invalid binary data".to_string(),
                                            Some(Position::new(
                                                id_option,
                                                tkn_line,
                                                Some(tkn_column),
                                            )),
                                        ));
                                    }
                                }
                            }

                            _ => {
                                return Some((
                                    "Unknown section".to_string(),
                                    Some(Position::new(id_option, tkn_line, Some(tkn_column))),
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
                            return Some((
                                "Invalid binary instruction".to_string(),
                                Some(Position::new(id_option, tkn_line, Some(tkn_column))),
                            ))
                        }
                    };

                    self.instruction_memory.push(v);

                    if self.instruction_memory.len() % 2 == 0 {
                        self.position_memory.push(Position::new(
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
                            return Some((
                                "Invalid binary data".to_string(),
                                Some(Position::new(id_option, tkn_line, Some(tkn_column))),
                            ))
                        }
                    };

                    data_memory_vector.push(v);
                }

                _ => {
                    return Some((
                        "Unknown section".to_string(),
                        Some(Position::new(id_option, tkn_line, Some(tkn_column))),
                    ));
                }
            }
        }

        for (i, b) in data_memory_vector.iter().enumerate() {
            match self.data_memory.get_mut(i) {
                Some(memory_byte) => *memory_byte = *b,
                None => return Some(("Data memory out of bounds".to_string(), None)),
            }
        }

        None
    }

    pub fn save_binary(&self, file_path: &str, data_section_size: usize) -> Option<String> {
        let mut file = match File::create(file_path.to_string()) {
            Ok(f) => f,
            Err(_) => return Some("Failed to create binary file".to_string()),
        };

        if file.write_all(".instr".as_bytes()).is_err() {
            return Some("Failed to write on binary file".to_string());
        }

        for (index, instr) in self.instruction_memory.iter().enumerate() {
            let mut output = format!("{:08b} ", instr);
            if index % 2 == 0 {
                output = format!("\n{}", output);
            }

            if file.write_all(output.as_bytes()).is_err() {
                return Some("Failed to write on binary file".to_string());
            }
        }

        if file.write_all("\n.data".as_bytes()).is_err() {
            return Some("Failed to write on binary file".to_string());
        }

        for byte_index in 0..data_section_size {
            let byte = match self.data_memory.get(byte_index) {
                Some(b) => *b,
                None => return Some("Data memory out of bounds".to_string()),
            };

            let output = format!("\n{:08b} ", byte);
            if file.write_all(output.as_bytes()).is_err() {
                return Some("Failed to write on binary file".to_string());
            }
        }

        None
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

    pub fn set_interface(&mut self, interface: Box<dyn Interface>) {
        self.interface = Some(interface);
    }

    pub fn take_interface(&mut self) -> Option<Box<dyn Interface>> {
        self.interface.take()
    }

    pub fn take_interface_mut(&mut self) -> Option<&mut Box<dyn Interface>> {
        self.interface.as_mut().take()
    }
}

