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
}