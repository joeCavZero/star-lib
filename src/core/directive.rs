#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StarDirective {
    Data,
    Instr,
    Byte,
    Word,
    Space,
    String,
    Stringz,
    Checkpoint,
}