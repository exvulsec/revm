use blst::{
    blst_fp, blst_fp_from_bendian, blst_map_to_g1, blst_p1, blst_p1_affine, blst_p1_to_affine,
};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

use crate::{u64_to_address, PrecompileWithAddress};

use super::{
    g1::{encode_g1_point, G1_OUTPUT_LENGTH},
    utils::{remove_padding, PADDED_FP_LENGTH},
};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP_TO_G1 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(map_fp_to_g1));
/// BLS12_MAP_FP_TO_G1 precompile address.
pub const ADDRESS: u64 = 0x12;
/// Base gas fee for BLS12-381 map_fp_to_g1 operation.
const MAP_FP_TO_G1_BASE: u64 = 5500;

/// Field-to-curve call expects 64 bytes as an input that is interpreted as an
/// element of Fp. Output of this call is 128 bytes and is an encoded G1 point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp-element-to-g1-point>
fn map_fp_to_g1(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if MAP_FP_TO_G1_BASE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP_TO_G1 Input should be {PADDED_FP_LENGTH} bits, was {}",
            input.len()
        )));
    }

    let input_p0 = match remove_padding(input) {
        Ok(input_p0) => input_p0,
        Err(e) => return Err(e),
    };

    let mut fp: blst_fp = Default::default();

    // SAFETY: input_p0 has fixed length, fp is a blst value.
    unsafe {
        blst_fp_from_bendian(&mut fp, input_p0.as_ptr());
    }

    let mut p: blst_p1 = Default::default();
    // SAFETY: p and fp are blst values.
    unsafe {
        // third argument is unused if null.
        blst_map_to_g1(&mut p, &fp, std::ptr::null());
    }

    let mut p_aff: blst_p1_affine = Default::default();
    // SAFETY: p_aff and p are blst values.
    unsafe {
        blst_p1_to_affine(&mut p_aff, &p);
    }

    let mut out = [0u8; G1_OUTPUT_LENGTH];
    encode_g1_point(&mut out, &p_aff);

    Ok((MAP_FP_TO_G1_BASE, out.into()))
}
