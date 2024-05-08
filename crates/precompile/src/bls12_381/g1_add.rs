use blst::{
    blst_p1, blst_p1_add_or_double_affine, blst_p1_affine, blst_p1_from_affine, blst_p1_to_affine,
};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

use crate::{u64_to_address, PrecompileWithAddress};

use super::g1::{encode_g1_point, extract_g1_input, G1_INPUT_ITEM_LENGTH, G1_OUTPUT_LENGTH};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g1_add));
/// BLS12_G1ADD precompile address.
pub const ADDRESS: u64 = 0x0b;
/// Base gas fee for BLS12-381 g1_add operation.
const BASE_GAS_FEE: u64 = 500;

/// Input length of g1_add operation.
const INPUT_LENGTH: usize = 256;

/// G1 addition call expects `256` bytes as an input that is interpreted as byte
/// concatenation of two G1 points (`128` bytes each).
/// Output is an encoding of addition operation result - single G1 point (`128`
/// bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-addition>
fn g1_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1ADD Input should be {INPUT_LENGTH} bits, was {}",
            input.len()
        )));
    }

    let mut a_aff: blst_p1_affine = Default::default();
    let a_aff = extract_g1_input(&mut a_aff, &input[..G1_INPUT_ITEM_LENGTH])?;

    let mut b_aff: blst_p1_affine = Default::default();
    let b_aff = extract_g1_input(&mut b_aff, &input[G1_INPUT_ITEM_LENGTH..])?;

    let mut b: blst_p1 = Default::default();
    // SAFETY: b and b_aff are blst values.
    unsafe {
        blst_p1_from_affine(&mut b, b_aff);
    }

    let mut p: blst_p1 = Default::default();
    // SAFETY: p, b and a_aff are blst values.
    unsafe {
        blst_p1_add_or_double_affine(&mut p, &b, a_aff);
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
