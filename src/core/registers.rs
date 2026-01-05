use crate::core::DATA_MEMORY_SIZE;
use crate::core::StarGeneralRegister;

#[derive(Debug, Clone)]
pub struct StarRegisters {
    // ==== GENERAL REGISTERS ====
    pub zero: u16,
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub e: u16,
    pub f: u16,
    pub g: u16,
    pub aux1: u16,
    pub aux2: u16,
    pub aux3: u16,
    pub carry: u16,
    pub low: u16,
    pub high: u16,
    pub return_address: u16,
    pub stack_pointer: u16,

    // ==== OCULT REGISTERS ====
    pub program_counter: u16,
    pub instruction_pointer: u32,
    pub instruction_register: u16,
}

impl StarRegisters {
    pub fn new() -> Self {
        Self {
            zero: 0,
            a: rand::random::<u16>(),
            b: rand::random::<u16>(),
            c: rand::random::<u16>(),
            d: rand::random::<u16>(),
            e: rand::random::<u16>(),
            f: rand::random::<u16>(),
            g: rand::random::<u16>(),
            aux1: rand::random::<u16>(),
            aux2: rand::random::<u16>(),
            aux3: rand::random::<u16>(),
            carry: rand::random::<u16>(),
            low: rand::random::<u16>(),
            high: rand::random::<u16>(),
            return_address: rand::random::<u16>(),
            stack_pointer: (DATA_MEMORY_SIZE - 1) as u16,
            program_counter: 0,
            instruction_pointer: rand::random::<u32>(),
            instruction_register: rand::random::<u16>(),
        }
    }

    pub fn get_general_register_value(&self, register: StarGeneralRegister) -> u16 {
        match register {
            StarGeneralRegister::Zero => self.zero,
            StarGeneralRegister::A => self.a,
            StarGeneralRegister::B => self.b,
            StarGeneralRegister::C => self.c,
            StarGeneralRegister::D => self.d,
            StarGeneralRegister::E => self.e,
            StarGeneralRegister::F => self.f,
            StarGeneralRegister::G => self.g,
            StarGeneralRegister::Aux1 => self.aux1,
            StarGeneralRegister::Aux2 => self.aux2,
            StarGeneralRegister::Aux3 => self.aux3,
            StarGeneralRegister::Carry => self.carry,
            StarGeneralRegister::Low => self.low,
            StarGeneralRegister::High => self.high,
            StarGeneralRegister::ReturnAddress => self.return_address,
            StarGeneralRegister::StackPointer => self.stack_pointer,
        }
    }

    pub fn set_general_register_value(&mut self, register: StarGeneralRegister, value: u16) {
        match register {
            StarGeneralRegister::Zero => {}
            StarGeneralRegister::A => self.a = value,
            StarGeneralRegister::B => self.b = value,
            StarGeneralRegister::C => self.c = value,
            StarGeneralRegister::D => self.d = value,
            StarGeneralRegister::E => self.e = value,
            StarGeneralRegister::F => self.f = value,
            StarGeneralRegister::G => self.g = value,
            StarGeneralRegister::Aux1 => self.aux1 = value,
            StarGeneralRegister::Aux2 => self.aux2 = value,
            StarGeneralRegister::Aux3 => self.aux3 = value,
            StarGeneralRegister::Carry => self.carry = value,
            StarGeneralRegister::Low => self.low = value,
            StarGeneralRegister::High => self.high = value,
            StarGeneralRegister::ReturnAddress => self.return_address = value,
            StarGeneralRegister::StackPointer => self.stack_pointer = value,
        }
    }

    pub fn increment_program_counter(&mut self) -> Result<(), String>{
        match self.program_counter.checked_add(1) {
            Some(new_pc) => {
                self.program_counter = new_pc;
                return Ok(());
            }
            None => {
                return Err("Program counter overflow".to_string());
            }
        }
    }
}