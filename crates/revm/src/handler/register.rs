use crate::{
    db::Database,
    handler::Handler,
    inspector_instruction,
    interpreter::{
        opcode::{make_instruction_table, InstructionTables},
        InterpreterResult, SelfDestructResult,
    },
    primitives::Spec,
    CallStackFrame, Evm, FrameOrResult, Inspector,
};
use alloc::sync::Arc;
use core::marker::PhantomData;

/// Register external handles.
pub trait RegisterHandler<'a, DB: Database, EXT> {
    fn register_handler<SPEC: Spec>(
        &self,
        handler: Handler<'a, Evm<'a, SPEC, EXT, DB>, EXT, DB>,
    ) -> Handler<'a, Evm<'a, SPEC, EXT, DB>, EXT, DB>
    where
        DB: 'a,
        EXT: Sized,
    {
        handler
    }
}

/// Default registered handler that produces default mainnet handler.
#[derive(Default)]
pub struct MainnetHandle {}

impl<'a, DB: Database> RegisterHandler<'a, DB, Self> for MainnetHandle {}

pub struct OptimismHandle {}

impl<'a, DB: Database> RegisterHandler<'a, DB, ()> for OptimismHandle {
    fn register_handler<SPEC: Spec>(
        &self,
        handler: Handler<'a, Evm<'a, SPEC, (), DB>, (), DB>,
    ) -> Handler<'a, Evm<'a, SPEC, (), DB>, (), DB>
    where
        DB: 'a,
        (): Sized,
    {
        Handler::mainnet::<SPEC>()
    }
}

pub struct InspectorHandle<'a, DB: Database, GI: GetInspector<'a, DB>> {
    pub inspector: GI,
    pub _phantomdata: PhantomData<&'a DB>,
}

impl<'a, DB: Database, GI: GetInspector<'a, DB>> InspectorHandle<'a, DB, GI> {
    fn new(inspector: GI) -> Self {
        Self {
            inspector,
            _phantomdata: PhantomData,
        }
    }
}

pub trait GetInspector<'a, DB: Database> {
    fn get(&mut self) -> &mut dyn Inspector<DB>;
}

impl<'handler, DB: Database, INS: GetInspector<'handler, DB>> RegisterHandler<'handler, DB, Self>
    for InspectorHandle<'handler, DB, INS>
{
    fn register_handler<SPEC: Spec>(
        &self,
        mut handler: Handler<'handler, Evm<'handler, SPEC, Self, DB>, Self, DB>,
    ) -> Handler<'handler, Evm<'handler, SPEC, Self, DB>, Self, DB>
    where
        Self: Sized,
        DB: 'handler,
    {
        // flag instruction table that is going to be wrapped.
        let flat_instruction_table = make_instruction_table::<
            Evm<'handler, SPEC, InspectorHandle<'handler, DB, INS>, DB>,
            SPEC,
        >();

        // wrap instruction table with inspector handle.
        handler.instruction_table = InstructionTables::Boxed(Arc::new(core::array::from_fn(|i| {
            inspector_instruction(flat_instruction_table[i])
        })));

        // handle sub create
        handler.frame_sub_create = Arc::new(
            move |context, frame, mut inputs| -> Option<Box<CallStackFrame>> {
                if let Some((result, address)) = context
                    .external
                    .inspector
                    .get()
                    .create(&mut context.evm, &mut inputs)
                {
                    frame.interpreter.insert_create_output(result, address);
                    return None;
                }

                match context.evm.make_create_frame::<SPEC>(&inputs) {
                    FrameOrResult::Frame(new_frame) => Some(new_frame),
                    FrameOrResult::Result(result) => {
                        let (result, address) = context.external.inspector.get().create_end(
                            &mut context.evm,
                            result,
                            frame.created_address,
                        );
                        // insert result of the failed creation of create CallStackFrame.
                        frame.interpreter.insert_create_output(result, address);
                        None
                    }
                }
            },
        );

        // handle sub call
        handler.frame_sub_call = Arc::new(
            move |context, mut inputs, frame, memory, return_memory_offset| -> Option<Box<_>> {
                // inspector handle
                let inspector = &mut context.external.inspector.get();
                if let Some((result, range)) = inspector.call(&mut context.evm, &mut inputs) {
                    frame.interpreter.insert_call_output(memory, result, range);
                    return None;
                }
                match context
                    .evm
                    .make_call_frame(&inputs, return_memory_offset.clone())
                {
                    FrameOrResult::Frame(new_frame) => Some(new_frame),
                    FrameOrResult::Result(result) => {
                        // inspector handle
                        let result = context
                            .external
                            .inspector
                            .get()
                            .call_end(&mut context.evm, result);
                        frame
                            .interpreter
                            .insert_call_output(memory, result, return_memory_offset);
                        None
                    }
                }
            },
        );

        // return frame handle
        let old_handle = handler.frame_return.clone();
        handler.frame_return = Arc::new(
            move |context, mut child, parent, memory, mut result| -> Option<InterpreterResult> {
                let inspector = &mut context.external.inspector.get();
                result = if child.is_create {
                    let (result, address) =
                        inspector.create_end(&mut context.evm, result, child.created_address);
                    child.created_address = address;
                    result
                } else {
                    inspector.call_end(&mut context.evm, result)
                };
                let output = old_handle(context, child, parent, memory, result);
                output
            },
        );

        // handle log
        let old_handle = handler.host_log.clone();
        handler.host_log = Arc::new(move |context, address, topics, data| {
            context
                .external
                .inspector
                .get()
                .log(&mut context.evm, &address, &topics, &data);
            old_handle(context, address, topics, data)
        });

        // selfdestruct handle
        let old_handle = handler.host_selfdestruct.clone();
        handler.host_selfdestruct = Arc::new(
            move |context, address, target| -> Option<SelfDestructResult> {
                let inspector = &mut context.external.inspector.get();
                let acc = context.evm.journaled_state.state.get(&address).unwrap();
                inspector.selfdestruct(address, target, acc.info.balance);
                old_handle(context, address, target)
            },
        );

        handler
    }
}

// pub struct ExternalData<DB: Database> {
//     pub flagg: bool,
//     pub phantom: PhantomData<DB>,
// }

// impl<DB: Database> RegisterHandler<DB> for ExternalData<DB> {
//     fn register_handler<'a, SPEC: Spec>(
//         &self,
//         mut handler: Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>,
//     ) -> Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>
//     where
//         DB: 'a,
//     {
//         let old_handle = handler.reimburse_caller.clone();
//         handler.reimburse_caller = Arc::new(move |data, gas| old_handle(data, gas));
//         handler
//     }
// }
