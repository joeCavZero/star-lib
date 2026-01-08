#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarDirective {
    Data,
    Instr,
    Byte,
    Word,
    Space,
    String,
    Stringz,
    Checkpoint,
    Custom(String),
}