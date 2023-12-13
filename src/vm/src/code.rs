use std::{borrow::Cow, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LocalId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub enum Code {
    LoadInt(i64),
    LoadFloat(f64),
    LoadBool(bool),
    LoadString(Rc<String>),
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
    /// Write all arguments to stdout.
    ///
    /// args: >= 0
    /// return: none
    Write,

    /// Flush stdout.
    ///
    /// args: 0
    /// return: none
    Flush,

    /// Write all arguments to stderr.
    ///
    /// args: >= 0
    /// return: none
    WriteError,

    /// Flush stderr.
    ///
    /// args: 0
    /// return: none
    FlushError,

    /// Read a line from stdin.
    ///
    /// args: 0
    /// return: 1 (String)
    ReadLine,

    /// Read the entire contents of a file.
    ///
    /// args: 1
    /// return: 1 (String)
    ReadFile,

    /// Write a string to a file.
    /// If the file does not exist, it will be created.
    ///
    /// args: 2 (filename: String, contents: String)
    /// return: none
    WriteFile,
}
