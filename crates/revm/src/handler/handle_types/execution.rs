use crate::{
    handler::mainnet,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{db::Database, EVMError, Spec},
    CallFrame, Context, CreateFrame, Frame, FrameOrResult, FrameResult,
};
use std::{boxed::Box, sync::Arc};

use revm_interpreter::{CallOutcome, CreateOutcome, InterpreterResult};

/// Handles first frame return handle.
pub type LastFrameReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(&mut Context<EXT, DB>, &mut FrameResult) -> Result<(), EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handle sub call.
pub type FrameCallHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Box<CallInputs>,
        ) -> Result<FrameOrResult, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handle call return
pub type FrameCallReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Box<CallFrame>,
            InterpreterResult,
        ) -> Result<CallOutcome, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCallOutcomeHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            &mut Frame,
            &mut SharedMemory,
            CallOutcome,
        ) -> Result<(), EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handle sub create.
pub type FrameCreateHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Box<CreateInputs>,
        ) -> Result<FrameOrResult, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handle create return
pub type FrameCreateReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Box<CreateFrame>,
            InterpreterResult,
        ) -> Result<CreateOutcome, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCreateOutcomeHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            &mut Frame,
            CreateOutcome,
        ) -> Result<(), EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handles related to stack frames.
pub struct ExecutionHandler<EXT, DB: Database> {
    /// Handles last frame return, modified gas for refund and
    /// sets tx gas limit.
    // pub last_frame_return: LastFrameReturnHandle<'a, EXT, DB>,
    // /// Frame call
    // pub call: FrameCallHandle<'a, EXT, DB>,
    // /// Call return
    // pub call_return: FrameCallReturnHandle<'a, EXT, DB>,
    // /// Insert call outcome
    // pub insert_call_outcome: InsertCallOutcomeHandle<'a, EXT, DB>,
    // /// Frame crate
    // pub create: FrameCreateHandle<'a, EXT, DB>,
    // /// Crate return
    // pub create_return: FrameCreateReturnHandle<'a, EXT, DB>,
    // /// Insert create outcome.
    // pub insert_create_outcome: InsertCreateOutcomeHandle<'a, EXT, DB>,
    pub phantom: std::marker::PhantomData<(EXT, DB)>,
}

impl<EXT, DB: Database> ExecutionHandler<EXT, DB> {
    /// Creates mainnet ExecutionHandler.
    pub fn new<SPEC: Spec>() -> Self {
        Self {
            // last_frame_return: Arc::new(mainnet::last_frame_return::<SPEC, EXT, DB>),
            // call: Arc::new(mainnet::call::<SPEC, EXT, DB>),
            // call_return: Arc::new(mainnet::call_return::<EXT, DB>),
            // insert_call_outcome: Arc::new(mainnet::insert_call_outcome),
            // create: Arc::new(mainnet::create::<SPEC, EXT, DB>),
            // create_return: Arc::new(mainnet::create_return::<SPEC, EXT, DB>),
            // insert_create_outcome: Arc::new(mainnet::insert_create_outcome),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<EXT, DB: Database> ExecutionHandler<EXT, DB> {
    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    #[inline]
    pub fn last_frame_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame_result: &mut FrameResult,
    ) -> Result<(), EVMError<DB::Error>> {
        //(self.last_frame_return)(context, frame_result)
        Ok(())
    }

    /// Call frame call handler.
    #[inline]
    pub fn call(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        //(self.call)(context, inputs.clone())
        Err(EVMError::Custom("t".into()))
    }

    /// Call registered handler for call return.
    #[inline]
    pub fn call_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CallOutcome, EVMError<DB::Error>> {
        //(self.call_return)(context, frame, interpreter_result)
        Err(EVMError::Custom("t".into()))
    }

    /// Call registered handler for inserting call outcome.
    #[inline]
    pub fn insert_call_outcome(
        &self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> Result<(), EVMError<DB::Error>> {
        //(self.insert_call_outcome)(context, frame, shared_memory, outcome)
        Err(EVMError::Custom("t".into()))
    }

    /// Call Create frame
    #[inline]
    pub fn create(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        //(self.create)(context, inputs)
        Err(EVMError::Custom("t".into()))
    }

    /// Call handler for create return.
    #[inline]
    pub fn create_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CreateOutcome, EVMError<DB::Error>> {
        //(self.create_return)(context, frame, interpreter_result)
        Err(EVMError::Custom("t".into()))
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_create_outcome(
        &self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> Result<(), EVMError<DB::Error>> {
        //(self.insert_create_outcome)(context, frame, outcome)
        Err(EVMError::Custom("t".into()))
    }
}
