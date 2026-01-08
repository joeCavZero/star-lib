# Interfaces

In the **Star Virtual Machine Library**, an **Interface** defines how the virtual machine responds to **machine calls (`mcall`)** made by a running program.

Interfaces are the main extension mechanism of the VM. They allow you to connect the virtual machine to the outside world (I/O, debugging, timing, randomness, etc.) **without modifying the VM core**.



## What Is an Interface?

An interface is a Rust type that implements the `StarInterface` trait.

### When the VM executes a machine call instruction:

1. Execution is paused
2. The active interface receives control
3. The interface inspects registers and memory
4. An external operation is performed
5. Control returns to the VM

This makes interfaces similar to **system call handlers** in operating systems.

### Typical use cases include:

* Input and output
* Debugging and tracing
* Timers and delays
* Random number generation
* Integration with external tools

 

## The `mcall` Function

Every interface must implement the following method:

```rust
fn mcall(&mut self, s: &mut dyn StarMcallContext) -> bool;
```

### Parameters

* `s`: A **machine call context**, which provides controlled access to:

  * General registers
  * Data memory
  * Instruction memory

### Return Value

* `false` → execution continues normally
* `true`  → execution is terminated immediately



## Register-Based Dispatch

Machine calls are identified using **register values**, not function arguments.

A common convention is:

* `aux1` → machine call identifier
* `aux2`, `aux3`, … → arguments
* return values written back to registers

The VM itself does not enforce this — it is defined by the interface.



## Minimal Example Interface

Below is a **small and didactic example** showing how to implement a simple interface with two machine calls:

* `1` → print an unsigned byte
* `2` → terminate execution

```rust
use star::prelude::*;
use std::io::Write;

pub struct SimpleInterface;

impl StarInterface for SimpleInterface {
    fn mcall(&mut self, s: &mut dyn StarMcallContext) -> bool {
        let registers = s.get_registers_mut().clone();
        match registers.aux1 {
            1 => { // Print byte in aux2
                let value = registers.aux2 as u8;
                print!("{value}");
                std::io::stdout().flush().unwrap();
            }
            2 => return true, // Exit program
            _ => {}
        }
        false
    }
}
```

## Attaching an Interface to the VM

To use an interface, attach it to the `Star` instance **before execution**:

```rust
fn main() {
    let mut star = Star::default();
    star.set_interface(Box::new(SimpleInterface));

    star.load_memory_from_assembly_file(&"./program.asm".to_string()).unwrap();
    star.execute().unwrap();
    println!();
}
```

Only **one interface** can be active at a time, but it can internally dispatch many machine calls.

> For more information about the execution process, see the [execution stage documentation](/docs/execution-stage.md).