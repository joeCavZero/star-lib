use crate::core::*;
use super::sequence::*;

#[derive(Debug, Clone)]
pub struct InstrCamp {
    pub label_declarations: Vec<StarPositionedToken>,
    pub instruction: StarPositionedToken,
    pub sequence: StarSequence,
}

#[derive(Debug, Clone)]
pub struct DataCamp {
    pub label_declarations: Vec<StarPositionedToken>,
    pub directive: StarPositionedToken,
    pub arg: DataCampArg,
}

#[derive(Debug, Clone)]
pub enum DataCampArg {
    Empty,
    Unique(StarPositionedToken),
    Multiple(Vec<StarPositionedToken>),
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub data_field: Vec<DataCamp>,
    pub instr_field: Vec<InstrCamp>
}

