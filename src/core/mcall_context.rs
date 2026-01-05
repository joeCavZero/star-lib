use crate::core::Star;
use crate::core::StarDataMemory;
use crate::core::StarFileTable;
use crate::core::StarInstructionMemory;
use crate::core::StarPositionMemory;
use crate::core::StarRegisters;
use crate::core::StarGeneralRegister;

/// Provides a restricted and controlled view of the `Star` virtual machine
/// state for use by system-call (MCALL) interfaces.
///
/// This trait is designed to decouple external interfaces from the full `Star`
/// implementation, exposing only the components that are safe and necessary
/// for system-level interactions such as I/O, debugging, or host integration.
///
/// # Design goals
/// - Avoid exposing the full `Star` struct to interfaces.
/// - Clearly separate read-only and mutable access.
/// - Provide fine-grained accessors for registers and memory.
pub trait StarMcallContext {
    /// Returns an immutable reference to the full register set.
    ///
    /// # Use cases
    /// - Inspecting CPU state without modifying it.
    /// - Debugging or tracing execution state.
    fn get_registers(&self) -> &StarRegisters;

    /// Returns a mutable reference to the full register set.
    ///
    /// # Use cases
    /// - Modifying registers during a system call.
    /// - Updating flags, program counter, or special registers.
    fn get_registers_mut(&mut self) -> &mut StarRegisters;

    /// Returns the current value of a general-purpose register.
    ///
    /// # Parameters
    /// - `register`: The general-purpose register identifier.
    ///
    /// # Returns
    /// The 16-bit value stored in the specified register.
    fn get_general_register_value(&self, register: StarGeneralRegister) -> u16;

    /// Sets the value of a general-purpose register.
    ///
    /// # Parameters
    /// - `register`: The general-purpose register identifier.
    /// - `value`: The new 16-bit value to store.
    fn set_general_register_value(&mut self, register: StarGeneralRegister, value: u16);

    /// Returns an immutable reference to the data memory (RAM).
    ///
    /// # Use cases
    /// - Reading memory-mapped data.
    /// - Inspecting memory contents without mutation.
    fn get_data_memory(&self) -> &StarDataMemory;

    /// Returns a mutable reference to the data memory (RAM).
    ///
    /// # Use cases
    /// - Reading and writing memory during system calls.
    /// - Implementing memory-mapped I/O or buffers.
    fn get_data_memory_mut(&mut self) -> &mut StarDataMemory;

    /// Returns an immutable reference to the instruction memory.
    ///
    /// # Notes
    /// Instruction memory is exposed as read-only to prevent interfaces
    /// from altering the loaded program.
    fn get_instruction_memory(&self) -> &StarInstructionMemory;

    /// Returns an immutable reference to the position memory.
    ///
    /// # Use cases
    /// - Mapping runtime behavior back to source positions.
    /// - Producing diagnostics or debug output.
    fn get_position_memory(&self) -> &StarPositionMemory;

    /// Returns an immutable reference to the file table.
    ///
    /// The file table maps internal file identifiers to source file paths.
    ///
    /// # Use cases
    /// - Error reporting with file context.
    /// - Debugging and tooling support.
    fn get_file_table(&self) -> &StarFileTable;
}

/// Concrete implementation of `StarMcallContext` for the `Star` virtual machine.
///
/// This implementation simply forwards each accessor to the corresponding
/// internal field or method of `Star`, enforcing the abstraction boundary
/// defined by the trait.
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
