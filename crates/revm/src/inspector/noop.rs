use crate::{handler::register::GetInspector, Database, Inspector};

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<DB: Database> Inspector<DB> for NoOpInspector {}

impl<'a, DB: Database> GetInspector<'a, DB> for NoOpInspector {
    fn get_inspector(&mut self) -> &mut dyn Inspector<DB> {
        self
    }
}
