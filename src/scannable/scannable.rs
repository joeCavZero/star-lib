use std::collections::HashSet;
use std::collections::HashMap;

use crate::core::*;
use crate::utils::*;
use crate::scannable::positioned_tokens_vectorable::*;

type MacroTable = HashMap<String, (Vec<StarPositionedToken>, Vec<StarPositionedToken>)>;

pub trait StarScannable {
    fn scan_file(&mut self, base_file_path: &str)
        -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)>;

    fn scan(&mut self, source: &String)
        -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)>;

    fn scan_and_resolve_processors(
        &mut self,
        including_file_path: &String,
        file_path_to_include: &String,
        file_counter: &mut usize,
        file_dependency_table: &mut HashMap<usize, HashSet<usize>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<usize>,
        file_mode: bool,
    ) -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)>;

    fn resolve_processors_common(
        &mut self,
        absolute_file_path: &String,
        file_id: usize,
        ptokens: &mut Vec<StarPositionedToken>,
        file_counter: &mut usize,
        file_dependency_table: &mut HashMap<usize, HashSet<usize>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<usize>,
        file_mode: bool,
    ) -> Result<(), (String, Option<StarPosition>)> ;
}


impl StarScannable for Star {
    fn scan_file(&mut self, base_file_path: &str)
        -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)>
    {
        let base_file_path_string = base_file_path.to_string();

        let mut file_dependency_table: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut macro_table: MacroTable = HashMap::new();
        let mut once_set: HashSet<String> = HashSet::new();
        let mut file_counter: usize = 0;
        let mut processing_stack: HashSet<usize> = HashSet::new();

        self.scan_and_resolve_processors(
            &"No one".to_string(),
            &base_file_path_string,
            &mut file_counter,
            &mut file_dependency_table,
            &mut macro_table,
            &mut once_set,
            &mut processing_stack,
            true,
        )
    }

    fn scan(&mut self, source: &String)
        -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)>
    {
        let mut file_dependency_table: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut macro_table: MacroTable = HashMap::new();
        let mut once_set: HashSet<String> = HashSet::new();
        let mut file_counter: usize = 0;
        let mut processing_stack: HashSet<usize> = HashSet::new();

        // “arquivo virtual” para posições/erros
        let virtual_path: String = "<memory>".to_string();

        // escolhe um id consistente pra “memória”
        let file_id: usize = 0;

        // opcional: registrar no file_table pra beautiful_path/erros
        if !self.file_table.contains_key(&file_id) {
            self.file_table.insert(file_id, virtual_path.clone());
        }

        processing_stack.insert(file_id);

        let mut ptokens = scan_positioned_tokens_from_str(&source, file_id)?;

        // file_mode = false => bloqueia @include
        self.resolve_processors_common(
            &virtual_path,
            file_id,
            &mut ptokens,
            &mut file_counter,
            &mut file_dependency_table,
            &mut macro_table,
            &mut once_set,
            &mut processing_stack,
            false,
        )?;

        processing_stack.remove(&file_id);
        Ok(ptokens)
    }

    fn scan_and_resolve_processors(
        &mut self,
        including_file_path: &String,
        file_path_to_include: &String,
        file_counter: &mut usize,
        file_dependency_table: &mut HashMap<usize, HashSet<usize>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<usize>,
        file_mode: bool,
    ) -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)> {
        use std::fs;

        // ==== GETTING THE ABSOLUTE FILE PATH STRING ====
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
        let mut ptokens: Vec<StarPositionedToken> =
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

        // ==== COMMON RESOLUTION ====
        if let Err(e) = self.resolve_processors_common(
            &absolute_file_path,
            file_id,
            &mut ptokens,
            file_counter,
            file_dependency_table,
            macro_table,
            once_set,
            processing_stack,
            file_mode,
        ) {
            processing_stack.remove(&file_id);
            return Err(e);
        }

        processing_stack.remove(&file_id);
        Ok(ptokens)
    }


    fn resolve_processors_common(
        &mut self,
        absolute_file_path: &String,
        file_id: usize,
        ptokens: &mut Vec<StarPositionedToken>,
        file_counter: &mut usize,
        file_dependency_table: &mut HashMap<usize, HashSet<usize>>,
        macro_table: &mut MacroTable,
        once_set: &mut HashSet<String>,
        processing_stack: &mut HashSet<usize>,
        file_mode: bool,
    ) -> Result<(), (String, Option<StarPosition>)> {
        // ==== RESOLVING INCLUDES / DEFINES / ONCE / MACROS ====
        let mut token_counter: usize = 0;
        let mut ptokens_len: usize = ptokens.len();

        while token_counter < ptokens_len {
            let tk = match ptokens.get(token_counter) {
                Some(tk) => tk,
                None => break,
            };

            match tk.token.clone() {
                StarToken::StarProcessor(StarProcessor::Include) => {
                    if !file_mode {
                        return Err((
                            "You should not use includes in this mode".to_string(),
                            Some(tk.position.clone()),
                        ));
                    }

                    match ptokens.get(token_counter + 1).cloned() {
                        Some(next_p_tkn) => {
                            if let StarToken::StringLiteral(include_path_literal_string) =
                                next_p_tkn.token.clone()
                            {
                                let included_ptokens = match self.scan_and_resolve_processors(
                                    absolute_file_path,
                                    &include_path_literal_string,
                                    file_counter,
                                    file_dependency_table,
                                    macro_table,
                                    once_set,
                                    processing_stack,
                                    file_mode,
                                ) {
                                    Ok(v) => v,
                                    Err(e) => return Err(e),
                                };

                                match file_dependency_table.get_mut(&file_id) {
                                    Some(deps) => {
                                        deps.insert(*file_counter);
                                    }
                                    None => {
                                        file_dependency_table
                                            .insert(file_id, HashSet::from([*file_counter]));
                                    }
                                }

                                ptokens.remove(token_counter); // @include
                                ptokens.remove(token_counter); // "path"

                                for included_ptkn in included_ptokens.into_iter().rev() {
                                    ptokens.insert(token_counter, included_ptkn);
                                }

                                ptokens_len = ptokens.len();
                                continue;
                            } else {
                                return Err((
                                    "Include directive must be followed by a file path".to_string(),
                                    Some(next_p_tkn.position),
                                ));
                            }
                        }
                        None => {
                            return Err((
                                "Include directive must be followed by a file path".to_string(),
                                Some(tk.position.clone()),
                            ));
                        }
                    }
                }

                StarToken::StarProcessor(StarProcessor::Define) => {
                    match ptokens.get(token_counter + 1).cloned() {
                        Some(define_identifier_ptkn) => match define_identifier_ptkn.token {
                            StarToken::Identifier(identifier_string) => {
                                let (macro_head, macro_definition_head_tkns_found) =
                                    match ptokens.scan_macro_definition_head(
                                        token_counter + 2,
                                        define_identifier_ptkn.position.clone(),
                                    ) {
                                        Ok((head, found)) => (head, found),
                                        Err((e, error_pos)) => return Err((e, Some(error_pos))),
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

                                        // remove define, identifier, head, sequence
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
                                    Err((err, err_pos)) => return Err((err, Some(err_pos))),
                                }
                            }
                            _ => {
                                return Err((
                                    "Define directive must be followed by an identifier"
                                        .to_string(),
                                    Some(define_identifier_ptkn.position),
                                ));
                            }
                        },
                        None => {
                            return Err((
                                "Define directive must be followed by an identifier".to_string(),
                                Some(tk.position.clone()),
                            ));
                        }
                    }
                }

                StarToken::StarProcessor(StarProcessor::Once) => {
                    match once_set.get(absolute_file_path) {
                        Some(_) => {
                            while token_counter < ptokens.len() {
                                ptokens.remove(token_counter);
                            }
                            ptokens_len = ptokens.len();
                            continue;
                        }
                        None => {
                            once_set.insert(absolute_file_path.clone());
                            ptokens.remove(token_counter);
                            ptokens_len = ptokens.len();
                            continue;
                        }
                    }
                }

                StarToken::Identifier(identifier_string) => {
                    if let Some((macro_head, define_sequence)) = macro_table.get(&identifier_string)
                    {
                        let (macro_call_head, head_tokens_quantity_found) =
                            match ptokens.scan_macro_calling_head(
                                token_counter + 1,
                                tk.position.clone(),
                            ) {
                                Ok((head, q)) => (head, q),
                                Err((e, error_pos)) => return Err((e, Some(error_pos))),
                            };

                        if macro_call_head.len() != macro_head.len() {
                            return Err((
                                "Macro called with incorrect number of arguments".to_string(),
                                Some(tk.position.clone()),
                            ));
                        }

                        let mut ptkns_to_substitute = define_sequence.clone();

                        for (i, macro_call_arg) in macro_call_head.iter().enumerate() {
                            let definition_arg_string = match macro_head.get(i) {
                                Some(arg) => {
                                    if let StarToken::MacroArgIdentifier(arg_name) = arg.token.clone() {
                                        arg_name
                                    } else {
                                        return Err((
                                            "Expected macro argument".to_string(),
                                            Some(arg.position.clone()),
                                        ));
                                    }
                                }
                                None => {
                                    return Err((
                                        format!(
                                            "Macro '{}' called with too many arguments",
                                            identifier_string
                                        ),
                                        Some(tk.position.clone()),
                                    ));
                                }
                            };

                            for df_sq_ptkn in ptkns_to_substitute.iter_mut() {
                                if let StarToken::MacroArgIdentifier(df_sq_macro_arg_name) =
                                    df_sq_ptkn.token.clone()
                                {
                                    if df_sq_macro_arg_name == definition_arg_string {
                                        *df_sq_ptkn = macro_call_arg.clone();
                                    }
                                }
                            }
                        }

                        for df_sq_ptkn in ptkns_to_substitute.iter() {
                            if let StarToken::MacroArgIdentifier(df_sq_macro_arg_name) =
                                df_sq_ptkn.token.clone()
                            {
                                return Err((
                                    format!(
                                        "Macro argument '{}' not associated with any argument in the macro definition head",
                                        df_sq_macro_arg_name
                                    ),
                                    Some(df_sq_ptkn.position.clone()),
                                ));
                            }
                        }

                        let tk_q_to_rem = 1 + head_tokens_quantity_found;
                        for _ in 0..tk_q_to_rem {
                            if ptokens.len() > token_counter {
                                ptokens.remove(token_counter);
                            }
                        }

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

        Ok(())
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
    file_id: usize,
) -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)> {
    match read_file_content(file_path) {
        Ok(content) => scan_positioned_tokens_from_str(&content, file_id),
        Err(err) => Err((err, None)),
    }
}
fn scan_positioned_tokens_from_str(
    content: &str,
    file_id: usize,
) -> Result<Vec<StarPositionedToken>, (String, Option<StarPosition>)> {
    let content = content.replace("\r", ""); // normaliza CRLF

    let mut tokens: Vec<StarPositionedToken> = Vec::new();
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
                        Some(file_id),
                        actual_line,
                        if !line_has_identation {
                            Some(initial_token_column)
                        } else {
                            None
                        },
                    ) {
                        return Err((
                            e,
                            Some(StarPosition::new(
                                Some(file_id),
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
                            Some(file_id),
                            actual_line,
                            if !line_has_identation {
                                Some(initial_token_column)
                            } else {
                                None
                            },
                        ) {
                            return Err((
                                e,
                                Some(StarPosition::new(
                                    Some(file_id),
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
                        Some(file_id),
                        actual_line,
                        if !line_has_identation {
                            Some(initial_token_column)
                        } else {
                            None
                        },
                    ) {
                        return Err((
                            e,
                            Some(StarPosition::new(
                                Some(file_id),
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
                        Some(file_id),
                        actual_line,
                        if !line_has_identation {
                            Some(initial_token_column)
                        } else {
                            None
                        },
                    ) {
                        return Err((
                            e,
                            Some(StarPosition::new(
                                Some(file_id),
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
                            Some(file_id),
                            actual_line,
                            if !line_has_identation {
                                Some(initial_token_column)
                            } else {
                                None
                            },
                        ) {
                            return Err((
                                e,
                                Some(StarPosition::new(
                                    Some(file_id),
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
                        Some(file_id),
                        actual_line,
                        if !line_has_identation {
                            Some(initial_token_column)
                        } else {
                            None
                        },
                    ) {
                        return Err((
                            e,
                            Some(StarPosition::new(
                                Some(file_id),
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
                    Some(file_id),
                    actual_line,
                    if !line_has_identation {
                        Some(initial_token_column)
                    } else {
                        None
                    },
                ) {
                    return Err((
                        e,
                        Some(StarPosition::new(
                            Some(file_id),
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
                        Some(file_id),
                        actual_line,
                        if !line_has_identation {
                            Some(initial_token_column)
                        } else {
                            None
                        },
                    ) {
                        return Err((
                            e,
                            Some(StarPosition::new(
                                Some(file_id),
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
            Some(file_id),
            actual_line,
            if !line_has_identation {
                Some(initial_token_column)
            } else {
                None
            },
        ) {
            return Err((
                e,
                Some(StarPosition::new(Some(file_id), actual_line, Some(initial_token_column))),
            ));
        }
    }

    Ok(tokens)
}



