pub const DATA_MEMORY_SIZE: usize = 65536;

pub type StarDataMemory = [u8; DATA_MEMORY_SIZE];

pub trait StarDataMemoryable {
    fn store(&mut self, address: u16, value: u8) -> Result<(), String>;

    fn load(&self, address: u16) -> Result<u8, String>;
}

impl StarDataMemoryable for StarDataMemory {
    fn store(&mut self, address: u16, value: u8) -> Result<(), String> {
        match self.get_mut(address as usize) {
            Some(cell) => {
                *cell = value;
                Ok(())
            }
            None => Err(format!("Data memory address {} out of bounds", address)),
        }
    }

    fn load(&self, address: u16) -> Result<u8, String> {
        match self.get(address as usize) {
            Some(value) => Ok(*value),
            None => Err(format!("Data memory address {} out of bounds", address)),
        }
    }

    
}