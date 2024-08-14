use super::LocalId;
use crate::object::*;

use core::fmt;

fn _size_check() {
    const {
        assert!(size_of::<ICode>() == 16);
        assert!(size_of::<Option<ICode>>() == 16);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ICode {
    /// Pushes a constant integer value as an object to the stack.
    LoadIntObject(i64),

    /// Pushes a constant float value as an object to the stack.
    LoadFloatObject(f64),

    /// Pushes a constant string value as an object to the stack.
    LoadStringObject(UString),

    /// Pushes a constant boolean value as an object to the stack.
    LoadBoolObject(bool),

    /// Pushes a constant nil value as an object to the stack.
    LoadNilObject,

    /// Pushes a constant array value as an object to the stack.
    LoadArrayObject(Array),

    /// Pushes a constant table value as an object to the stack.
    LoadTableObject(Table),

    /// Pushes the stored local object specified by `.0` onto the local variable table.
    ///
    /// # Panic
    ///
    /// Not found the specified local object.
    LoadLocal(LocalId),

    /// Pushes a rust function to the stack.
    LoadRustFunction(RustFunction),

    /// Pushes the top value from the stack.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    Unload,

    /// Writes the top value of the stack to the location specified by `.0` in the local variable
    /// table.
    /// The top value of the stack is popped.
    ///
    /// # Panic
    ///
    /// Not found the specified local object.
    StoreLocal(LocalId),

    /// Writes the top value of the stack to the last location in the local variable table.
    /// The top value of the stack is popped.
    StoreNewLocal,

    /// Pops the specified number (`.0`) of values from the stack, makes them into an array, and
    /// pushes the array to the stack.
    ///
    /// # Panic
    ///
    /// Stack size is less than the specified number.
    MakeArray(usize),

    /// Pops the specified number of values from the stack, makes them into a table, and pushes the
    /// table to the stack.
    ///
    /// Specified number must be even, which is the number of keys and values in the table.
    /// The even-numbered popped must be type of string, which is the table key, and the
    /// odd-numbered one allows any type of object, which is the table value.
    ///
    /// # Exeption
    ///
    /// The even-numbered popped is not type of string.
    ///
    /// # Panic
    ///
    /// - Stack size is less than the specified number.
    /// - Specified number is odd.
    MakeTable(usize),

    /// Removes the last n (specified number, .0) added values from local variable table.
    ///
    /// # Panic
    ///
    /// Specified number is greater than the local variable table size.
    DropLocal(usize),

    /// Adds the specified number (`.0`) to program counter.
    Jump(isize),

    /// Pops the top value from the stack, and if it is truthy, adds the specified number (`.0`) to
    /// program counter.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    JumpIfTrue(isize),

    /// Pops the top value from the stack, and if it is falsy, adds the specified number (`.0`) to
    /// program counter.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    JumpIfFalse(isize),

    /// Pops the specified number (`.0`) of values as arguments from the stack.
    /// These argument are stored in reverse order, with the first argument being at the top of the
    /// stack and the last argument being at the bottom.
    ///
    /// After collecting the arguments, pops next value from the stack, which is expected to be a
    /// callable object.
    ///
    /// Calls the callable object with the collected arguments.
    ///
    /// # Exeption
    ///
    /// - The callee is not type of Function or RustFunction or Table.
    /// - No `__call` method defined for the popped table type value.
    /// - The number of calee arguments is not equal to the specified argument count.
    ///
    /// # Panic
    ///
    /// Stack size is less than (`.0` + 1).
    Call(u8),

    /// Pops the specified number (`.0`) of values as arguments from the stack.
    /// These argument are stored in reverse order, with the first argument being at the top of the
    /// stack and the last argument being at the bottom.
    ///
    /// After collecting the arguments, pops next value from the stack, which is expected to be a
    /// table object which has the specified method name.
    ///
    /// # Exeption
    ///
    /// - The receiver is not type of Table.
    /// - The specified method name is not defined in the receiver table object.
    /// - The number of method arguments is not equal to the specified argument count.
    ///
    /// # Panic
    ///
    /// Stack size is less than (`.0` + 1).
    CallMethod(u8, UString),

    /// TODO
    ///
    /// # Exeption
    ///
    /// - The container is not type of Table or Array.
    /// - The key is not type of Int if the container is Array.
    /// - The key is not type of String if the container is Table.
    SetItem,

    /// TODO
    ///
    /// # Exeption
    ///
    /// - The container is not type of Table or Array.
    /// - The key is not type of Int if the container is Array.
    /// - The key is not type of String if the container is Table.
    GetItem,

    /// TODO
    ///
    /// # Exeption
    ///
    /// - The container is not type of Table.
    SetMethod(UString),

    /// Pops the top two values from the stack, and pushes the `+` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped values are not type of Int or Float or Table.
    /// - No `__add` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Add,

    /// Pops the top two values from the stack, and pushes the `-` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped values are not type of Int or Float or Table.
    /// - No `__sub` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Sub,

    /// Pops the top two values from the stack, and pushes the `*` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped values are not type of Int or Float or Table.
    /// - No `__mul` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Mul,

    /// Pops the top two values from the stack, and pushes the `/` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped values are not type of Int or Float or Table.
    /// - No `__div` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Div,

    /// Pops the top two values from the stack, and pushes the `%` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped values are not type of Int or Float or Table.
    /// - No `__mod` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Mod,

    /// Pops the value from the stack, and pushes the unary `-` operation result to the stack.
    ///
    /// # Exeption
    ///
    /// - Popped value is not type of Int or Float or Table.
    /// - No `__unm` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    Unm,

    /// Pops the value from the stack, and pushes the unary `+` operation result to the stack.
    ///
    ///
    /// # Exeption
    ///
    /// - Popped value is not type of Int or Float or Table.
    /// - No `__unp` method defined for the popped table type value.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    Unp,

    /// TODO
    Not,

    /// Pops the top two values from the stack, and pushes the result of the equivalence comparison
    /// to the stack as a boolean value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    Eq,

    /// Pops the top two values from the stack, and pushes the result of the non-equivalence
    /// comparison to the stack as a boolean value.
    ///
    /// # Panic
    ///
    /// Stack size is less than 2.
    NotEq,

    /// TODO: <
    Less,

    /// TODO: <=
    LessEq,

    /// TODO: >
    Greater,

    /// TODO: >=
    GreaterEq,

    /// TODO: ..
    Concat,

    /// TODO: &
    BitAnd,

    /// TODO: |
    BitOr,

    /// TODO: ^
    BitXor,

    /// TODO: ~
    BitNot,

    /// TODO: <<
    ShiftL,

    /// TODO: >>
    ShiftR,

    /// TODO: `__get_iter` method
    GetIter,

    /// TODO: `__move_next` method
    IterMoveNext,

    /// TODO: `__current` method
    IterCurrent,

    /// Enters the "Create Function Object" section.
    ///
    /// In this section, only `FuncSetProperty`, `FuncAddCapture`, and `EndFuncSection` are
    /// allowed.
    /// This section is exited by `EndFuncSection`.
    BeginFuncSection,

    /// Sets the property of the function.
    ///
    /// - `.0` is the parameter count of the function.
    /// - `.1` is the start program counter of the function.
    ///
    /// # Panic
    ///
    /// - Used outside of the "Create Function Object" section.
    /// - Used twice or more.
    FuncSetProperty(u8, usize),

    /// Adds the specified local object to the function's capture list.
    ///
    /// # Panic
    ///
    /// - Used outside of the "Create Function Object" section.
    /// - The specified local object is not found.
    FuncAddCapture(LocalId),

    /// Exits the "Create Function Object" section.
    ///
    /// # Panic
    ///
    /// - Used outside of the "Create Function Object" section.
    /// - `FuncSetProperty` is not used.
    EndFuncSection,

    /// Do nothing. Only increment the program counter.
    Nop,

    /// Returns the top value of the stack and exits current function.
    /// The top value of the stack is popped.
    ///
    /// # Panic
    ///
    /// Stack is empty.
    Leave,
}

impl fmt::Display for ICode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip]
        let res = match self {
            ICode::LoadIntObject(a0)       => write!(f, "LoadIntObject    {}", a0),
            ICode::LoadFloatObject(a0)     => write!(f, "LoadFloatObject  {}", a0),
            ICode::LoadStringObject(a0)    => write!(f, "LoadStringObject {}", a0),
            ICode::LoadBoolObject(a0)      => write!(f, "LoadBoolObject   {}", a0),
            ICode::LoadNilObject           => write!(f, "LoadNilObject    "),
            ICode::LoadArrayObject(a0)     => write!(f, "LoadArrayObject  len={}", a0.len()),
            ICode::LoadTableObject(a0)     => write!(f, "LoadTableObject  len={}", a0.len()),
            ICode::LoadLocal(a0)           => write!(f, "LoadLocal        {}", a0.as_usize()),
            ICode::LoadRustFunction(a0)    => write!(f, "LoadRustFunction {:?}", a0),
            ICode::Unload                  => write!(f, "Unload           "),
            ICode::StoreLocal(a0)          => write!(f, "StoreLocal       {}", a0.as_usize()),
            ICode::StoreNewLocal           => write!(f, "StoreNewLocal    "),
            ICode::MakeArray(a0)           => write!(f, "MakeArray        {}", a0),
            ICode::MakeTable(a0)           => write!(f, "MakeTable        {}", a0),
            ICode::DropLocal(a0)           => write!(f, "DropLocal        {}", a0),
            ICode::Jump(a0)                => write!(f, "Jump             {}", a0),
            ICode::JumpIfTrue(a0)          => write!(f, "JumpIfTrue       {}", a0),
            ICode::JumpIfFalse(a0)         => write!(f, "JumpIfFalse      {}", a0),
            ICode::CallMethod(a0, a1)      => write!(f, "CallMethod       {} {}", a0, a1),
            ICode::Call(a0)                => write!(f, "Call             {}", a0),
            ICode::SetItem                 => write!(f, "SetItem          "),
            ICode::GetItem                 => write!(f, "GetItem          "),
            ICode::SetMethod(a0)           => write!(f, "SetMethod        {}", a0),
            ICode::Add                     => write!(f, "Add              "),
            ICode::Sub                     => write!(f, "Sub              "),
            ICode::Mul                     => write!(f, "Mul              "),
            ICode::Div                     => write!(f, "Div              "),
            ICode::Mod                     => write!(f, "Mod              "),
            ICode::Unm                     => write!(f, "Unm              "),
            ICode::Unp                     => write!(f, "Unp              "),
            ICode::Not                     => write!(f, "Not              "),
            ICode::Eq                      => write!(f, "Eq               "),
            ICode::NotEq                   => write!(f, "NotEq            "),
            ICode::Less                    => write!(f, "Less             "),
            ICode::LessEq                  => write!(f, "LessEq           "),
            ICode::Greater                 => write!(f, "Greater          "),
            ICode::GreaterEq               => write!(f, "GreaterEq        "),
            ICode::Concat                  => write!(f, "Concat           "),
            ICode::BitAnd                  => write!(f, "BitAnd           "),
            ICode::BitOr                   => write!(f, "BitOr            "),
            ICode::BitXor                  => write!(f, "BitXor           "),
            ICode::BitNot                  => write!(f, "BitNot           "),
            ICode::ShiftL                  => write!(f, "ShiftL           "),
            ICode::ShiftR                  => write!(f, "ShiftR           "),
            ICode::GetIter                 => write!(f, "GetIter          "),
            ICode::IterMoveNext            => write!(f, "IterMoveNext     "),
            ICode::IterCurrent             => write!(f, "IterCurrent      "),
            ICode::BeginFuncSection        => write!(f, "BeginFuncSection "),
            ICode::FuncSetProperty(a0, a1) => write!(f, "  SetProperty    param={} start={}", a0, a1),
            ICode::FuncAddCapture(a0)      => write!(f, "  AddCapture     {}", a0.as_usize()),
            ICode::EndFuncSection          => write!(f, "EndFuncSection   "),
            ICode::Nop                     => write!(f, "Nop              "),
            ICode::Leave                   => write!(f, "Leave            "),
        };
        res
    }
}
