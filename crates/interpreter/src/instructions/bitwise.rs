use super::i256::{i256_cmp, i256_sign_compl, two_compl, Sign};
use crate::{
    gas,
    primitives::{Spec, U256},
    Host, Interpreter,
};
use core::cmp::Ordering;
use revm_primitives::uint;

pub fn lt<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 < *op2);
}

pub fn gt<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 > *op2);
}

pub fn slt<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Less);
}

pub fn sgt<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Greater);
}

pub fn eq<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 == *op2);
}

pub fn iszero<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = U256::from(*op1 == U256::ZERO);
}

pub fn bitand<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 & *op2;
}

pub fn bitor<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 | *op2;
}

pub fn bitxor<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 ^ *op2;
}

pub fn not<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = !*op1;
}

pub fn byte<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let o1 = as_u64_saturated!(op1);
    const MAX_BYTE: u64 = 31;

    *op2 = if o1 < 32 {
        /*
			number := z[4-1-number/8]
			offset := (n[0] & 0x7) << 3 // 8*(n.d % 8)
			z[0] = (number & (0xff00000000000000 >> offset)) >> (56 - offset)
			z[3], z[2], z[1] = 0, 0, 0
			return z
         */
        // Calculate which byte to extract
        // Calculate bit position for the byte extraction
        let byte_index = (o1 * 8) as u64;
        let shift_amount = 248 - byte_index;

        // Shift the value down to the least significant byte and mask off the rest
        (*op2 >> shift_amount) & U256::from(0xFF)
    } else {
        U256::ZERO
    };
}

/// Converts a `U256` value to a `u64`, saturating to `MAX` if the value is too large.
#[macro_export]
macro_rules! as_u64_saturated {
    ($v:expr) => {{
        let x: &[u64; 4] = $v.as_limbs();
        if x[1] == 0 && x[2] == 0 && x[3] == 0 {
            x[0]
        } else {
            u64::MAX
        }
    }};
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn shl<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 <<= as_usize_saturated!(op1);
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn shr<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 >>= as_usize_saturated!(op1);
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn sar<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let value_sign = i256_sign_compl(op2);

    // If the shift count is 255+, we can short-circuit. This is because shifting by 255 bits is the
    // maximum shift that still leaves 1 bit in the original 256-bit number. Shifting by 256 bits or
    // more would mean that no original bits remain. The result depends on what the highest bit of
    // the value is.
    *op2 = if value_sign == Sign::Zero || op1 >= U256::from(255) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::ZERO,
            // value is <0, pushing -1
            Sign::Minus => U256::MAX,
        }
    } else {
        const ONE: U256 = uint!(1_U256);
        // SAFETY: shift count is checked above; it's less than 255.
        let shift = usize::try_from(op1).unwrap();
        match value_sign {
            Sign::Plus | Sign::Zero => op2.wrapping_shr(shift),
            Sign::Minus => two_compl(op2.wrapping_sub(ONE).wrapping_shr(shift).wrapping_add(ONE)),
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::instructions::bitwise::{sar, shl, shr};
    use crate::{Contract, DummyHost, Interpreter};
    use revm_primitives::{uint, Env, LatestSpec, U256};

    #[test]
    fn test_shift_left() {
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(Contract::default(), u64::MAX, false);

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
            let test_cases = [
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x00_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000002_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0xff_U256,
                    expected: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x0101_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x00_U256,
                    expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0xff_U256,
                    expected: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe_U256,
                },
            ];
        }

        for test in test_cases {
            host.clear();
            push!(interpreter, test.value);
            push!(interpreter, test.shift);
            shl::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, test.expected);
        }
    }

    #[test]
    fn test_logical_shift_right() {
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(Contract::default(), u64::MAX, false);

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
            let test_cases = [
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x00_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x4000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0xff_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x0101_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x00_U256,
                    expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0xff_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
            ];
        }

        for test in test_cases {
            host.clear();
            push!(interpreter, test.value);
            push!(interpreter, test.shift);
            shr::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, test.expected);
        }
    }

    #[test]
    fn test_arithmetic_shift_right() {
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(Contract::default(), u64::MAX, false);

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
        let test_cases = [
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                shift: 0x00_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                shift: 0x01_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x01_U256,
                expected: 0xc000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0xff_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x0100_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x0101_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x00_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x01_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xff_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x0100_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x01_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x4000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0xfe_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xf8_U256,
                expected: 0x000000000000000000000000000000000000000000000000000000000000007f_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xfe_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xff_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x0100_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
        ];
            }

        for test in test_cases {
            host.clear();
            push!(interpreter, test.value);
            push!(interpreter, test.shift);
            sar::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, test.expected);
        }
    }
}
