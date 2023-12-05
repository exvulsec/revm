//! Mainnet related handlers.

pub mod frames;
pub mod host;
pub mod main;
pub mod preexecution;

use crate::{
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    primitives::{
        db::Database,
        EVMError, Env, Spec,
        SpecId::{CANCUN, LONDON},
        TransactTo, U256,
    },
    Context,
};

/// Handle output of the transaction
#[inline]
pub fn handle_call_return<SPEC: Spec>(
    env: &Env,
    call_result: InstructionResult,
    returned_gas: Gas,
) -> Gas {
    let tx_gas_limit = env.tx.gas_limit;
    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    let mut gas = Gas::new(tx_gas_limit);
    gas.record_cost(tx_gas_limit);

    match call_result {
        return_ok!() => {
            gas.erase_cost(returned_gas.remaining());
            gas.record_refund(returned_gas.refunded());
        }
        return_revert!() => {
            gas.erase_cost(returned_gas.remaining());
        }
        _ => {}
    }
    gas
}

#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(context.evm.env.tx.caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(context.evm.env.tx.gas_limit)
        .saturating_mul(context.evm.env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = context.evm.env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // touch account so we know it is changed.
    caller_account.mark_touch();

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if matches!(context.evm.env.tx.transact_to, TransactTo::Call(_)) {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    Ok(())
}

#[inline]
pub fn handle_reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let caller = context.evm.env.tx.caller;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64));

    Ok(())
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let beneficiary = context.evm.env.block.coinbase;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if SPEC::enabled(LONDON) {
        effective_gas_price.saturating_sub(context.evm.env.block.basefee)
    } else {
        effective_gas_price
    };

    let (coinbase_account, _) = context
        .evm
        .journaled_state
        .load_account(beneficiary, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spend() - gas.refunded() as u64));

    Ok(())
}

/// Calculate gas refund for transaction.
///
/// If config is set to disable gas refund, it will return 0.
///
/// If spec is set to london, it will decrease the maximum refund amount to 5th part of
/// gas spend. (Before london it was 2th part of gas spend)
#[inline]
pub fn calculate_gas_refund<SPEC: Spec>(env: &Env, gas: &Gas) -> u64 {
    if env.cfg.is_gas_refund_disabled() {
        0
    } else {
        // EIP-3529: Reduction in refunds
        let max_refund_quotient = if SPEC::enabled(LONDON) { 5 } else { 2 };
        (gas.refunded() as u64).min(gas.spend() / max_refund_quotient)
    }
}

#[cfg(test)]
mod tests {
    use revm_interpreter::primitives::CancunSpec;

    use super::*;

    #[test]
    fn test_consume_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let gas = handle_call_return::<CancunSpec>(&env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let mut return_gas = Gas::new(90);
        return_gas.record_refund(30);

        let gas = handle_call_return::<CancunSpec>(&env, InstructionResult::Stop, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 30);

        let gas = handle_call_return::<CancunSpec>(&env, InstructionResult::Revert, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_revert_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let gas = handle_call_return::<CancunSpec>(&env, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }
}
