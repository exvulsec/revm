use blst::{blst_p1, blst_p1_affine, blst_p1_from_affine, blst_p1_mult, blst_p1_to_affine};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

use crate::{u64_to_address, PrecompileWithAddress};

use super::{
    g1::{encode_g1_point, extract_g1_input, G1_INPUT_ITEM_LENGTH, G1_OUTPUT_LENGTH},
    utils::{extract_scalar_input, NBITS},
};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1MUL precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g1_mul));
/// BLS12_G1MUL precompile address.
pub const ADDRESS: u64 = 0x0c;
/// Base gas fee for BLS12-381 g1_mul operation.
pub(super) const BASE_GAS_FEE: u64 = 12000;

/// Input length of g1_mul operation.
pub(super) const INPUT_LENGTH: usize = 160;

/// G1 multiplication call expects `160` bytes as an input that is interpreted as
/// byte concatenation of encoding of G1 point (`128` bytes) and encoding of a
/// scalar value (`32` bytes).
/// Output is an encoding of multiplication operation result - single G1 point
/// (`128` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-multiplication>
pub fn g1_mul(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1MUL Input should be {INPUT_LENGTH} bits, was {}",
            input.len()
        )));
    }

    let mut p0_aff: blst_p1_affine = Default::default();
    let p0_aff = extract_g1_input(&mut p0_aff, &input[..G1_INPUT_ITEM_LENGTH])?;
    let mut p0: blst_p1 = Default::default();
    // SAFETY: p0 and p0_aff are blst values.
    unsafe {
        blst_p1_from_affine(&mut p0, p0_aff);
    }

    let input_scalar0 = extract_scalar_input(&input[G1_INPUT_ITEM_LENGTH..])?;

    let mut p: blst_p1 = Default::default();
    // SAFETY: input_scalar0.b has fixed size, p and p0 are blst values.
    unsafe {
        blst_p1_mult(&mut p, &p0, input_scalar0.b.as_ptr(), NBITS);
    }
    let mut p_aff: blst_p1_affine = Default::default();
    // SAFETY: p_aff and p are blst values.
    unsafe {
        blst_p1_to_affine(&mut p_aff, &p);
    }

    let mut out = [0u8; G1_OUTPUT_LENGTH];
    encode_g1_point(&mut out, &p_aff);

    Ok((BASE_GAS_FEE, out.into()))
}