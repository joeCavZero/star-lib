#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StarProcessor {
    Include,
    Define,
    Once,
}