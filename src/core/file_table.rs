use std::collections::HashMap;

/// Represents the file table used by the `Star` virtual machine.
///
/// This type maps internal file identifiers to their corresponding source
/// file paths or names. It is primarily used to associate instructions and
/// positions with their originating source files.
///
/// # Semantics
/// - The key (`usize`) is an internal file identifier assigned by the VM.
/// - The value (`String`) is the file path or name associated with that id.
///
/// # Usage
/// The file table is leveraged for diagnostics, error reporting, and debugging,
/// allowing the VM to report precise source locations across multiple files.
pub type StarFileTable = HashMap<usize, String>;