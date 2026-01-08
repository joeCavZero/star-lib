
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarCustomSectionDefinition {
    pub name: String,
    pub section_type: StarSectionParsingType, 
}

impl StarCustomSectionDefinition {
    pub fn new(name: String, section_type: StarSectionParsingType) -> Self {
        Self {
            name,
            section_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarSectionParsingType {
    Data,
    Instr,
}