use crate::core::Star;

/// Represents a source code position associated with an instruction or token
/// in the `Star` virtual machine.
///
/// A `StarPosition` optionally tracks the originating file, line number,
/// and column number, enabling precise diagnostics and error reporting across
/// multi-file assembly inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StarPosition {
    /// Optional file identifier, referring to an entry in the VM file table.
    pub file: Option<usize>,

    /// 1-based line number in the source file.
    pub line: usize,

    /// Optional 1-based column number within the source line.
    pub column: Option<usize>,
}

impl StarPosition {
    /// Creates a new `StarPosition` instance.
    ///
    /// # Parameters
    /// - `file`: Optional file identifier associated with this position.
    /// - `line`: The 1-based line number in the source.
    /// - `column`: Optional 1-based column number.
    ///
    /// # Returns
    /// A fully initialized `StarPosition`.
    pub fn new(file: Option<usize>, line: usize, column: Option<usize>) -> Self {
        Self { file, line, column }
    }

    /// Formats this position as a human-readable string, including file name
    /// information when available.
    ///
    /// # Parameters
    /// - `star`: A reference to the `Star` virtual machine, used to resolve
    ///   file identifiers into file names.
    ///
    /// # Returns
    /// A formatted string describing the source position, such as:
    /// - `"file: main.star, line: 10, column: 5"`
    /// - `"file: utils.star, line: 42"`
    /// - `"line: 7, column: 3"`
    /// - `"line: 15"`
    ///
    /// # Notes
    /// If no file identifier is present, only line and column information
    /// is included in the output.
    pub fn get_position_path(&self, star: &Star) -> String {
        match self.file {
            Some(file_num) => {
                let file_name = star.get_file_name(file_num);
                match self.column {
                    Some(column_num) => {
                        format!(
                            "file: {}, line: {}, column: {}",
                            file_name, self.line, column_num
                        )
                    }
                    None => format!("file: {}, line: {}", file_name, self.line),
                }
            }
            None => match self.column {
                Some(column_num) => format!("line: {}, column: {}", self.line, column_num),
                None => format!("line: {}", self.line),
            },
        }
    }
}
