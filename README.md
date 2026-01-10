<div align="center">
  <img src="/docs/images/star-logo.png" width="300" />
</div>

<h1 align="center">STAR VIRTUAL MACHINE LIB</h1>

A extensible 16-bit virtual machine and assembly programming language library designed for educational purposes.

---

## A Simple Interface

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

fn main() {
    let mut star = Star::default();
    star.set_interface(Box::new(SimpleInterface));

    star.load_memory_from_assembly_file(&"./program.asm".to_string()).unwrap();
    star.execute().unwrap();
    println!();
}
```
---

## Introduction to Star Interfacing

To get started with the **Star** interfacing system you can read the following documentation:
- [Interfaces](/docs/interfaces.md): Learn about how to define and use interfaces in the **Star**.

---

## Documentation
### Syntax
To learn about the syntax of the **Star**
you can read the following documentation:
- [Instructions](/docs/instructions.md): Learn about the available instructions in the **Star**.
- [Pseudo-Instructions](/docs/pseudo-intructions.md): Learn about pseudo-instructions and how they simplify assembly programming.
- [Registers](/docs/registers.md): Understand the registers used in the **Star** and their purposes.
- [Directives](/docs/directives.md): Understand the directives used in **Star**.
- [Processors](/docs/processors.md): Explore the processors that enhance the assembly language capabilities.
- [Machine Calls](/docs/machine-calls.md): Learn about system interaction.

### Memory
To understand the memory model of the **Star**
you can read the following documentation:
- [Instruction Memory](/docs/instruction-memory.md): Understand how instruction memory works and its role in the virtual machine.
- [Data Memory](/docs/data-memory.md): Learn about data memory and how it is used to store values.
- [Position Memory](/docs/position-memory.md): Learn about position memory and its importance for debugging and error reporting.

### Compilation Stages
To learn about how the **Star** compilation processes you can read the following documentation:
- [Scanner Stage](/docs/scanner-stage.md): Understand how the scanner reads source code and converts it into tokens.
- [Parser Stage](/docs/parser-stage.md): Learn how the parser analyzes tokens and constructs an abstract syntax tree (AST).
- [Resolver Stage](/docs/resolver-stage.md): Discover how the resolver processes the AST and resolves symbols.
- [Generation Stage](/docs/generation-stage.md): Understand how the assembler generates machine code
- [Execution Stage](/docs/execution-stage.md): Learn how the virtual machine executes the generated code.