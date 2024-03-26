use super::LocalId;
use crate::object::{RustFunction, UString};

#[derive(Clone, PartialEq, Debug)]
pub enum Code {
    /// Load (push) a constant int value to the stack.
    LoadInt(i64),

    /// Load (push) a constant float value to the stack.
    LoadFloat(f64),

    /// Load (push) a constant bool value to the stack.
    LoadBool(bool),

    /// Load (push) a constant string value to the stack.
    LoadString(UString),

    /// Load (push) a constant nil value to the stack.
    LoadNil,

    /// Load (push) the stored local object specified by `LocalId` onto the stack.
    LoadLocal(LocalId),

    /// Load (push) a rust function to the stack.
    LoadRustFunction(RustFunction),

    /// Unload (pop) the top value from the stack.
    UnloadTop,

    /// Set (write) the top value of the stack to the location specified by `LoalId` in the local object table.
    SetLocal(LocalId),

    MakeLocal,
    MakeArray(u32),
    MakeNamed,
    MakeTable(u32),
    DropLocal(usize),

    Jump(isize),
    JumpIfTrue(isize),
    JumpIfFalse(isize),

    CallMethod(),
    Call(u8),
    SetItem,
    GetItem,
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Mod,       // %
    Pow,       // *
    Unm,       // - (unary)
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=
    Concat,    // ..
    BitAnd,    // &
    BitOr,     // |
    BitXor,    // ^
    BitNot,    // ~
    ShiftL,    // <<
    ShiftR,    // >>

    // Builtin(BuiltinInstr, u8),
    BeginFuncCreation,
    AddCapture(LocalId),
    // AddArgument(ArgumentKind),
    EndFuncCreation,

    Nop,
    Return,

    Exit,
}
