#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod gas;
mod host;
pub mod inner_models;
pub mod instruction_result;
mod instructions;
mod interpreter;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");

// Reexport primary types.
pub use gas::Gas;
pub use host::{DummyHost, Host};
pub use inner_models::*;
pub use instruction_result::InstructionResult;
pub use instructions::*;
pub use interpreter::*;
pub use interpreter::{BytecodeLocked, Contract, Interpreter, Memory, Stack};

#[doc(inline)]
pub use revm_primitives as primitives;
