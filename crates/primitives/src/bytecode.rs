pub mod eof;
pub mod legacy;

use crate::{hex, keccak256, Bytes, B256, KECCAK_EMPTY};
use alloc::{sync::Arc, vec::Vec};
use bitvec::{
    prelude::{bitvec, Lsb0},
    vec::BitVec,
};
use core::fmt::Debug;
use legacy::JumpMap;

/// State of the [`Bytecode`] analysis.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeState {
    /// No analysis has been performed.
    Raw,
    /// The bytecode has been checked for validity.
    Checked { len: usize },
    /// The bytecode has been analyzed for valid jump destinations.
    Analysed { len: usize, jump_map: JumpMap },
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bytecode {
    pub bytecode: Bytes,
    pub state: BytecodeState,
}

impl Debug for Bytecode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bytecode")
            .field("bytecode", &self.bytecode)
            .field("state", &self.state)
            .finish()
    }
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        Bytecode::new()
    }
}

impl Bytecode {
    /// Creates a new [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        Bytecode {
            bytecode: Bytes::from_static(&[0]),
            state: BytecodeState::Analysed {
                len: 0,
                jump_map: JumpMap(Arc::new(bitvec![u8, Lsb0; 0])),
            },
        }
    }

    /// Calculate hash of the bytecode.
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(&self.original_bytes())
        }
    }

    /// Creates a new raw [`Bytecode`].
    #[inline]
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self {
            bytecode,
            state: BytecodeState::Raw,
        }
    }

    /// Create new checked bytecode
    ///
    /// # Safety
    ///
    /// Bytecode need to end with STOP (0x00) opcode as checked bytecode assumes
    /// that it is safe to iterate over bytecode without checking lengths
    pub unsafe fn new_checked(bytecode: Bytes, len: usize) -> Self {
        Self {
            bytecode,
            state: BytecodeState::Checked { len },
        }
    }

    /// Returns a reference to the bytecode.
    #[inline]
    pub fn bytes(&self) -> &Bytes {
        &self.bytecode
    }

    /// Returns a reference to the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self.state {
            BytecodeState::Raw => self.bytecode.clone(),
            BytecodeState::Checked { len } | BytecodeState::Analysed { len, .. } => {
                self.bytecode.slice(0..len)
            }
        }
    }

    /// Returns the length of the bytecode.
    #[inline]
    pub fn len(&self) -> usize {
        match self.state {
            BytecodeState::Raw => self.bytecode.len(),
            BytecodeState::Checked { len, .. } | BytecodeState::Analysed { len, .. } => len,
        }
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [`BytecodeState`].
    #[inline]
    pub fn state(&self) -> &BytecodeState {
        &self.state
    }

    pub fn to_checked(self) -> Self {
        match self.state {
            BytecodeState::Raw => {
                let len = self.bytecode.len();
                let mut padded_bytecode = Vec::with_capacity(len + 33);
                padded_bytecode.extend_from_slice(&self.bytecode);
                padded_bytecode.resize(len + 33, 0);
                Self {
                    bytecode: padded_bytecode.into(),
                    state: BytecodeState::Checked { len },
                }
            }
            _ => self,
        }
    }
}
