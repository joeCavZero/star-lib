use std::collections::HashSet;
use std::collections::HashMap;

use crate::core::*;
use crate::utils::*;
use crate::scannable::positioned_tokens_vectorable::*;

type MacroTable = HashMap<String, (Vec<PositionedToken>, Vec<PositionedToken>)>;

pub trait Scannable {
    fn scan(&mut self, base_file_path: &String) -> Result<Vec<PositionedToken>, (String, Option<Position>)>;

    fn scan_and_resolve_processors(
        &mut self,
        including_file_path: &String,
        file_path_to_include: &String,
        file_counter: &mut u32,
        file_dependency_table: &mut HashMap<u32, HashSet<u32>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<u32>,
    ) -> Result<Vec<PositionedToken>, (String, Option<Position>)>;
}

impl Scannable for Star {
    fn scan(&mut self, base_file_path: &String) -> Result<Vec<PositionedToken>, (String, Option<Position>)> {
        let mut file_dependency_table: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut macro_table: MacroTable = HashMap::new();
        let mut once_set: HashSet<String> = HashSet::new();
        let mut file_counter: u32 = 0;
        let mut processing_stack: HashSet<u32> = HashSet::new();

        self.scan_and_resolve_processors(
            &"No one".to_string(),
            base_file_path,
            &mut file_counter,
            &mut file_dependency_table,
            &mut macro_table,
            &mut once_set,
            &mut processing_stack,
        )
    }

    fn scan_and_resolve_processors(
        &mut self,
        including_file_path: &String,
        file_path_to_include: &String,
        file_counter: &mut u32,
        file_dependency_table: &mut HashMap<u32, HashSet<u32>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<u32>,
    ) -> Result<Vec<PositionedToken>, (String, Option<Position>)> {
        // ==== GETTING THE ABSOLUTE FILE PATH STRING ====
        use std::fs;
        let absolute_file_path: String = match fs::canonicalize(file_path_to_include) {
            Ok(path) => path.to_str().unwrap_or(file_path_to_include).to_string(),
            Err(_) => {
                return Err((
                    format!(
                        "The file {} does not exist or could not be read",
                        file_path_to_include
                    ),
                    None,
                ));
            }
        };

        // ==== CHECKING IF THE FILE IS ALREADY SCANNED ====
        let file_id = match self.get_file_id_by_path(&absolute_file_path) {
            Some(id) => id,
            None => {
                *file_counter += 1;
                *file_counter
            }
        };

        // ==== CHECKING FOR IMPORT CYCLES ====
        if processing_stack.contains(&file_id) {
            return Err((
                format!(
                    "Include cycle detected: [{} -> {}]",
                    including_file_path.beautiful_path(),
                    absolute_file_path.beautiful_path(),
                ),
                None,
            ));
        }
        processing_stack.insert(file_id);

        // ==== SCANNING THE FILE CONTENT ====
        let mut ptokens: Vec<PositionedToken> =
            match scan_positioned_tokens_from_file(&absolute_file_path, file_id) {
                Ok(tkns) => tkns,
                Err((err, position_option)) => {
                    processing_stack.remove(&file_id);
                    return Err((err, position_option));
                }
            };

        // ==== ADDING THE FILE TO THE FILE TABLE ====
        if !self.file_table.contains_key(&file_id) {
            self.file_table.insert(file_id, absolute_file_path.clone());
        }

        // ==== RESOLVING INCLUDES ====
        let mut token_counter: usize = 0;
        let mut ptokens_len: usize = ptokens.len();

        while token_counter < ptokens_len {
            let tk = match ptokens.get(token_counter) {
                Some(tk) => tk,
                None => break,
            };

            match tk.token.clone() {
                Token::Processor(Processor::Include) => {
                    match ptokens.get(token_counter + 1).cloned() {
                        Some(next_p_tkn) => {
                            if let Token::StringLiteral(include_path_literal_string) =
                                next_p_tkn.token.clone()
                            {
                                let included_ptokens = match self.scan_and_resolve_processors(
                                    &absolute_file_path,
                                    &include_path_literal_string,
                                    file_counter,
                                    file_dependency_table,
                                    macro_table,
                                    once_set,
                                    processing_stack,
                                ) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        processing_stack.remove(&file_id);
                                        return Err(e);
                                    }
                                };

                                match file_dependency_table.get_mut(&file_id) {
                                    Some(dependencies) => {
                                        dependencies.insert(*file_counter);
                                    }
                                    None => {
                                        file_dependency_table
                                            .insert(file_id, HashSet::from([*file_counter]));
                                    }
                                }

                                ptokens.remove(token_counter); // Remove the @include token
                                ptokens.remove(token_counter); // Remove the file_path token

                                // Insert the included tokens at the current position
                                for included_ptkn in included_ptokens.into_iter().rev() {
                                    ptokens.insert(token_counter, included_ptkn);
                                }
                                ptokens_len = ptokens.len();
                            } else {
                                processing_stack.remove(&file_id);
                                return Err((
                                    "Include directive must be followed by a file path".to_string(),
                                    Some(next_p_tkn.position),
                                ));
                            }
                        }
                        None => {
                            processing_stack.remove(&file_id);
                            return Err((
                                "Include directive must be followed by a file path".to_string(),
                                Some(tk.position),
                            ));
                        }
                    }
                }

                Token::Processor(Processor::Define) => {
                    match ptokens.get(token_counter + 1).cloned() {
                        Some(define_identifier_ptkn) => match define_identifier_ptkn.token {
                            Token::Identifier(identifier_string) => {
                                let (macro_head, macro_definition_head_tkns_found) =
                                    match ptokens.scan_macro_definition_head(
                                        token_counter + 2,
                                        define_identifier_ptkn.position,
                                    ) {
                                        Ok((head, ptkns_found)) => (head, ptkns_found),
                                        Err((e, error_pos)) => {
                                            processing_stack.remove(&file_id);
                                            return Err((e, Some(error_pos)));
                                        }
                                    };

                                match ptokens.scan_macro_sequence(
                                    token_counter + macro_definition_head_tkns_found + 2,
                                    define_identifier_ptkn.position.line,
                                ) {
                                    Ok((define_sequence, ptokens_sequence_quantity_found)) => {
                                        macro_table.insert(
                                            identifier_string,
                                            (macro_head, define_sequence.clone()),
                                        );

                                        // remove the define, identifier, head and sequence tokens
                                        for _ in 0..(ptokens_sequence_quantity_found
                                            + macro_definition_head_tkns_found
                                            + 2)
                                        {
                                            if ptokens.len() > token_counter {
                                                ptokens.remove(token_counter);
                                            }
                                        }
                                        ptokens_len = ptokens.len();
                                        continue;
                                    }
                                    Err((err, err_pos)) => {
                                        processing_stack.remove(&file_id);
                                        return Err((err, Some(err_pos)));
                                    }
                                }
                            }
                            _ => {
                                processing_stack.remove(&file_id);
                                return Err((
                                    "Define directive must be followed by an identifier".to_string(),
                                    Some(define_identifier_ptkn.position),
                                ));
                            }
                        },
                        None => {
                            processing_stack.remove(&file_id);
                            return Err((
                                "Define directive must be followed by an identifier".to_string(),
                                Some(tk.position),
                            ));
                        }
                    }
                }

                Token::Processor(Processor::Once) => {
                    match once_set.get(&absolute_file_path) {
                        Some(_) => {
                            // remove all forward
                            while token_counter < ptokens.len() {
                                ptokens.remove(token_counter);
                            }
                        }
                        None => {
                            once_set.insert(absolute_file_path.clone());
                            ptokens.remove(token_counter);
                            ptokens_len = ptokens.len();
                        }
                    }
                }

                Token::Identifier(identifier_string) => {
                    if let Some((macro_head, define_sequence)) = macro_table.get(&identifier_string)
                    {
                        let (macro_call_head, head_tokens_quantity_found) =
                            match ptokens.scan_macro_calling_head(token_counter + 1, tk.position) {
                                Ok((head, ptkns_to_skip)) => (head, ptkns_to_skip),
                                Err((e, error_pos)) => {
                                    processing_stack.remove(&file_id);
                                    return Err((e, Some(error_pos)));
                                }
                            };

                        if macro_call_head.len() != macro_head.len() {
                            processing_stack.remove(&file_id);
                            return Err((
                                "Macro called with incorrect number of arguments".to_string(),
                                Some(tk.position),
                            ));
                        }

                        let mut ptkns_to_substitute = define_sequence.clone();

                        for (i, macro_call_arg) in macro_call_head.iter().enumerate() {
                            let definition_arg_string = match macro_head.get(i) {
                                Some(arg) => {
                                    if let Token::MacroArgIdentifier(arg_name) = arg.token.clone() {
                                        arg_name
                                    } else {
                                        processing_stack.remove(&file_id);
                                        return Err((
                                            "Expected macro argument".to_string(),
                                            Some(arg.position),
                                        ));
                                    }
                                }
                                None => {
                                    processing_stack.remove(&file_id);
                                    return Err((
                                        format!(
                                            "Macro '{}' called with too many arguments",
                                            identifier_string
                                        ),
                                        Some(tk.position),
                                    ));
                                }
                            };

                            for df_sq_ptkn in ptkns_to_substitute.iter_mut() {
                                if let Token::MacroArgIdentifier(df_sq_macro_arg_name) =
                                    df_sq_ptkn.token.clone()
                                {
                                    if df_sq_macro_arg_name == definition_arg_string {
                                        *df_sq_ptkn = macro_call_arg.clone();
                                    }
                                }
                            }
                        }

                        // check if there are not substituted macro arguments
                        for df_sq_ptkn in ptkns_to_substitute.iter() {
                            if let Token::MacroArgIdentifier(df_sq_macro_arg_name) =
                                df_sq_ptkn.token.clone()
                            {
                                processing_stack.remove(&file_id);
                                return Err((
                                    format!(
                                        "Macro argument '{}' not associated with any argument in the macro definition head",
                                        df_sq_macro_arg_name
                                    ),
                                    Some(df_sq_ptkn.position),
                                ));
                            }
                        }

                        // remove the identifier token and the head
                        let tk_q_to_rem = 1 + head_tokens_quantity_found;
                        for _ in 0..tk_q_to_rem {
                            if ptokens.len() > token_counter {
                                ptokens.remove(token_counter);
                            }
                        }

                        // Insert the defined processor tokens at the current position
                        for def_ptk in ptkns_to_substitute.iter().rev() {
                            ptokens.insert(token_counter, def_ptk.clone());
                        }

                        ptokens_len = ptokens.len();
                        continue;
                    } else {
                        token_counter += 1;
                        continue;
                    }
                }

                _ => {
                    token_counter += 1;
                    continue;
                }
            }
        }

        processing_stack.remove(&file_id);
        Ok(ptokens)
    }
}

// (restante do seu arquivo permanece igual)
fn read_file_content(file_path: &String) -> Result<String, String> {
    use std::fs;

    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content.replace("\r", "")), // Normalize line endings
        Err(_) => Err(format!("The file {} does not exist or could not be read", file_path)),
    }
}

fn scan_positioned_tokens_from_file(
    file_path: &String,
    file_id: u32,
) -> Result<Vec<PositionedToken>, (String, Option<Position>)> {
    match read_file_content(file_path) {
        Ok(content) => {
            let mut tokens: Vec<PositionedToken> = Vec::new();
            let mut token_accumulator = String::new();

            let mut actual_line = 1;
            let mut actual_column = 1;

            let mut initial_token_column = 1;

            let mut is_string_literal_mode = false;
            let mut is_commentary = false;
            let mut line_has_identation = false;

            let mut chars = content.chars().peekable();
            while let Some(ch) = chars.next() {
                if token_accumulator.is_empty() {
                    initial_token_column = actual_column;
                }
                match ch {
                    '\t' => {
                        if !is_commentary && !is_string_literal_mode {
                            line_has_identation = true;
                        }
                        actual_column += 1;
                        continue;
                    }
                    '\n' => {
                        is_commentary = false;
                        line_has_identation = false;

                        if !token_accumulator.is_empty() && !is_string_literal_mode {
                            if let Err(e) = tokens.push_positioned_token(
                                token_accumulator.clone(),
                                file_id,
                                actual_line,
                                if !line_has_identation {
                                    Some(initial_token_column)
                                } else {
                                    None
                                },
                            ) {
                                return Err((
                                    e,
                                    Some(Position::new(
                                        file_id,
                                        actual_line,
                                        Some(initial_token_column),
                                    )),
                                ));
                            }
                            token_accumulator.clear();
                        }

                        actual_line += 1;
                        actual_column = 1;
                        continue;
                    }
                    '#' => {
                        if !is_string_literal_mode {
                            is_commentary = true;
                            if !token_accumulator.is_empty() {
                                if let Err(e) = tokens.push_positioned_token(
                                    token_accumulator.clone(),
                                    file_id,
                                    actual_line,
                                    if !line_has_identation {
                                        Some(initial_token_column)
                                    } else {
                                        None
                                    },
                                ) {
                                    return Err((
                                        e,
                                        Some(Position::new(
                                            file_id,
                                            actual_line,
                                            Some(initial_token_column),
                                        )),
                                    ));
                                }
                                token_accumulator.clear();
                            }
                        } else {
                            token_accumulator.push(ch);
                        }
                        actual_column += 1;
                        continue;
                    }
                    ' ' => {
                        if is_commentary || is_string_literal_mode {
                            if is_string_literal_mode {
                                token_accumulator.push(ch);
                            }
                            actual_column += 1;
                            continue;
                        }
                        if !token_accumulator.is_empty() {
                            if let Err(e) = tokens.push_positioned_token(
                                token_accumulator.clone(),
                                file_id,
                                actual_line,
                                if !line_has_identation {
                                    Some(initial_token_column)
                                } else {
                                    None
                                },
                            ) {
                                return Err((
                                    e,
                                    Some(Position::new(
                                        file_id,
                                        actual_line,
                                        Some(initial_token_column),
                                    )),
                                ));
                            }
                            token_accumulator.clear();
                        }
                        actual_column += 1;
                        continue;
                    }
                    '"' => {
                        if is_commentary {
                            actual_column += 1;
                            continue;
                        }
                        if is_string_literal_mode {
                            token_accumulator.push(ch);
                            if let Err(e) = tokens.push_positioned_token(
                                token_accumulator.clone(),
                                file_id,
                                actual_line,
                                if !line_has_identation {
                                    Some(initial_token_column)
                                } else {
                                    None
                                },
                            ) {
                                return Err((
                                    e,
                                    Some(Position::new(
                                        file_id,
                                        actual_line,
                                        Some(initial_token_column),
                                    )),
                                ));
                            }
                            token_accumulator.clear();
                            is_string_literal_mode = false;
                        } else {
                            if !token_accumulator.is_empty() {
                                if let Err(e) = tokens.push_positioned_token(
                                    token_accumulator.clone(),
                                    file_id,
                                    actual_line,
                                    if !line_has_identation {
                                        Some(initial_token_column)
                                    } else {
                                        None
                                    },
                                ) {
                                    return Err((
                                        e,
                                        Some(Position::new(
                                            file_id,
                                            actual_line,
                                            Some(initial_token_column),
                                        )),
                                    ));
                                }
                                token_accumulator.clear();
                            }
                            token_accumulator.push(ch);
                            is_string_literal_mode = true;
                        }
                        actual_column += 1;
                        continue;
                    }
                    ',' | '[' | ']' | '(' | ')' | '\\' => {
                        if is_commentary || is_string_literal_mode {
                            if is_string_literal_mode {
                                token_accumulator.push(ch);
                            }
                            actual_column += 1;
                            continue;
                        }
                        if !token_accumulator.is_empty() {
                            if let Err(e) = tokens.push_positioned_token(
                                token_accumulator.clone(),
                                file_id,
                                actual_line,
                                if !line_has_identation {
                                    Some(initial_token_column)
                                } else {
                                    None
                                },
                            ) {
                                return Err((
                                    e,
                                    Some(Position::new(
                                        file_id,
                                        actual_line,
                                        Some(initial_token_column),
                                    )),
                                ));
                            }
                            token_accumulator.clear();
                        }
                        initial_token_column = actual_column;
                        if let Err(e) = tokens.push_positioned_token(
                            ch.to_string(),
                            file_id,
                            actual_line,
                            if !line_has_identation {
                                Some(initial_token_column)
                            } else {
                                None
                            },
                        ) {
                            return Err((
                                e,
                                Some(Position::new(
                                    file_id,
                                    actual_line,
                                    Some(initial_token_column),
                                )),
                            ));
                        }
                        actual_column += 1;
                        continue;
                    }
                    ':' => {
                        if is_commentary || is_string_literal_mode {
                            if is_string_literal_mode {
                                token_accumulator.push(ch);
                            }
                            actual_column += 1;
                            continue;
                        }

                        token_accumulator.push(ch);
                        actual_column += 1;

                        if !token_accumulator.is_empty() {
                            if let Err(e) = tokens.push_positioned_token(
                                token_accumulator.clone(),
                                file_id,
                                actual_line,
                                if !line_has_identation {
                                    Some(initial_token_column)
                                } else {
                                    None
                                },
                            ) {
                                return Err((
                                    e,
                                    Some(Position::new(
                                        file_id,
                                        actual_line,
                                        Some(initial_token_column),
                                    )),
                                ));
                            }
                            token_accumulator.clear();
                        }
                    }
                    _ => {
                        if is_commentary {
                            actual_column += 1;
                            continue;
                        }
                        token_accumulator.push(ch);
                        actual_column += 1;
                    }
                }
            }

            if !token_accumulator.is_empty() && !is_string_literal_mode {
                if let Err(e) = tokens.push_positioned_token(
                    token_accumulator.clone(),
                    file_id,
                    actual_line,
                    if !line_has_identation {
                        Some(initial_token_column)
                    } else {
                        None
                    },
                ) {
                    return Err((
                        e,
                        Some(Position::new(file_id, actual_line, Some(initial_token_column))),
                    ));
                }
            }

            Ok(tokens)
        }
        Err(err) => Err((err, None)),
    }
}
