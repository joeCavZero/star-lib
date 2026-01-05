use crate::core::Star;
use crate::core::StarDataMemory;
use crate::core::StarFileTable;
use crate::core::StarInstructionMemory;
use crate::core::StarPositionMemory;
use crate::core::StarRegisters;
use crate::core::StarGeneralRegister;

pub trait StarMcallContext {
    fn get_registers(&self) -> &StarRegisters;
    fn get_registers_mut(&mut self) -> &mut StarRegisters;
    fn get_general_register_value(&self, register: StarGeneralRegister) -> u16;
    fn set_general_register_value(&mut self, register: StarGeneralRegister, value: u16);
    
    fn get_data_memory(&self) -> &StarDataMemory;
    fn get_data_memory_mut(&mut self) -> &mut StarDataMemory;

    fn get_instruction_memory(&self) -> &StarInstructionMemory;

    fn get_position_memory(&self) -> &StarPositionMemory;

    fn get_file_table(&self) -> &StarFileTable;
}

impl StarMcallContext for Star {
    fn get_registers(&self) -> &StarRegisters {
        &self.registers
    }
    fn get_registers_mut(&mut self) -> &mut StarRegisters {
        &mut self.registers
    }
    fn get_general_register_value(&self, register: StarGeneralRegister) -> u16 {
        self.registers.get_general_register_value(register)
    }
    fn set_general_register_value(&mut self, register: StarGeneralRegister, value: u16) {
        self.registers.set_general_register_value(register, value);
    }

    fn get_data_memory(&self) -> &StarDataMemory {
        &self.data_memory
    }

    fn get_data_memory_mut(&mut self) -> &mut StarDataMemory {
        &mut self.data_memory
    }

    fn get_instruction_memory(&self) -> &StarInstructionMemory {
        &self.instruction_memory
    }

    fn get_position_memory(&self) -> &StarPositionMemory {
        &self.position_memory
    }

    fn get_file_table(&self) -> &StarFileTable {
        &self.file_table
    }
}