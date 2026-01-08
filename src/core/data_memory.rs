pub const STAR_MEMORY_64KB: usize = 65536;

/// Represents the data memory (RAM) of the `Star` virtual machine.
///
/// This type is a fixed-size byte array that models the VM's data address space.
/// Each element corresponds to a single addressable byte in memory.
///
/// # Characteristics
/// - Fixed size defined by `DATA_MEMORY_SIZE`.
/// - Byte-addressable (`u8`), suitable for low-level load/store operations.
/// - Typically initialized with random values to emulate uninitialized RAM.
///
/// # Usage
/// Data memory is accessed during execution by load/store instructions and may
/// also be inspected or modified by system-call interfaces through controlled
/// accessors.
pub type StarDataMemory = [u8; STAR_MEMORY_64KB];

/// Defines the interface for readable and writable data memory in the `Star`
/// virtual machine.
///
/// This trait abstracts byte-level access to the VM data memory, providing
/// explicit methods for storing to and loading from memory addresses with
/// bounds and validity checking.
///
/// # Design goals
/// - Centralize memory access semantics (load/store).
/// - Allow consistent error handling for invalid addresses.
/// - Enable alternative memory backends while preserving a common API.
pub trait StarDataMemoryable {
    /// Stores a single byte at the given memory address.
    ///
    /// # Parameters
    /// - `address`: The 16-bit memory address where the byte will be written.
    /// - `value`: The 8-bit value to store.
    ///
    /// # Returns
    /// - `Ok(())` if the write succeeds.
    /// - `Err(String)` if the address is out of bounds or otherwise invalid.
    ///
    /// # Notes
    /// Implementations are expected to perform bounds checking and preserve
    /// memory safety.
    fn store(&mut self, address: u16, value: u8) -> Result<(), String>;

    /// Loads a single byte from the given memory address.
    ///
    /// # Parameters
    /// - `address`: The 16-bit memory address to read from.
    ///
    /// # Returns
    /// - `Ok(u8)` containing the byte read from memory.
    /// - `Err(String)` if the address is out of bounds or otherwise invalid.
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