//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    precompile::PrecompileSpecId,
    primitives::{
        Account, Block, EVMError, EVMResultGeneric, EnvWiring, Spec, SpecId, Transaction,
        BLOCKHASH_STORAGE_ADDRESS, KECCAK_EMPTY, U256,
    },
    Context, ContextPrecompiles, EvmWiring,
};
use std::vec::Vec;

/// Main precompile load
#[inline]
pub fn load_precompiles<EvmWiringT: EvmWiring, SPEC: Spec>() -> ContextPrecompiles<EvmWiringT> {
    ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
}

/// Main load handle
#[inline]
pub fn load_accounts<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    // set journaling state flag.
    context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

    // load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if SPEC::enabled(SpecId::SHANGHAI) {
        let coinbase = *context.evm.inner.env.block.coinbase();
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(coinbase);
    }

    // Load blockhash storage address
    // EIP-2935: Serve historical block hashes from state
    if SPEC::enabled(SpecId::PRAGUE) {
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(BLOCKHASH_STORAGE_ADDRESS);
    }

    // EIP-7702. Load bytecode to authorized accounts.
    if SPEC::enabled(SpecId::PRAGUE) {
        if let Some(authorization_list) = context.evm.inner.env.tx.authorization_list().as_ref() {
            let mut valid_auths = Vec::with_capacity(authorization_list.len());
            for authorization in authorization_list.recovered_iter() {
                // 1. recover authority and authorized addresses.
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
                    .load_account(authority, &mut context.evm.inner.db)
                    .map_err(EVMError::Database)?;

                // 3. Verify that the code of authority is empty.
                // In case of multiple same authorities this step will skip loading of
                // authorized account.
                if authority_acc.info.code_hash() != KECCAK_EMPTY {
                    continue;
                }

                // 4. If nonce list item is length one, verify the nonce of authority is equal to nonce.
                if let Some(nonce) = authorization.nonce() {
                    if nonce != authority_acc.info.nonce {
                        continue;
                    }
                }

                // warm code account and get the code.
                // 6. Add the authority account to accessed_addresses
                let (account, _) = context
                    .evm
                    .inner
                    .journaled_state
                    .load_code(authorization.address, &mut context.evm.inner.db)
                    .map_err(EVMError::Database)?;

                let code = account.info.code.clone();
                let code_hash = account.info.code_hash;

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

                valid_auths.push(authority);
            }

            context.evm.inner.valid_authorizations = valid_auths;
        }
    }

    context.evm.load_access_list().map_err(EVMError::Database)?;
    Ok(())
}

/// Helper function that deducts the caller balance.
#[inline]
pub fn deduct_caller_inner<EvmWiringT: EvmWiring, SPEC: Spec>(
    caller_account: &mut Account,
    env: &EnvWiring<EvmWiringT>,
) {
    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit()).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(SpecId::CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if env.tx.kind().is_call() {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();
}

/// Deducts the caller balance to the transaction limit.
#[inline]
pub fn deduct_caller<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(
            *context.evm.inner.env.tx.caller(),
            &mut context.evm.inner.db,
        )
        .map_err(EVMError::Database)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<EvmWiringT, SPEC>(caller_account, &context.evm.inner.env);

    Ok(())
}
