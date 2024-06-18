use revm_interpreter::gas;

use crate::{
    primitives::{
        db::Database, ChainSpec, EVMError, Env, InvalidTransaction, Spec, Transaction,
        TransactionValidation,
    },
    Context,
};

/// Validate environment for the mainnet.
pub fn validate_env<ChainSpecT: ChainSpec, SPEC: Spec, DB: Database>(
    env: &Env<ChainSpecT>,
) -> Result<
    (),
    EVMError<
        DB::Error,
        <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
    >,
>
where
    <ChainSpecT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()
        .map_err(|error| EVMError::Transaction(error.into()))?;
    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<ChainSpecT: ChainSpec, SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<ChainSpecT, EXT, DB>,
) -> Result<
    (),
    EVMError<
        DB::Error,
        <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
    >,
>
where
    <ChainSpecT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    // load acc
    let tx_caller = context.evm.env.tx.caller();
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(*tx_caller, &mut context.evm.inner.db)
        .map_err(EVMError::Database)?;

    context
        .evm
        .inner
        .env
        .validate_tx_against_state::<SPEC>(caller_account)
        .map_err(|error| EVMError::Transaction(error.into()))?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<ChainSpecT: ChainSpec, SPEC: Spec, DB: Database>(
    env: &Env<ChainSpecT>,
) -> Result<
    u64,
    EVMError<
        DB::Error,
        <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
    >,
>
where
    <ChainSpecT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    let input = &env.tx.data();
    let is_create = env.tx.kind().is_create();
    let access_list = &env.tx.access_list();

    let initial_gas_spend =
        gas::validate_initial_tx_gas(SPEC::SPEC_ID, input, is_create, access_list);

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > env.tx.gas_limit() {
        return Err(EVMError::Transaction(
            InvalidTransaction::CallGasCostMoreThanGasLimit.into(),
        ));
    }
    Ok(initial_gas_spend)
}
