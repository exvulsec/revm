//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    precompile::PrecompileSpecId,
    primitives::{
        db::Database,
        Account, EVMError, Env, Spec,
        SpecId::{CANCUN, PRAGUE, SHANGHAI},
        TxKind, BLOCKHASH_STORAGE_ADDRESS, KECCAK_EMPTY, U256,
    },
    Context, ContextPrecompiles,
};
use std::vec::Vec;

/// Main precompile load
#[inline]
pub fn load_precompiles<SPEC: Spec, DB: Database>() -> ContextPrecompiles<DB> {
    ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
}

/// Main load handle
#[inline]
pub fn load_accounts<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // set journaling state flag.
    context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

    // load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if SPEC::enabled(SHANGHAI) {
        let coinbase = context.evm.inner.env.block.coinbase;
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(coinbase);
    }

    // Load blockhash storage address
    // EIP-2935: Serve historical block hashes from state
    if SPEC::enabled(PRAGUE) {
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(BLOCKHASH_STORAGE_ADDRESS);
    }

    // EIP-7702. Load bytecode to authorized accounts.
    if SPEC::enabled(PRAGUE) {
        if let Some(authorization_list) = context.evm.inner.env.tx.authorization_list.as_ref() {
            for authorization in authorization_list.recovered_iter() {
                // 1. recover authority and authorized addresses.
                // authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]
                let Some(authority) = authorization.authority() else {
                    continue;
                };

                // 2. Verify the chain id is either 0 or the chain's current ID.
                if authorization.chain_id() != 0
                    && authorization.chain_id() != context.evm.inner.env.cfg.chain_id
                {
                    continue;
                }

                // warm authority account and check nonce.
                let (authority_acc, _) = context
                    .evm
                    .inner
                    .journaled_state
                    .load_code(authority, &mut context.evm.inner.db)?;

                // 3. Verify the code of authority is either empty or already delegated.
                if let Some(bytecode) = authority_acc.info.code {
                    // if it is not empty or it is not eip7702
                    if !bytecode.is_empty() || !bytecode.is_eip7702() {
                        continue;
                    }
                }

                // TODO In case of signer setting its own delegation check when
                //      Signer nonce is checked and bumped!
                // 4. If nonce list item is length one, verify the nonce of authority is equal to nonce.
                if let Some(nonce) = authorization.nonce() {
                    if nonce != authority_acc.info.nonce {
                        continue;
                    }
                }

                // If code is empty no need to set code or add it to valid
                // authorizations, as it is a noop operation.
                if code_hash == KECCAK_EMPTY {
                    continue;
                }

                // 5. Set the code of authority to code associated with address.
                context.evm.inner.journaled_state.set_code_with_hash(
                    authority,
                    code.unwrap_or_default(),
                    code_hash,
                );

                // TODO(EIP-7702) set contract to account.
            }
        }
    }

    // Load access list
    context.evm.load_access_list()?;
    Ok(())
}

/// Helper function that deducts the caller balance.
#[inline]
pub fn deduct_caller_inner<SPEC: Spec>(caller_account: &mut Account, env: &Env) {
    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if matches!(env.tx.transact_to, TxKind::Call(_)) {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();
}

/// Deducts the caller balance to the transaction limit.
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<SPEC>(caller_account, &context.evm.inner.env);

    Ok(())
}
