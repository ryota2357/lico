mod object;
pub use object::*;

mod stack;
pub use stack::*;

mod variable_table;
pub use variable_table::*;

mod global;
pub use global::*;

#[derive(Default, Debug, PartialEq)]
pub struct Runtime<'a> {
    pub stack: Stack<'a>,
    pub variable_table: VariableTable<'a>,
    pub global: Global,
}

impl Runtime<'_> {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            variable_table: VariableTable::new(),
            global: Global::new(),
        }
    }

    pub fn dump(&self) {
        println!("[Runtime]");
        self.stack.dump(2);
        self.variable_table.dump(2);
    }
}
