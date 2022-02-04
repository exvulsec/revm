use crate::{alloc::vec::Vec, util, Return};
use primitive_types::{H160, H256, U256};

pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone)]
pub struct Stack {
    data: Vec<U256>,
}

use std::fmt::{Display, Error, Formatter};
impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.data.is_empty() {
            f.write_str("[]")?;
        } else {
            f.write_str("[")?;
            for i in self.data[..self.data.len() - 1].iter() {
                f.write_str(&i.to_string())?;
                f.write_str(", ")?;
            }
            f.write_str(&self.data.last().unwrap().to_string())?;
            f.write_str("]")?;
        }
        Ok(())
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Create a new stack with given limit.
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(STACK_LIMIT),
        }
    }

    #[inline]
    /// Stack length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    /// Stack data.
    pub fn data(&self) -> &Vec<U256> {
        &self.data
    }

    #[inline(always)]
    pub fn reduce_one(&mut self) -> Result<(), Return> {
        self.pop().map(|_| ())
    }

    // XXX remove?
    #[inline(always)]
    pub fn top_mut(&mut self) -> Option<&mut U256> {
        self.data.last_mut()
    }

    #[inline(always)]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop(&mut self) -> Result<U256, Return> {
        self.data.pop().ok_or(Return::StackUnderflow)
    }

    #[inline(always)]
    pub fn pop2(&mut self) -> Result<(U256, U256), Return> {
        if let [.., b, a] = self.data[..] {
            self.data.truncate(self.data.len() - 2);
            Ok((a, b))
        } else {
            Err(Return::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn pop3(&mut self) -> Result<(U256, U256, U256), Return> {
        if let [.., c, b, a] = self.data[..] {
            self.data.truncate(self.data.len() - 3);
            Ok((a, b, c))
        } else {
            Err(Return::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn pop4(&mut self) -> Result<(U256, U256, U256, U256), Return> {
        if let [.., d, c, b, a] = self.data[..] {
            self.data.truncate(self.data.len() - 4);
            Ok((a, b, c, d))
        } else {
            Err(Return::StackUnderflow)
        }
    }

    #[inline(always)]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop_address(&mut self) -> Result<H160, Return> {
        let mut temp = H256::zero();
        self.pop()?.to_big_endian(temp.as_bytes_mut());
        Ok(temp.into())
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push_h256(&mut self, value: H256) -> Result<(), Return> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }
        self.data.push(util::be_to_u256(&value[..]));
        Ok(())
    }

    #[inline(always)]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push(&mut self, value: U256) -> Result<(), Return> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    #[inline(always)]
    pub fn push_unchecked(&mut self, value: U256) {
        self.data.push(value);
    }

    #[inline]
    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<U256, Return> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(Return::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn dup<const N: usize>(&mut self) -> Result<(), Return> {
        let len = self.data.len();
        if len < N {
            Err(Return::StackUnderflow)
        } else if len + 1 > STACK_LIMIT {
            Err(Return::StackOverflow)
        } else {
            self.data.push(self.data[len - N]);
            Ok(())
        }
    }

    #[inline(always)]
    pub fn swap<const N: usize>(&mut self) -> Result<(), Return> {
        let len = self.data.len();
        if len <= N {
            return Err(Return::StackUnderflow);
        }
        self.data.swap(len - 1, len - 1 - N);
        Ok(())
    }

    /// push slice onto memory it is expected to be max 32 bytes and be contains inside H256
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Result<(), Return> {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }

        let mut slot = U256::zero();
        let mut dangling = [0u8; 8];
        if N < 8 {
            dangling[8 - N..].copy_from_slice(slice);
            slot.0[0] = u64::from_be_bytes(dangling);
        } else if N < 16 {
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            if N != 8 {
                dangling[8 * 2 - N..].copy_from_slice(&slice[..N - 8]);
                slot.0[1] = u64::from_be_bytes(dangling);
            }
        } else if N < 24 {
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            slot.0[1] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 16, 8));
            if N != 16 {
                dangling[8 * 3 - N..].copy_from_slice(&slice[..N - 16]);
                slot.0[2] = u64::from_be_bytes(dangling);
            }
        } else {
            // M<32
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            slot.0[1] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 16, 8));
            slot.0[2] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 24, 8));
            if N == 32 {
                slot.0[3] = u64::from_be_bytes(*arrayref::array_ref!(slice, 0, 8));
            } else if N != 24 {
                dangling[8 * 4 - N..].copy_from_slice(&slice[..N - 24]);
                slot.0[3] = u64::from_be_bytes(dangling);
            }
        }
        self.data.push(slot);
        Ok(())
    }

    #[inline]
    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), Return> {
        if self.data.len() > no_from_top {
            let len = self.data.len();
            self.data[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(Return::StackUnderflow)
        }
    }
}
