# Machine Calls

Machine calls are **special instructions** that allow a program running inside the virtual machine to **interact with the host system** and perform **operations outside the normal instruction set**.

They act as the primary bridge between the **guest program** and the **runtime environment**, enabling controlled access to features such as input/output, debugging, system services, and environment-dependent behavior.

## Purpose

The instruction set of the virtual machine is intentionally minimal and deterministic.
Machine calls exist to handle tasks that:

* Cannot be expressed purely with standard instructions
* Require interaction with the external environment
* Depend on platform-specific or runtime-specific behavior

Examples include:

* Printing to standard output
* Reading input
* Debugging or tracing execution
* Interfacing with external tools or subsystems

## Conceptual Model

A machine call is triggered by a **dedicated instruction** (for example, `mcall`) and is handled by a **machine call interface** provided by the runtime.

Conceptually, the flow looks like this:

1. The program executes a machine call instruction
2. Control is transferred to the machine call handler
3. The handler inspects registers and execution state
4. An external operation is performed
5. Control returns to the virtual machine

The virtual machine itself does **not** implement the behavior — it merely **delegates** execution.

## Machine Call Interface

Machine calls are resolved through a **machine call interface**, which defines how external logic is invoked.

Key characteristics:

* The interface is **runtime-provided**
* It receives controlled access to the machine state
* It may read or modify registers
* It may signal termination or continuation of execution

This design allows:

* Multiple runtime implementations
* Platform-specific behavior
* Test and debug environments without changing the core VM

## Register-Based Communication

Machine calls communicate with the program **exclusively through registers**.

Typically:

* One register defines the **machine call identifier**
* Other registers provide **arguments**
* Result values (if any) are written back to registers

This ensures:

* No hidden state
* Deterministic behavior
* Clear ABI-like conventions

The exact register usage depends on the runtime implementation and calling convention.

## Execution Semantics

From the VM’s perspective, a machine call is:

* A **synchronous operation**
* Executed atomically from the instruction stream
* Either completes normally or requests termination

The VM does not interpret the meaning of the call — it only:

* Pauses execution
* Invokes the handler
* Resumes execution based on the handler’s result

## Extensibility

Machine calls are designed to be **open-ended**.

New capabilities can be added by:

* Defining new machine call identifiers
* Implementing new handlers
* Keeping backward compatibility with existing calls

This allows the system to evolve without modifying:

* The instruction set
* The assembler
* Existing programs

## Summary

Machine calls provide a **controlled escape hatch** from the virtual machine into the external world.

They:

* Extend the VM without bloating the instruction set
* Enable I/O, debugging, and system interaction
* Preserve determinism and isolation through explicit interfaces

They are a foundational mechanism for building practical systems on top of a minimal virtual machine.

> For more information about the execution process, see the [execution stage documentation](/docs/execution-stage.md).