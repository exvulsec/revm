use crate::primitives::{specification, EVMError, EVMResult, Env, ExecutionResult, SpecId};
use crate::{
    db::{Database, DatabaseCommit, DatabaseRef},
    evm_impl::{EVMImpl, Transact},
    inspectors::NoOpInspector,
    Inspector,
};
use alloc::boxed::Box;
use revm_interpreter::primitives::db::WrapDatabaseRef;
use revm_interpreter::primitives::ResultAndState;
use revm_precompile::Precompiles;

/// Struct that takes Database and enabled transact to update state directly to database.
/// additionally it allows user to set all environment parameters.
///
/// Parameters that can be set are divided between Config, Block and Transaction(tx)
///
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
///
/// You can do a lot with rust and traits. For Database abstractions that we need you can implement,
/// Database, DatabaseRef or Database+DatabaseCommit and they enable functionality depending on what kind of
/// handling of struct you want.
/// * Database trait has mutable self in its functions. It is usefully if on get calls you want to modify
/// your cache or update some statistics. They enable `transact` and `inspect` functions
/// * DatabaseRef takes reference on object, this is useful if you only have reference on state and don't
/// want to update anything on it. It enabled `transact_ref` and `inspect_ref` functions
/// * Database+DatabaseCommit allow directly committing changes of transaction. it enabled `transact_commit`
/// and `inspect_commit`
///
/// /// # Example
///
/// ```
/// # use revm::EVM;        // Assuming this struct is in 'your_crate_name'
/// # struct SomeDatabase;  // Mocking a database type for the purpose of this example
/// # struct Env;           // Assuming the type Env is defined somewhere
///
/// let evm: EVM<SomeDatabase> = EVM::new(SomeDatabase);
/// ```
///
#[derive(Clone)]
pub struct EVM<DB> {
    pub env: Env,
    pub db: DB,
}

pub fn new<DB>(db: DB) -> EVM<DB> {
    EVM::new(db)
}

impl<DB: Default> Default for EVM<DB> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<DB: Database + DatabaseCommit> EVM<DB> {
    /// Execute transaction and apply result to database
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.db.commit(state);
        Ok(result)
    }

    /// Inspect transaction and commit changes to database.
    pub fn inspect_commit<INSP: Inspector<DB>>(
        &mut self,
        inspector: INSP,
    ) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.inspect(inspector)?;
        self.db.commit(state);
        Ok(result)
    }
}

impl<DB: Database> EVM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        evm_inner::<DB, false>(&mut self.env, &mut self.db, &mut NoOpInspector)
            .preverify_transaction()
    }

    /// Skip preverification steps and execute transaction without writing to DB, return change
    /// state.
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        evm_inner::<DB, false>(&mut self.env, &mut self.db, &mut NoOpInspector)
            .transact_preverified()
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        evm_inner::<DB, false>(&mut self.env, &mut self.db, &mut NoOpInspector).transact()
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector<DB>>(&mut self, mut inspector: INSP) -> EVMResult<DB::Error> {
        evm_inner::<DB, true>(&mut self.env, &mut self.db, &mut inspector).transact()
    }
}

impl<'a, DB: DatabaseRef> EVM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction_ref(&self) -> Result<(), EVMError<DB::Error>> {
        evm_inner::<_, false>(
            &mut self.env.clone(),
            &mut WrapDatabaseRef(&self.db),
            &mut NoOpInspector,
        )
        .preverify_transaction()
    }

    /// Skip preverification steps and execute transaction
    /// without writing to DB, return change state.
    pub fn transact_preverified_ref(&self) -> EVMResult<DB::Error> {
        evm_inner::<_, false>(
            &mut self.env.clone(),
            &mut WrapDatabaseRef(&self.db),
            &mut NoOpInspector,
        )
        .transact_preverified()
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&self) -> EVMResult<DB::Error> {
        evm_inner::<_, false>(
            &mut self.env.clone(),
            &mut WrapDatabaseRef(&self.db),
            &mut NoOpInspector,
        )
        .transact()
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect_ref<I: Inspector<WrapDatabaseRef<&'a DB>>>(
        &'a self,
        mut inspector: I,
    ) -> EVMResult<DB::Error> {
        evm_inner::<_, true>(
            &mut self.env.clone(),
            &mut WrapDatabaseRef(&self.db),
            &mut inspector,
        )
        .transact()
    }
}

impl<DB> EVM<DB> {
    /// Creates a new [EVM] instance with the default environment,
    pub fn new(db: DB) -> Self {
        Self::with_env(Default::default(), db)
    }

    /// Creates a new [EVM] instance with the given environment.
    pub fn with_env(env: Env, db: DB) -> Self {
        Self { env, db }
    }

    pub fn db(&mut self) -> &mut DB {
        &mut self.db
    }
}

pub fn to_precompile_id(spec_id: SpecId) -> revm_precompile::SpecId {
    match spec_id {
        SpecId::FRONTIER
        | SpecId::FRONTIER_THAWING
        | SpecId::HOMESTEAD
        | SpecId::DAO_FORK
        | SpecId::TANGERINE
        | SpecId::SPURIOUS_DRAGON => revm_precompile::SpecId::HOMESTEAD,
        SpecId::BYZANTIUM | SpecId::CONSTANTINOPLE | SpecId::PETERSBURG => {
            revm_precompile::SpecId::BYZANTIUM
        }
        SpecId::ISTANBUL | SpecId::MUIR_GLACIER => revm_precompile::SpecId::ISTANBUL,
        SpecId::BERLIN
        | SpecId::LONDON
        | SpecId::ARROW_GLACIER
        | SpecId::GRAY_GLACIER
        | SpecId::MERGE
        | SpecId::SHANGHAI
        | SpecId::CANCUN
        | SpecId::LATEST => revm_precompile::SpecId::BERLIN,
    }
}

pub fn evm_inner<'a, DB: Database, const INSPECT: bool>(
    env: &'a mut Env,
    db: &'a mut DB,
    insp: &'a mut dyn Inspector<DB>,
) -> Box<dyn Transact<DB::Error> + 'a> {
    macro_rules! create_evm {
        ($spec:ident) => {
            Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
                db,
                env,
                insp,
                Precompiles::new(to_precompile_id($spec::SPEC_ID)).clone(),
            )) as Box<dyn Transact<DB::Error> + 'a>
        };
    }

    use specification::*;
    match env.cfg.spec_id {
        SpecId::FRONTIER | SpecId::FRONTIER_THAWING => create_evm!(FrontierSpec),
        SpecId::HOMESTEAD | SpecId::DAO_FORK => create_evm!(HomesteadSpec),
        SpecId::TANGERINE => create_evm!(TangerineSpec),
        SpecId::SPURIOUS_DRAGON => create_evm!(SpuriousDragonSpec),
        SpecId::BYZANTIUM => create_evm!(ByzantiumSpec),
        SpecId::PETERSBURG | SpecId::CONSTANTINOPLE => create_evm!(PetersburgSpec),
        SpecId::ISTANBUL | SpecId::MUIR_GLACIER => create_evm!(IstanbulSpec),
        SpecId::BERLIN => create_evm!(BerlinSpec),
        SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => {
            create_evm!(LondonSpec)
        }
        SpecId::MERGE => create_evm!(MergeSpec),
        SpecId::SHANGHAI => create_evm!(ShanghaiSpec),
        SpecId::CANCUN => create_evm!(CancunSpec),
        SpecId::LATEST => create_evm!(LatestSpec),
    }
}
