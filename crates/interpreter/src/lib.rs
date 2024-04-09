//! # revm-interpreter
//!
//! REVM Interpreter.
#![warn(rustdoc::all)]
#![warn(unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[macro_use]
mod macros;

// silence lint
#[cfg(test)]
use serde_json as _;

#[cfg(test)]
use walkdir as _;

mod function_stack;
pub mod gas;
mod host;
mod instruction_result;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_action;
pub mod opcode;

// Reexport primary types.
pub use function_stack::{FunctionReturnFrame, FunctionStack};
pub use gas::Gas;
pub use host::{DummyHost, Host, LoadAccountResult, SStoreResult, SelfDestructResult};
pub use instruction_result::*;
pub use interpreter::{
    analysis, next_multiple_of_32, Contract, Interpreter, InterpreterAction, InterpreterResult,
    SharedMemory, Stack, EMPTY_SHARED_MEMORY, STACK_LIMIT,
};
pub use interpreter_action::{
    CallInputs, CallOutcome, CallScheme, CreateInputs, CreateOutcome, CreateScheme, EOFCreateInput,
    EOFCreateOutcome, TransferValue,
};
pub use opcode::{Instruction, OpCode, OPCODE_INFO_JUMPTABLE};
pub use primitives::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};

#[doc(hidden)]
pub use revm_primitives as primitives;
