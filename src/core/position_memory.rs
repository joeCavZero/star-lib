use crate::core::StarPosition;

pub type StarPositionMemory = Vec<StarPosition>;

pub trait StarPositionMemoryable {
    fn get_by_program_counter(&self, program_counter_value: u16) -> Option<&StarPosition>;
}

impl StarPositionMemoryable for StarPositionMemory {
    fn get_by_program_counter(&self, program_counter_value: u16) -> Option<&StarPosition> {
        self
        .get(program_counter_value as usize)
    }
}