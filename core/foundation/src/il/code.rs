use super::LocalId;
use crate::object::{RustFunction, UString};

#[derive(Clone, PartialEq, Debug)]
pub enum Code {
    /// Push a constant 64bit signed integer value to the stack.
    LoadInt(i64),

    /// Push a constant 64bit floating point value to the stack.
    LoadFloat(f64),

    /// Push a constant bool value to the stack.
    LoadBool(bool),

    /// Push a constant string value to the stack.
    LoadString(UString),

    /// Push a constant nil value to the stack.
    LoadNil,

    /// Push the stored local object specified by `LocalId` onto the stack.
    LoadLocal(LocalId),

    /// Push a rust function to the stack.
    LoadRustFunction(RustFunction),

    /// Pop the top value from the stack.
    UnloadTop,

    /// Write the top value of the stack to the location specified by `LoalId` in the local object table.
    SetLocal(LocalId),

    /// TODO
    MakeLocal,

    /// Pop the specified number of values from the stack, make them into an array, and push the array to the stack.
    /// TODO: 取り出されるスタックの値の制約記述
    MakeArray(u32),

    /// TODO
    MakeNamed,

    /// Pop the specified number of values from the stack, make them into a table, and push the table to the stack.
    /// TODO: 取り出されるスタックの値の制約記述
    MakeTable(u32),

    /// TODO
    DropLocal(usize),

    /// TODO
    Jump(isize),

    /// TODO
    JumpIfTrue(isize),

    /// TODO
    JumpIfFalse(isize),

    /// TODO
    CallMethod(),

    /// TODO
    Call(u8),

    /// TODO
    SetItem,

    /// TODO
    GetItem,

    /// TODO: +
    Add,

    /// TODO: -
    Sub,

    /// TODO: *
    Mul,

    /// TODO: /
    Div,

    /// TODO: %
    Mod,

    /// TODO: *
    Pow,

    /// TODO: - (unary)
    Unm,

    /// TODO: ==
    Eq,

    /// TODO: !=
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

    BeginFuncCreation,
    AddCapture(LocalId),
    // AddArgument(ArgumentKind),
    EndFuncCreation,

    /// Do nothing.
    /// Only increment the program counter.
    Nop,

    /// Returns the top value of the stack and exits current executable (program).
    Return,
}
