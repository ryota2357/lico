use super::*;
use crate::syntax::{SyntaxNode, SyntaxToken};
use core::fmt;

#[derive(Debug)]
pub struct Module {
    effects: EffectsKey,
    functions: Vec<FunctionKey>,
    strage: Strage,
}

impl Module {
    pub fn new(effects: EffectsKey, functions: Vec<FunctionKey>, strage: Strage) -> Self {
        Self {
            effects,
            functions,
            strage,
        }
    }

    pub fn effects(&self) -> &EffectsKey {
        &self.effects
    }

    pub fn functions(&self) -> &[FunctionKey] {
        &self.functions
    }

    pub fn strage(&self) -> &Strage {
        &self.strage
    }
}

// p (prefix): [p]ut, first line has not indent
// w (prefix): [w]rite, first line has indent
// l (suffix): [l]ine, last line has newline

macro_rules! wl {
    ($dst:expr, $indent:expr, $format:literal $(,)?) => {
        writeln!($dst, concat!("{:indent$}", $format), "", indent = $indent as usize * 2)
    };
    ($dst:expr, $indent:expr, $format:literal, $($arg:expr),* $(,)?) => {
        writeln!($dst, concat!("{:indent$}", $format), "", $($arg),*, indent = $indent as usize * 2)
    };
}
macro_rules! w {
    ($dst:expr, $indent:expr, $format:literal $(,)?) => {
        write!($dst, concat!("{:indent$}", $format), "", indent = $indent as usize * 2)
    };
    ($dst:expr, $indent:expr, $format:literal, $($arg:tt)*) => {
        write!($dst, concat!("{:indent$}", $format), "", $($arg)*, indent = $indent as usize * 2)
    };
}
use write as p;
use writeln as pl;

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wl!(f, 0, "Module {{")?;

        wl!(f, 1, "effects:")?;
        wl_effects(f, 2, self.strage().get(self.effects()), self.strage())?;

        wl!(f, 1, "functions:")?;
        for key in self.functions() {
            wl!(f, 2, "${}", key.as_u64())?;
            let (param, effects) = self.strage().get(key);

            w!(f, 3, "param: [")?;
            for (i, (syntax, symbol)) in param.enumerate() {
                if i != 0 {
                    p!(f, ", ")?;
                }
                p!(f, "\"{}\"@{:?}", symbol.text(), syntax.text_range())?;
            }
            pl!(f, "]")?;

            wl!(f, 3, "effects:")?;
            wl_effects(f, 4, effects, self.strage())?;
        }

        wl!(f, 0, "}}")
    }
}

fn wl_effects<'s>(
    f: &mut fmt::Formatter<'_>,
    indent: u32,
    effect: impl Iterator<Item = (&'s SyntaxNode, &'s Effect)>,
    strage: &Strage,
) -> fmt::Result {
    for (syntax, effect) in effect {
        #[rustfmt::skip]
        match effect {
            Effect::MakeLocal { name, value } => {
                wl!(f, indent, "MakeLocal@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "name: ")?; pl_symbol(f, strage.get(name))?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::MakeFunc { name, func } => {
                wl!(f, indent, "MakeFunc@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "name: ")?; pl_symbol(f, strage.get(name))?;
                wl!(f, indent + 1, "func: ${}", func.as_u64())?;
                wl!(f, indent, "}}")?;
            }
            Effect::SetLocal { local, value } => {
                wl!(f, indent, "SetLocal@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "local: ")?; pl_symbol(f, strage.get(local))?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::SetIndex { target, index, value } => {
                wl!(f, indent, "SetIndex@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "target: ")?; pl_value(f, indent + 1, strage.get(target), strage)?;
                w!(f, indent + 1, "index: ")?;  pl_value(f, indent + 1, strage.get(index), strage)?;
                w!(f, indent + 1, "value: ")?;  pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::SetField { target, field, value } => {
                wl!(f, indent, "SetField@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "target: ")?; pl_value(f, indent + 1, strage.get(target), strage)?;
                w!(f, indent + 1, "field: ")?; pl_string(f, strage.get(field))?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::SetFieldFunc { table, path, func } => {
                wl!(f, indent, "SetFieldFunc@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "table: ")?; pl_symbol(f, strage.get(table))?;
                w!(f, indent + 1, "path: ")?; pl_strings(f, strage.get(path))?;
                wl!(f, indent + 1, "func: ${}", func.as_u64())?;
                wl!(f, indent, "}}")?;
            }
            Effect::SetMethod { table, path, name, func } => {
                wl!(f, indent, "SetMethod@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "table: ")?; pl_symbol(f, strage.get(table))?;
                w!(f, indent + 1, "path: ")?; pl_strings(f, strage.get(path))?;
                w!(f, indent + 1, "name: ")?; pl_string(f, strage.get(name))?;
                wl!(f, indent + 1, "func: ${}", func.as_u64())?;
                wl!(f, indent, "}}")?;
            }
            Effect::Branch { condition, then, else_ } => {
                wl!(f, indent, "Branch@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "condition: ")?; pl_value(f, indent + 1, strage.get(condition), strage)?;
                wl!(f, indent + 1, "then:")?;
                wl_effects(f, indent + 2, strage.get(then), strage)?;
                wl!(f, indent + 1, "else:")?;
                wl_effects(f, indent + 2, strage.get(else_), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::LoopFor { variable, iterable, effects } => {
                wl!(f, indent, "LoopFor@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "variable: ")?; pl_symbol(f, strage.get(variable))?;
                w!(f, indent + 1, "iterable: ")?; pl_value(f, indent + 1, strage.get(iterable), strage)?;
                wl!(f, indent + 1, "effects:")?;
                wl_effects(f, indent + 2, strage.get(effects), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::LoopWhile { condition, effects } => {
                wl!(f, indent, "LoopWhile@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "condition: ")?; pl_value(f, indent + 1, strage.get(condition), strage)?;
                wl!(f, indent + 1, "effects:")?;
                wl_effects(f, indent + 2, strage.get(effects), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::Scope { body } => {
                wl!(f, indent, "Scope@{:?} {{", syntax.text_range())?;
                wl!(f, indent + 1, "body:")?;
                wl_effects(f, indent + 2, strage.get(body), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::Call { value, args } => {
                wl!(f, indent, "Call@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent + 1, "args: [")?;
                wl_values(f, indent + 2, strage.get(args), strage)?;
                wl!(f, indent + 1, "]")?;
                wl!(f, indent, "}}")?;
            }
            Effect::MethodCall { table, name, args } => {
                wl!(f, indent, "MethodCall@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "table: ")?; pl_value(f, indent + 1, strage.get(table), strage)?;
                w!(f, indent + 1, "name: ")?; pl_string(f, strage.get(name))?;
                wl!(f, indent + 1, "args: [")?;
                wl_values(f, indent + 2, strage.get(args), strage)?;
                wl!(f, indent + 1, "]")?;
                wl!(f, indent, "}}")?;
            }
            Effect::Return { value } => {
                wl!(f, indent, "Return@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
            Effect::BreakLoop => {
                wl!(f, indent, "BreakLoop@{:?}", syntax.text_range())?;
            }
            Effect::ContinueLoop => {
                wl!(f, indent, "ContinueLoop@{:?}", syntax.text_range())?;
            }
            Effect::NoEffectValue { value } => {
                wl!(f, indent, "NoEffectValue@{:?} {{", syntax.text_range())?;
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
                wl!(f, indent, "}}")?;
            }
        };
    }
    Ok(())
}

fn wl_values<'s>(
    f: &mut fmt::Formatter<'_>,
    indent: u32,
    iter: impl Iterator<Item = (&'s SyntaxNode, &'s Value)>,
    strage: &Strage,
) -> fmt::Result {
    for arg in iter {
        w!(f, indent, "")?;
        pl_value(f, indent, Some(arg), strage)?;
    }
    Ok(())
}

fn pl_value(
    f: &mut fmt::Formatter<'_>,
    indent: u32,
    value: Option<(&SyntaxNode, &Value)>,
    strage: &Strage,
) -> fmt::Result {
    let Some((syntax, value)) = value else {
        return pl!(f, "None");
    };
    #[rustfmt::skip]
    match value {
        Value::Branch { condition, then, then_tail, else_, else_tail } => {
            pl!(f,"Branch@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "condition: ")?; pl_value(f, indent + 1, strage.get(condition), strage)?;
            w!(f, indent + 1, "then:")?;
            wl_effects(f, indent + 2, strage.get(then), strage)?;
            w!(f, indent + 1, "then_tail: ")?; pl_value(f, indent + 1, strage.get(then_tail), strage)?;
            w!(f, indent + 1, "else:")?;
            wl_effects(f, indent + 2, strage.get(else_), strage)?;
            w!(f, indent + 1, "else_tail: ")?; pl_value(f, indent + 1, strage.get(else_tail), strage)?;
            wl!(f, indent,"}}")?;
        }
        Value::Prefix { op, value } => {
            pl!(f, "Prefix@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "op: ")?; match op {
                PrefixOp::Plus(syntax)   => pl!(f, "'+'@{:?}", syntax.text_range())?,
                PrefixOp::Minus(syntax)  => pl!(f, "'-'@{:?}", syntax.text_range())?,
                PrefixOp::Not(syntax)    => pl!(f, "'not'@{:?}", syntax.text_range())?,
                PrefixOp::BitNot(syntax) => pl!(f, "'~'@{:?}", syntax.text_range())?,
                PrefixOp::Missing => pl!(f, "None")?,
            }
            w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            wl!(f, indent,"}}")?;
        }
        Value::Binary { op, lhs, rhs } => {
            pl!(f, "Binary@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "op: ")?; match op {
                BinaryOp::Add(syntax)      => pl!(f, "'+'@{:?}",   syntax.text_range())?,
                BinaryOp::Sub(syntax)      => pl!(f, "'-'@{:?}",   syntax.text_range())?,
                BinaryOp::Mul(syntax)      => pl!(f, "'*'@{:?}",   syntax.text_range())?,
                BinaryOp::Div(syntax)      => pl!(f, "'/'@{:?}",   syntax.text_range())?,
                BinaryOp::Mod(syntax)      => pl!(f, "'%'@{:?}",   syntax.text_range())?,
                BinaryOp::Shl(syntax)      => pl!(f, "'>>'@{:?}",  syntax.text_range())?,
                BinaryOp::Shr(syntax)      => pl!(f, "'<<'@{:?}",  syntax.text_range())?,
                BinaryOp::Concat(syntax)   => pl!(f, "'..'@{:?}",  syntax.text_range())?,
                BinaryOp::Eq(syntax)       => pl!(f, "'=='@{:?}",  syntax.text_range())?,
                BinaryOp::Ne(syntax)       => pl!(f, "'!='@{:?}",  syntax.text_range())?,
                BinaryOp::Lt(syntax)       => pl!(f, "'<'@{:?}",   syntax.text_range())?,
                BinaryOp::Le(syntax)       => pl!(f, "'<='@{:?}",  syntax.text_range())?,
                BinaryOp::Gt(syntax)       => pl!(f, "'>'@{:?}",   syntax.text_range())?,
                BinaryOp::Ge(syntax)       => pl!(f, "'=>'@{:?}",  syntax.text_range())?,
                BinaryOp::And(syntax)      => pl!(f, "'and'@{:?}", syntax.text_range())?,
                BinaryOp::Or(syntax)       => pl!(f, "'or'@{:?}",  syntax.text_range())?,
                BinaryOp::BitAnd(syntax)   => pl!(f, "'&'@{:?}",   syntax.text_range())?,
                BinaryOp::BitOr(syntax)    => pl!(f, "'|'@{:?}",   syntax.text_range())?,
                BinaryOp::BitXor(syntax)   => pl!(f, "'^'@{:?}",   syntax.text_range())?,
                BinaryOp::Assign(syntax)   => pl!(f, "'='@{:?}",   syntax.text_range())?,
                BinaryOp::Missing => pl!(f, "None")?,
            }
            w!(f, indent + 1, "lhs: ")?; pl_value(f, indent + 1, strage.get(lhs), strage)?;
            w!(f, indent + 1, "rhs: ")?; pl_value(f, indent + 1, strage.get(rhs), strage)?;
            wl!(f, indent,"}}")?;
        }
        Value::Call { value, args } => {
            pl!(f, "Call@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            wl!(f, indent + 1, "args: [")?;
            wl_values(f, indent + 2, strage.get(args), strage)?;
            wl!(f, indent + 1, "]")?;
            wl!(f, indent,"}}")?;
        }
        Value::Index { value, index } => {
            pl!(f, "Index@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            w!(f, indent + 1, "index: ")?; pl_value(f, indent + 1, strage.get(index), strage)?;
            wl!(f, indent,"}}")?;
        }
        Value::Field { value, name } => {
            pl!(f, "Field@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            w!(f, indent + 1, "name: ")?; pl_string(f, strage.get(name))?;
            wl!(f, indent,"}}")?;
        }
        Value::MethodCall { value, name, args } => {
            pl!(f, "MethodCall@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            w!(f, indent + 1, "name: ")?; pl_string(f, strage.get(name))?;
            wl!(f, indent + 1, "args: [")?;
            wl_values(f, indent + 2, strage.get(args), strage)?;
            wl!(f, indent + 1, "]")?;
            wl!(f, indent,"}}")?;
        }
        Value::Block { effects, tail } => {
            pl!(f, "Block@{:?} {{", syntax.text_range())?;
            wl!(f, indent + 1, "effects:")?;
            wl_effects(f, indent + 1, strage.get(effects), strage)?;
            w!(f, indent + 1, "tail: ")?; pl_value(f, indent + 1, strage.get(tail), strage)?;
            wl!(f, indent,"}}")?;
        }
        Value::Local { name } => {
            pl!(f, "Local@{:?} {{", syntax.text_range())?;
            w!(f, indent + 1, "name: ")?; pl_symbol(f, strage.get(name))?;
            wl!(f, indent,"}}")?;
        }
        Value::Int(x) => {
            pl!(f, "Int@{:?} {}", syntax.text_range(), x)?;
        }
        Value::Float(x) => {
            pl!(f, "Float@{:?} {}", syntax.text_range(), x)?;
        }
        Value::String(x) => {
            pl!(f, "String@{:?} \"{}\"", syntax.text_range(), x)?;
        }
        Value::Bool(x) => {
            pl!(f, "Bool@{:?} {}", syntax.text_range(), x)?;
        }
        Value::Nil => {
            pl!(f, "Nil@{:?}", syntax.text_range())?;
        }
        Value::Function(func) => {
            pl!(f, "Function@{:?} ${}", syntax.text_range(), func.as_u64())?;
        }
        Value::Array { elements } => {
            p!(f, "Array@{:?} [", syntax.text_range())?;
            wl_values(f, indent + 1, strage.get(elements), strage)?;
            wl!(f, indent,"]")?;
        }
        Value::Table { fields } => {
            pl!(f, "Table@{:?} {{", syntax.text_range())?;
            for (key, value) in fields {
                w!(f, indent + 1, "key: ")?; match key {
                    TableKeyName::Value(value) => pl_value(f, indent + 1, strage.get(value), strage)?,
                    TableKeyName::String(string) => pl_string(f, strage.get(string))?,
                }
                w!(f, indent + 1, "value: ")?; pl_value(f, indent + 1, strage.get(value), strage)?;
            }
            wl!(f, indent,"}}")?;
        }
    };
    Ok(())
}

fn pl_symbol(f: &mut fmt::Formatter<'_>, symbol: Option<(&SyntaxToken, &Symbol)>) -> fmt::Result {
    if let Some((syntax, symbol)) = symbol {
        let range = syntax.text_range();
        pl!(f, "\"{}\"@{:?}", symbol.text(), range)
    } else {
        pl!(f, "None")
    }
}

fn pl_string(
    f: &mut fmt::Formatter<'_>,
    string: Option<(&SyntaxToken, &CompactString)>,
) -> fmt::Result {
    if let Some((syntax, string)) = string {
        let range = syntax.text_range();
        pl!(f, "\"{}\"@{:?}", &string, range)
    } else {
        pl!(f, "None")
    }
}

fn pl_strings<'s>(
    f: &mut fmt::Formatter<'_>,
    iter: impl Iterator<Item = (&'s SyntaxToken, &'s CompactString)>,
) -> fmt::Result {
    p!(f, "[")?;
    for (i, (syntax, string)) in iter.enumerate() {
        if i != 0 {
            p!(f, ", ")?;
        }
        p!(f, "\"{}\"@{:?}", &string, syntax.text_range())?;
    }
    pl!(f, "]")
}
