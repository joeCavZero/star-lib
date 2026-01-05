use std::mem::transmute;

pub type StarInstructionMemory = Vec<u8>;

pub trait StarInstructionMemoryable {
    fn get_full_instruction_by_program_counter(&self, program_counter_value: u16) -> Option<(u16, u32)>;
}

impl StarInstructionMemoryable for StarInstructionMemory {
    fn get_full_instruction_by_program_counter(&self, program_counter_value: u16) -> Option<(u16, u32)> {
        let (instruction_pointer, instr_high, instr_low) = match (program_counter_value as u32).checked_mul(2) {
            Some(high_pos) => {
                match high_pos.checked_add(1) {
                    Some(low_pos) => {
                        (   
                            low_pos,

                            match self.get(high_pos as usize) {
                                Some(byte) => *byte,
                                None => return None,
                            },
                            
                            match self.get(low_pos as usize) {
                                Some(byte) => *byte,
                                None => return None,
                            },
                        )
                    }
                    None => return None,
                }
            }
            None => return None,
        };

        return Some(
            (
                unsafe { transmute::<(u8, u8), u16>((instr_low, instr_high)) },
                instruction_pointer
            )
        );
    }
    
}