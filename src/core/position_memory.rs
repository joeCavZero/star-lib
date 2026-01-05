use crate::core::StarPosition;

/// Represents the position memory of the `Star` virtual machine.
///
/// This type stores source-level position metadata associated with instructions,
/// allowing the VM to map runtime behavior back to the original source code.
/// Each entry typically corresponds to a single instruction index.
pub type StarPositionMemory = Vec<StarPosition>;

/// Defines the interface for accessing source position information by program counter.
///
/// This trait abstracts the lookup of source positions associated with executed
/// instructions, enabling diagnostics, error reporting, and debugging without
/// coupling execution logic to the concrete storage layout.
pub trait StarPositionMemoryable {
    /// Retrieves the source position associated with the given program counter.
    ///
    /// # Parameters
    /// - `program_counter_value`: The current program counter (instruction index).
    ///
    /// # Returns
    /// - `Some(&StarPosition)` if a position entry exists for the given counter.
    /// - `None` if the counter is out of bounds or no position information is available.
    fn get_by_program_counter(&self, program_counter_value: u16) -> Option<&StarPosition>;
}

/// Provides a concrete implementation of `StarPositionMemoryable` for
/// `StarPositionMemory`.
///
/// This implementation performs a direct index lookup, interpreting the program
/// counter as an index into the position vector.
impl StarPositionMemoryable for StarPositionMemory {
    /// Returns the source position corresponding to the given program counter.
    ///
    /// # Notes
    /// This is a simple, zero-cost abstraction over `Vec::get`, and therefore
    /// inherits its bounds-checking behavior.
    fn get_by_program_counter(&self, program_counter_value: u16) -> Option<&StarPosition> {
        self.get(program_counter_value as usize)
    }
}
