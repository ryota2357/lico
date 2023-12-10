mod object;
pub use object::*;

mod stack;
pub use stack::*;

mod variable_table;
pub use variable_table::*;

mod global;
pub use global::Global;

mod stdio;
pub use stdio::Stdio;

#[derive(Debug, Default)]
pub struct Runtime {
    pub stack: Stack,
    pub variable_table: VariableTable,
    pub global: Global,
    pub stdio: Stdio,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            variable_table: VariableTable::new(),
            global: Global::new(),
            stdio: Stdio::new(),
        }
    }

    pub fn dump(&self) {
        println!("[Runtime]");
        self.stack.dump(2);
        self.variable_table.dump(2);
    }
}
