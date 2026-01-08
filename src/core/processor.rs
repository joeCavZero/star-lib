#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarProcessor {
    Include,
    Define,
    Once,
}