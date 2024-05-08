use blst::{blst_p1, blst_p1_affine, blst_p1_from_affine, blst_p1_to_affine, p1_affines};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

use crate::{u64_to_address, PrecompileWithAddress};

use super::{
    g1::{encode_g1_point, extract_g1_input, G1_INPUT_ITEM_LENGTH, G1_OUTPUT_LENGTH},
    g1_mul,
    msm::msm_required_gas,
    utils::{extract_scalar_input, NBITS, SCALAR_LENGTH},
};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g1_msm));
/// BLS12_G1MSM precompile address.
pub const ADDRESS: u64 = 0x0d;

/// Implements EIP-2537 G1MSM precompile.
/// G1 multi-scalar-multiplication call expects `160*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G1 point (`128` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G1
/// point (`128` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-multiexponentiation>
fn g1_msm(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % g1_mul::INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G1MSM input length should be multiple of {}, was {}",
            g1_mul::INPUT_LENGTH,
            input_len
        )));
    }

    let k = input_len / g1_mul::INPUT_LENGTH;
    let required_gas = msm_required_gas(k, g1_mul::BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let mut g1_points: Vec<blst_p1> = Vec::with_capacity(k);
    let mut scalars: Vec<u8> = Vec::with_capacity(k * SCALAR_LENGTH);
    for i in 0..k {
        let mut p0_aff: blst_p1_affine = Default::default();
        let p0_aff = extract_g1_input(
            &mut p0_aff,
            &input[i * g1_mul::INPUT_LENGTH..i * g1_mul::INPUT_LENGTH + G1_INPUT_ITEM_LENGTH],
        )?;
        let mut p0: blst_p1 = Default::default();
        // SAFETY: p0 and p0_aff are blst values.
        unsafe {
            blst_p1_from_affine(&mut p0, p0_aff);
        }

        g1_points.push(p0);

        scalars.extend_from_slice(
            &extract_scalar_input(
                &input[i * g1_mul::INPUT_LENGTH + G1_INPUT_ITEM_LENGTH
                    ..i * g1_mul::INPUT_LENGTH + G1_INPUT_ITEM_LENGTH + SCALAR_LENGTH],
            )?
            .b,
        );
    }

    let points = p1_affines::from(&g1_points);
    let multiexp = points.mult(&scalars, NBITS);

    let mut multiexp_aff: blst_p1_affine = Default::default();
    // SAFETY: multiexp_aff and multiexp are blst values.
    unsafe {
        blst_p1_to_affine(&mut multiexp_aff, &multiexp);
    }

    let mut out = [0u8; G1_OUTPUT_LENGTH];
    encode_g1_point(&mut out, &multiexp_aff);

    Ok((required_gas, out.into()))
}