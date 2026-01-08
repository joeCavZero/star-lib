use crate::core::STAR_MEMORY_64KB;
use crate::core::StarInstructionMemory;
use crate::core::StarPositionMemory;
use crate::core::StarDataMemory;

pub struct StarMemories {
    pub data_memory: StarDataMemory,
    pub instruction_memory: StarInstructionMemory,
    pub position_memory: StarPositionMemory,
}

impl StarMemories {
    pub fn new() -> Self {
        let mut data_memory = [0; STAR_MEMORY_64KB];
        for b in data_memory.iter_mut() {
            *b = rand::random::<u8>();
        }
        Self {
            data_memory,
            instruction_memory: Vec::new(),
            position_memory: Vec::new(),
        }
    }
}