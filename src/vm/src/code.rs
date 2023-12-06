use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LocalId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub enum Code {
    LoadInt(i64),
    LoadFloat(f64),
    LoadBool(bool),
    LoadString(String),
    LoadNil,
    LoadLocal(LocalId),
    LoadRustFunction(fn(&[crate::Object]) -> Result<crate::Object, String>),
    UnloadTop,

    SetLocal(LocalId),
    MakeLocal,
    MakeArray(u32),
    MakeNamed,
    MakeTable(u32),
    DropLocal(usize),

    Jump(isize),
    JumpIfTrue(isize),
    JumpIfFalse(isize),

    CallMethod(Cow<'static, str>, u8),
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

    Builtin(BuiltinInstr, u8),

    BeginFuncCreation,
    AddCapture(LocalId),
    AddArgument(()),
    EndFuncCreation,

    Nop,
    Return,

    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinInstr {
    Write,
    Flush,
}
