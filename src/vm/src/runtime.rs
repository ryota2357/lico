mod object;
pub use object::*;

mod stack;
pub use stack::*;

mod variable_table;
pub use variable_table::*;

mod global;
pub use global::*;

#[derive(Default, Debug, PartialEq)]
pub struct Runtime<Writer: std::io::Write> {
    pub stack: Stack,
    pub variable_table: VariableTable,
    pub global: Global,
    pub writer: Writer,
}

impl<W: std::io::Write> Runtime<W> {
    pub fn new(writer: W) -> Self {
        Self {
            stack: Stack::new(),
            variable_table: VariableTable::new(),
            global: Global::new(),
            writer,
        }
    }

    pub fn dump(&self) {
        println!("[Runtime]");
        self.stack.dump(2);
        self.variable_table.dump(2);
    }
}
