use std::mem::transmute;

/// Represents the instruction memory of the `Star` virtual machine.
///
/// This type is a dynamically sized byte vector that stores the raw instruction
/// stream produced by the assembler or loaded from a binary representation.
/// Instructions are encoded as sequences of bytes and are typically fetched
/// and combined into larger instruction words during execution.
pub type StarInstructionMemory = Vec<u8>;

/// Defines the interface for instruction fetching from instruction memory.
///
/// This trait abstracts the logic required to retrieve a complete instruction
/// word from instruction memory based on the current program counter, handling
/// instruction width, alignment, and bounds checking.
///
/// # Design goals
/// - Centralize instruction fetch semantics.
/// - Decouple execution logic from raw memory layout.
/// - Allow different instruction encodings or memory backends.
pub trait StarInstructionMemoryable {
    /// Fetches a full instruction word using the given program counter value.
    ///
    /// # Parameters
    /// - `program_counter_value`: The current program counter (in instruction units),
    ///   where each unit corresponds to one 16-bit instruction.
    ///
    /// # Returns
    /// - `Some((instruction, instruction_pointer))` where:
    ///   - `instruction` is the decoded 16-bit instruction word.
    ///   - `instruction_pointer` is the underlying byte offset in instruction memory
    ///     corresponding to the *low* byte position (i.e., `pc * 2 + 1` in this implementation).
    /// - `None` if:
    ///   - The program counter would overflow when converted to a byte index,
    ///   - The computed byte offsets are out of bounds,
    ///   - Or the instruction cannot be fetched for any reason.
    ///
    /// # Notes
    /// Implementations should ensure consistent endianness between encoding and decoding.
    fn get_full_instruction_by_program_counter(
        &self,
        program_counter_value: u16,
    ) -> Option<(u16, u32)>;
}

/// Provides a concrete instruction-fetch implementation for `StarInstructionMemory`.
///
/// This implementation treats instruction memory as a stream of bytes where each
/// instruction occupies exactly 2 bytes. The program counter is interpreted as an
/// instruction index, which is converted to a byte offset by multiplying by 2.
///
/// # Encoding
/// This fetcher reconstructs a `u16` from two `u8` bytes using `transmute`.
/// In the returned instruction word, `instr_low` is placed as the low byte and
/// `instr_high` as the high byte.
///
/// # Safety
/// Uses `unsafe` `transmute` to combine bytes into a `u16`. This relies on the
/// platform having 8-bit bytes and a `u16` representation compatible with two bytes,
/// which is true for all Rust-supported targets, but still requires care and
/// consistent byte ordering with the assembler.
impl StarInstructionMemoryable for StarInstructionMemory {
    /// Fetches the 16-bit instruction at the given program counter.
    ///
    /// # Algorithm
    /// - Compute `high_pos = pc * 2`.
    /// - Compute `low_pos = high_pos + 1`.
    /// - Read two bytes from those positions.
    /// - Reconstruct a `u16` as `(low, high)`.
    ///
    /// # Returns
    /// - `Some((instruction, instruction_pointer))` if both bytes exist.
    /// - `None` if arithmetic overflows or if either byte index is out of bounds.
    fn get_full_instruction_by_program_counter(
        &self,
        program_counter_value: u16,
    ) -> Option<(u16, u32)> {
        let (instruction_pointer, instr_high, instr_low) =
            match (program_counter_value as u32).checked_mul(2) {
                Some(high_pos) => match high_pos.checked_add(1) {
                    Some(low_pos) => (
                        low_pos,
                        match self.get(high_pos as usize) {
                            Some(byte) => *byte,
                            None => return None,
                        },
                        match self.get(low_pos as usize) {
                            Some(byte) => *byte,
                            None => return None,
                        },
                    ),
                    None => return None,
                },
                None => return None,
            };

        Some((
            unsafe { transmute::<(u8, u8), u16>((instr_low, instr_high)) },
            instruction_pointer,
        ))
    }
}
