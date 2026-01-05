use crate::core::Star;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StarPosition {
    pub file: Option<usize>,
    pub line: usize,
    pub column: Option<usize>,
}

impl StarPosition {
    pub fn new(file: Option<usize>, line: usize, column: Option<usize>) -> Self {
        Self {
            file,
            line,
            column,
        }
    }

    pub fn get_position_path(&self, star: &Star) -> String {
        match self.file {
            Some(file_num) => {
                let file_name = star.get_file_name(file_num);
                match self.column {
                    Some(column_num) => format!("file: {}, line: {}, column: {}", file_name, self.line, column_num),
                    None => format!("file: {}, line: {}", file_name, self.line),
                }
            }
            None => {
                match self.column {
                    Some(column_num) => format!("line: {}, column: {}", self.line, column_num),
                    None => format!("line: {}", self.line),
                }
            }
        }
    }
}