//! # revm-primitives
//!
//! EVM primitive types.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]
#![feature(error_in_core)]

extern crate alloc;

mod bytecode;
mod constants;
pub mod db;
pub mod env;
#[cfg(feature = "c-kzg")]
pub mod kzg;
mod log;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;

pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, ruint, uint, Address, Bytes,
    FixedBytes, B256, U256,
};
pub use bitvec;
pub use bytecode::*;
pub use constants::*;
pub use env::*;
pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
#[cfg(feature = "c-kzg")]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use log::*;
pub use precompile::*;
pub use result::*;
pub use specification::*;
pub use state::*;
pub use utilities::*;
