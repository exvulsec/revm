use crate::{
    gas,
    primitives::{Spec, U256},
    Host, InstructionResult, Interpreter,
};

pub fn pop<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    if let Err(result) = interpreter.stack.pop() {
        interpreter.instruction_result = result;
    }
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, SHANGHAI);
    gas!(interpreter, gas::BASE);
    if let Err(result) = interpreter.stack.push(U256::ZERO) {
        interpreter.instruction_result = result;
    }
}

pub fn push<const N: usize, H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    // SAFETY: In analysis we append trailing bytes to the bytecode so that this is safe to do
    // without bounds checking.
    let ip = interpreter.instruction_pointer;
    if let Err(result) = interpreter
        .stack
        .push_slice(unsafe { core::slice::from_raw_parts(ip, N) })
    {
        interpreter.instruction_result = result;
        return;
    }
    interpreter.instruction_pointer = unsafe { ip.add(N) };
}

pub fn dup<const N: usize, H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    if let Err(result) = interpreter.stack.dup::<N>() {
        interpreter.instruction_result = result;
    }
}

pub fn swap<const N: usize, H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    if let Err(result) = interpreter.stack.swap::<N>() {
        interpreter.instruction_result = result;
    }
}

pub fn dupn<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {}

pub fn swapn<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {}

pub fn exchange<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {}
