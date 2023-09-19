use crate::Address;

/// Interpreter stack limit
pub const STACK_LIMIT: u64 = 1024;
/// EVM call stack limit
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EIP-170: Contract code size limit
/// By default limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// Number of blocks hashes that EVM can access in the past
pub const BLOCK_HASH_HISTORY: usize = 256;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is 2 * MAX_CODE_SIZE
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

/// Precompile 3 is special in few places
pub const PRECOMPILE3: Address =
    Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
// EIP-4844 constants
/// Maximum consumable blob gas for data blobs per block.
pub const MAX_BLOB_GAS_PER_BLOCK: u64 = 6 * GAS_PER_BLOB;
/// Target consumable blob gas for data blobs per block (for 1559-like pricing).
pub const TARGET_BLOB_GAS_PER_BLOCK: u64 = 3 * GAS_PER_BLOB;
/// Gas consumption of a single data blob (== blob byte size).
pub const GAS_PER_BLOB: u64 = 1 << 17;
/// Minimum gas price for data blobs.
pub const MIN_BLOB_GASPRICE: u64 = 1;
/// Controls the maximum rate of change for blob gas price.
pub const BLOB_GASPRICE_UPDATE_FRACTION: u64 = 3338477;
