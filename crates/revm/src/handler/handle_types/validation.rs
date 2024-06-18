use crate::{
    handler::mainnet,
    primitives::{
        db::Database, ChainSpec, EVMError, Env, InvalidTransaction, Spec, TransactionValidation,
    },
    Context,
};
use std::sync::Arc;

/// Handle that validates env.
pub type ValidateEnvHandle<'a, ChainSpecT, DB> = Arc<
    dyn Fn(
            &Env<ChainSpecT>,
        ) -> Result<
            (),
            EVMError<
                <DB as Database>::Error,
                <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
            >,
        > + 'a,
>;

/// Handle that validates transaction environment against the state.
/// Second parametar is initial gas.
pub type ValidateTxEnvAgainstState<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
        ) -> Result<
            (),
            EVMError<
                <DB as Database>::Error,
                <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
            >,
        > + 'a,
>;

/// Initial gas calculation handle
pub type ValidateInitialTxGasHandle<'a, ChainSpecT, DB> = Arc<
    dyn Fn(
            &Env<ChainSpecT>,
        ) -> Result<
            u64,
            EVMError<
                <DB as Database>::Error,
                <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
            >,
        > + 'a,
>;

/// Handles related to validation.
pub struct ValidationHandler<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Validate and calculate initial transaction gas.
    pub initial_tx_gas: ValidateInitialTxGasHandle<'a, ChainSpecT, DB>,
    /// Validate transactions against state data.
    pub tx_against_state: ValidateTxEnvAgainstState<'a, ChainSpecT, EXT, DB>,
    /// Validate Env.
    pub env: ValidateEnvHandle<'a, ChainSpecT, DB>,
}

impl<'a, ChainSpecT: ChainSpec, EXT: 'a, DB: Database + 'a>
    ValidationHandler<'a, ChainSpecT, EXT, DB>
where
    <ChainSpecT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    /// Create new ValidationHandles
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            initial_tx_gas: Arc::new(mainnet::validate_initial_tx_gas::<ChainSpecT, SPEC, DB>),
            env: Arc::new(mainnet::validate_env::<ChainSpecT, SPEC, DB>),
            tx_against_state: Arc::new(
                mainnet::validate_tx_against_state::<ChainSpecT, SPEC, EXT, DB>,
            ),
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> ValidationHandler<'a, ChainSpecT, EXT, DB> {
    /// Validate env.
    pub fn env(
        &self,
        env: &Env<ChainSpecT>,
    ) -> Result<
        (),
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        (self.env)(env)
    }

    /// Initial gas
    pub fn initial_tx_gas(
        &self,
        env: &Env<ChainSpecT>,
    ) -> Result<
        u64,
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        (self.initial_tx_gas)(env)
    }

    /// Validate ttansaction against the state.
    pub fn tx_against_state(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
    ) -> Result<
        (),
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        (self.tx_against_state)(context)
    }
}
