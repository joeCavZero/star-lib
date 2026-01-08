use crate::core::*;

pub trait StarPositionedTokensVectorable {
    fn push_positioned_token(
        &mut self,
        token_string: String,
        file_id: Option<usize>,
        line: usize,
        column: Option<usize>,
    ) -> Result<(), String>;

    fn scan_macro_definition_head(
        &self,
        vector_offset: usize,
        identifier_position: StarPosition,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)>;

    fn scan_macro_sequence(
        &self,
        start_index: usize,
        identifier_line: usize,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)>;

    fn scan_macro_calling_head(
        &self,
        vector_offset: usize,
        identifier_position: StarPosition,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)>;
}

impl StarPositionedTokensVectorable for Vec<StarPositionedToken> {
    fn push_positioned_token(
        &mut self,
        token_string: String,
        file_id: Option<usize>,
        line: usize,
        column: Option<usize>,
    ) -> Result<(), String> {
        let tkn = StarToken::from_string(token_string);
        match tkn {
            Ok(token) => {
                self.push(StarPositionedToken {
                    token,
                    position: StarPosition::new(file_id, line, column),
                });
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    
    fn scan_macro_sequence(
        &self,
        start_index: usize,
        identifier_line: usize,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)> {
        // This function reads a sequence of tokens that defines a define processor
        // Backslash ables to continue reading the sequence in the next line
        let mut sequence: Vec<StarPositionedToken> = Vec::new();
        let mut ptokens_read: usize = 0;
        let mut line_to_read: usize = identifier_line;
        let mut index = start_index;

        while index < self.len() {
            let ptk = match self.get(index) {
                Some(ptk) => ptk,
                None => break,
            };

            if ptk.position.line > line_to_read {
                break;
            } else if ptk.position.line < line_to_read {
                return Err((
                    "Unexpected line change in define sequence".to_string(),
                    ptk.position,
                ));
            }

            match ptk.token.clone() {
                StarToken::Backslash => {
                    // If the token is a backslash, we continue reading in the next line
                    line_to_read += 1;
                    ptokens_read += 1;
                    index += 1;
                    continue;
                }
                _ => {
                    sequence.push(ptk.clone());
                    ptokens_read += 1;
                    index += 1;
                    continue;
                }
            }
        }

        Ok((sequence, ptokens_read))
    }

    fn scan_macro_definition_head(
        &self,
        vector_offset: usize,
        identifier_position: StarPosition,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)> {
        let mut head: Vec<StarPositionedToken> = Vec::new();
        let mut ptkns_found_quantity = 0;

        let mut is_parenthesis_head = false;
        let mut ptoken_counter = vector_offset;
        let mut last_position = identifier_position.clone();
        let mut has_comma = false;

        while ptoken_counter < self.len() {
            let ptkn = match self.get(ptoken_counter) {
                Some(ptkn) => ptkn,
                None => break,
            };

            let is_last_token = ptoken_counter == self.len() - 1;

            match ptkn.token.clone() {
                StarToken::LeftParenthesis => {
                    if !is_parenthesis_head {
                        is_parenthesis_head = true;
                        ptkns_found_quantity += 1;
                    } else {
                        return Err((
                            "Unexpected left parenthesis in macro definition head".to_string(),
                            ptkn.position.clone(),
                        ));
                    }
                }

                StarToken::RightParenthesis => {
                    if !is_parenthesis_head {
                        return Err((
                            "Unexpected right parenthesis in macro definition head".to_string(),
                            ptkn.position.clone(),
                        ));
                    }
                    if has_comma {
                        return Err((
                            "Unexpected right parenthesis in macro definition head, expected macro argument".to_string(),
                            ptkn.position.clone(),
                        ));
                    } else {
                        ptkns_found_quantity += 1;
                        return Ok((head, ptkns_found_quantity));
                    }
                }

                StarToken::Comma => {
                    if is_parenthesis_head {
                        if head.is_empty() {
                            return Err((
                                "Unexpected comma in macro definition head, expected macro argument".to_string(),
                                ptkn.position.clone(),
                            ));
                        }

                        if has_comma {
                            return Err((
                                "Unexpected comma in macro definition head".to_string(),
                                ptkn.position.clone(),
                            ));
                        }

                        has_comma = true;
                        ptkns_found_quantity += 1;
                    } else {
                        return Ok((head, ptkns_found_quantity));
                    }
                }

                StarToken::MacroArgIdentifier(_) => {
                    if is_parenthesis_head {
                        if head.is_empty() && has_comma {
                            return Err((
                                "Unexpected macro argument identifier in macro definition head, expected macro argument".to_string(),
                                ptkn.position.clone(),
                            ));
                        }

                        for existing_ptkn in head.iter() {
                            if let StarToken::MacroArgIdentifier(existing_arg_name) =
                                &existing_ptkn.token
                            {
                                if let StarToken::MacroArgIdentifier(arg_name) = &ptkn.token {
                                    if existing_arg_name == arg_name {
                                        return Err((
                                            "Duplicate macro argument identifier detected"
                                                .to_string(),
                                            ptkn.position.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                        head.push(ptkn.clone());
                        ptkns_found_quantity += 1;
                        has_comma = false;
                    } else {
                        return Ok((head, ptkns_found_quantity));
                    }
                }

                _ => {
                    if is_parenthesis_head {
                        return Err((
                            "Unexpected expression in macro definition head, expected macro argument or comma".to_string(),
                            ptkn.position.clone(),
                        ));
                    } else {
                        return Ok((head, ptkns_found_quantity));
                    }
                }
            }

            if is_last_token && !matches!(ptkn.token, StarToken::RightParenthesis) {
                return Err((
                    "Unexpected end of macro definition head, expected right parenthesis".to_string(),
                    ptkn.position.clone(),
                ));
            }

            ptoken_counter += 1;
            last_position = ptkn.position.clone();
        }

        if is_parenthesis_head && has_comma {
            return Err((
                "Unexpected end of macro definition head, expected macro argument".to_string(),
                last_position,
            ));
        }

        Ok((head, ptkns_found_quantity))
    }

    fn scan_macro_calling_head(
        &self,
        vector_offset: usize,
        identifier_position: StarPosition,
    ) -> Result<(Vec<StarPositionedToken>, usize), (String, StarPosition)> {
        let mut head: Vec<StarPositionedToken> = Vec::new();
        let mut ptkns_found_quantity = 0;

        let mut is_parenthesis_head = false;
        let mut ptoken_counter = vector_offset;
        let mut last_position = identifier_position.clone();
        let mut has_comma = false;

        while ptoken_counter < self.len() {
            let ptkn = match self.get(ptoken_counter) {
                Some(ptkn) => ptkn,
                None => break,
            };

            let is_last_token = ptoken_counter == self.len() - 1;

            match ptkn.token.clone() {
                StarToken::LeftParenthesis => {
                    if !is_parenthesis_head {
                        is_parenthesis_head = true;
                        ptkns_found_quantity += 1;
                    } else {
                        return Err((
                            "Unexpected left parenthesis in macro calling head".to_string(),
                            ptkn.position.clone(),
                        ));
                    }
                }

                StarToken::RightParenthesis => {
                    if !is_parenthesis_head {
                        return Err((
                            "Unexpected right parenthesis in macro calling head".to_string(),
                            ptkn.position.clone(),
                        ));
                    }
                    if has_comma {
                        return Err((
                            "Unexpected right parenthesis in macro calling head, expected argument"
                                .to_string(),
                            ptkn.position.clone(),
                        ));
                    } else {
                        ptkns_found_quantity += 1;
                        return Ok((head, ptkns_found_quantity));
                    }
                }

                StarToken::Comma => {
                    if is_parenthesis_head {
                        if head.is_empty() {
                            return Err((
                                "Unexpected comma in macro calling head, expected argument"
                                    .to_string(),
                                ptkn.position.clone(),
                            ));
                        }

                        if has_comma {
                            return Err((
                                "Unexpected comma in macro calling head".to_string(),
                                ptkn.position.clone(),
                            ));
                        }

                        has_comma = true;
                        ptkns_found_quantity += 1;
                    } else {
                        return Ok((head, ptkns_found_quantity));
                    }
                }

                _ => {
                    if is_parenthesis_head {
                        if has_comma || head.is_empty() {
                            head.push(ptkn.clone());
                            ptkns_found_quantity += 1;
                            has_comma = false;
                        } else {
                            return Err((
                                "Unexpected token in macro calling head, expected comma"
                                    .to_string(),
                                ptkn.position.clone(),
                            ));
                        }
                    } else {
                        return Ok((head, ptkns_found_quantity));
                    }
                }
            }

            if is_last_token && !matches!(ptkn.token, StarToken::RightParenthesis) {
                return Err((
                    "Unexpected end of macro calling head, expected right parenthesis".to_string(),
                    ptkn.position.clone(),
                ));
            }

            ptoken_counter += 1;
            last_position = ptkn.position.clone();
        }

        if is_parenthesis_head && has_comma {
            return Err((
                "Unexpected end of macro calling head, expected argument".to_string(),
                last_position,
            ));
        }

        Ok((head, ptkns_found_quantity))
    }
}
