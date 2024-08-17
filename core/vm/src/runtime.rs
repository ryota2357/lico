mod stack;
use stack::*;

mod local_table;
use local_table::*;

pub(crate) struct Runtime {
    pub(crate) stack: Stack,
    pub(crate) local_table: LocalTable,
    pub(crate) leave_hook: LeaveHook,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        Self {
            stack: Stack::new(),
            local_table: LocalTable::new(),
            leave_hook: LeaveHook::new(),
        }
    }
}
