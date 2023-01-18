//! BlockHash database component from [`crate::Database`]
//! it is used inside [crate::DatabaseComponents`]

use crate::primitives::{B256, U256};
use auto_impl::auto_impl;

#[auto_impl(& mut, Box)]
pub trait BlockHash {
    type Error;

    /// Get block hash by block number
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error>;
}

#[auto_impl(&, Box, Arc)]
pub trait BlockHashRef {
    type Error;

    /// Get block hash by block number
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error>;
}

impl<T> BlockHash for &T
where
    T: BlockHashRef,
{
    type Error = <T as BlockHashRef>::Error;

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        BlockHashRef::block_hash(*self, number)
    }
}
