use crate::core::StarMcallContext;

/// Defines the contract for system-call (MCALL) interfaces that can be attached
/// to the `Star` virtual machine.
///
/// Implementors of this trait provide host-side functionality that can be
/// invoked from within the VM via the `Mcall` instruction. The interface is
/// executed with access to a restricted execution context (`StarMcallContext`)
/// rather than the full VM, ensuring encapsulation and safety.
///
/// # Execution model
/// - The VM invokes `mcall` when an `Mcall` instruction is executed.
/// - The interface may inspect and mutate registers and memory through the
///   provided context.
/// - The return value determines whether the VM execution loop should stop.
pub trait StarInterface {
    /// Handles a system call from the `Star` virtual machine.
    ///
    /// # Parameters
    /// - `s`: A mutable reference to a `StarMcallContext`, providing controlled
    ///   access to registers, memory, and execution metadata.
    ///
    /// # Returns
    /// - `true` if the VM execution loop should be terminated immediately.
    /// - `false` if execution should continue with the next instruction.
    ///
    /// # Notes
    /// Implementations should be careful to preserve VM invariants and avoid
    /// leaving the machine state in an inconsistent configuration.
    fn mcall(&mut self, s: &mut dyn StarMcallContext) -> bool;
}
