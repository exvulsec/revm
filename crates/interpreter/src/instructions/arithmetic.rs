use super::i256::{i256_div, i256_mod};
use super::prelude::*;

pub(super) fn wrapped_add(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.wrapping_add(*op2);
}

pub(super) fn wrapping_mul(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.wrapping_mul(*op2);
}

pub(super) fn wrapping_sub(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.wrapping_sub(*op2);
}

pub(super) fn div(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    if *op2 != U256::ZERO {
        *op2 = op1.wrapping_div(*op2);
    }
}

pub(super) fn sdiv(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    *op2 = i256_div(op1, *op2);
}

pub(super) fn rem(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    if *op2 != U256::ZERO {
        *op2 = op1.wrapping_rem(*op2);
    }
}

pub(super) fn smod(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    if *op2 != U256::ZERO {
        *op2 = i256_mod(op1, *op2)
    }
}

pub(super) fn addmod(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::MID);
    pop_top!(interpreter, op1, op2, op3);
    *op3 = op1.add_mod(op2, *op3)
}

pub(super) fn mulmod(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::MID);
    pop_top!(interpreter, op1, op2, op3);
    *op3 = op1.mul_mod(op2, *op3)
}

pub(super) fn eval_exp(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    pop_top!(interpreter, op1, op2);
    gas_or_fail!(interpreter, gas::exp_cost(*op2, spec));
    *op2 = op1.pow(*op2);
}

/// In the yellow paper `SIGNEXTEND` is defined to take two inputs, we will call them
/// `x` and `y`, and produce one output. The first `t` bits of the output (numbering from the
/// left, starting from 0) are equal to the `t`-th bit of `y`, where `t` is equal to
/// `256 - 8(x + 1)`. The remaining bits of the output are equal to the corresponding bits of `y`.
/// Note: if `x >= 32` then the output is equal to `y` since `t <= 0`. To efficiently implement
/// this algorithm in the case `x < 32` we do the following. Let `b` be equal to the `t`-th bit
/// of `y` and let `s = 255 - t = 8x + 7` (this is effectively the same index as `t`, but
/// numbering the bits from the right instead of the left). We can create a bit mask which is all
/// zeros up to and including the `t`-th bit, and all ones afterwards by computing the quantity
/// `2^s - 1`. We can use this mask to compute the output depending on the value of `b`.
/// If `b == 1` then the yellow paper says the output should be all ones up to
/// and including the `t`-th bit, followed by the remaining bits of `y`; this is equal to
/// `y | !mask` where `|` is the bitwise `OR` and `!` is bitwise negation. Similarly, if
/// `b == 0` then the yellow paper says the output should start with all zeros, then end with
/// bits from `b`; this is equal to `y & mask` where `&` is bitwise `AND`.
pub(super) fn signextend(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::LOW);
    pop_top!(interpreter, op1, op2);
    if op1 < U256::from(32) {
        // `low_u32` works since op1 < 32
        let bit_index = (8 * op1.as_limbs()[0] + 7) as usize;
        let bit = op2.bit(bit_index);
        let mask = (U256::from(1) << bit_index) - U256::from(1);
        *op2 = if bit { *op2 | !mask } else { *op2 & mask };
    }
}
