use crate::syntax::{SyntaxNode, SyntaxToken};
use compact_str::CompactString;

mod storage;
pub use storage::*;

mod item;
pub use item::*;

mod module;
pub use module::Module;

pub struct ModuleBuilder {
    sb: StrageBuilder,
    fns: Vec<FunctionKey>,
}

// ModuleBuilder は StrageBuilder をほぼラップしているだけなので、ModuleBuilder が全部やれば？となるかもしれないが、そうではない。
// StrageBuilder の方に書いたが、StrageBuilder::add() で全ての key が登録できるようにしたい。
// StrageBuilder::add() が実現できた場合:
//   - StrageBuilder は Into* 系の trait を扱えない (StrageKey の抽象化の意味的に) ので、ModuleBuilder がそれを担当する必要がある。(現在もそのようになっている)
//   - StrageBuilder::add() はシグネチャが抽象化されていて、わかりにくいので、より具体的な型を ModuleBuilder で提供する。
// という理由があるため、ModuleBuilder と StrageBuilder は分けている。
impl ModuleBuilder {
    pub const fn new() -> Self {
        ModuleBuilder {
            sb: StrageBuilder::new(),
            fns: Vec::new(),
        }
    }

    pub fn finish_with(self, root_effects: EffectsKey) -> Module {
        let strage = self.sb.finish();
        Module::new(root_effects, self.fns, strage)
    }

    pub fn add_value(&mut self, value: impl Into<Option<(SyntaxNode, Value)>>) -> ValueKey {
        let value = value.into();
        self.sb.add_value(value)
    }

    pub fn add_value_many<I>(&mut self, values: I) -> ValueSliceKey
    where
        I: IntoIterator<Item = (SyntaxNode, Value)>,
    {
        let values = values.into_iter();
        self.sb.add_value_many(values)
    }

    pub fn add_effects<I>(&mut self, effects: I) -> EffectsKey
    where
        I: IntoIterator<Item = (SyntaxNode, Effect)>,
    {
        let effects = effects.into_iter();
        self.sb.add_effects(effects)
    }

    pub fn add_string(
        &mut self,
        string: impl Into<Option<(SyntaxToken, CompactString)>>,
    ) -> StringKey {
        let string = string.into();
        self.sb.add_string(string)
    }

    pub fn add_string_many<I>(&mut self, strings: I) -> StringSliceKey
    where
        I: IntoIterator<Item = (SyntaxToken, CompactString)>,
    {
        let strings = strings.into_iter();
        self.sb.add_string_many(strings)
    }

    pub fn add_symbol(&mut self, symbol: impl Into<Option<(SyntaxToken, Symbol)>>) -> SymbolKey {
        let symbol = symbol.into();
        self.sb.add_symbol(symbol)
    }

    pub fn add_function<S, E>(&mut self, symbols: S, effects: E) -> FunctionKey
    where
        S: IntoIterator<Item = (SyntaxToken, Symbol)>,
        E: IntoIterator<Item = (SyntaxNode, Effect)>,
    {
        let symbols = symbols.into_iter();
        let effects = effects.into_iter();
        let key = self.sb.add_function(symbols, effects);
        self.fns.push(key);
        key
    }
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}
