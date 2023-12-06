use super::*;
use lexer::Token;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, rc::Rc};

type ParserError<'src> = extra::Err<error::Error<'src>>;
type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]>;

pub(crate) fn program<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Program<'src>, ParserError<'src>> + Clone {
    block().then_ignore(end()).map(|block| Program {
        attributes: vec![],
        body: block.into(),
    })
}

mod block;
pub use block::*;

mod statement;
pub use statement::*;

mod attribute_statement;
pub use attribute_statement::*;

mod control_statement;
pub use control_statement::*;

mod variable_statement;
pub use variable_statement::*;

mod call_statement;
pub use call_statement::*;

mod expression;
pub use expression::*;

mod table_object;
pub use table_object::*;

mod array_object;
pub use array_object::*;

mod function_object;
pub use function_object::*;

mod misc;
pub use misc::*;

pub trait Walkable<'walker, 'src: 'walker> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>);
}

#[derive(Debug, Default)]
pub struct Walker<'walker, 'src: 'walker> {
    master_defs: Vec<&'walker FxHashSet<&'src str>>,
    defs: FxHashSet<&'src str>,
    caps: Rc<RefCell<FxHashMap<&'src str, Span>>>,
    attrs: Rc<RefCell<FxHashMap<&'src str, Vec<Span>>>>,
}

#[derive(Debug)]
pub struct WalkerArtifact<'src> {
    caps: Option<FxHashMap<&'src str, Span>>,
    attrs: Option<FxHashMap<&'src str, Vec<Span>>>,
}

impl<'walker, 'src: 'walker> Walker<'walker, 'src> {
    pub fn new() -> Self {
        Self {
            master_defs: Vec::new(),
            defs: FxHashSet::default(),
            caps: Rc::new(RefCell::new(FxHashMap::default())),
            attrs: Rc::new(RefCell::new(FxHashMap::default())),
        }
    }

    pub fn fork(&'walker self) -> Self {
        let mut master_defs = self.master_defs.clone();
        master_defs.push(&self.defs);
        Self {
            master_defs,
            defs: FxHashSet::default(),
            caps: Rc::clone(&self.caps),
            attrs: Rc::clone(&self.attrs),
        }
    }

    pub fn go(&mut self, walkable: &mut impl Walkable<'walker, 'src>) {
        walkable.accept(self);
    }

    pub fn record_variable_definition(&mut self, name: &'src str) {
        self.defs.insert(name);
    }

    pub fn record_attribute(&mut self, name: &'src str, span: &Span) {
        self.attrs
            .borrow_mut()
            .entry(name)
            .or_default()
            .push(span.clone());
    }

    pub fn record_variable_usage(&mut self, name: &'src str, span: &Span) {
        if self.defs.contains(name) {
            return;
        }
        for defs in self.master_defs.iter().rev() {
            if defs.contains(name) {
                return;
            }
        }
        self.caps
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| span.clone());
    }

    pub fn finish(self) -> WalkerArtifact<'src> {
        // NOTE: if Rc::strong_count(&self.*) != 1 then None else Some.
        let caps = Rc::into_inner(self.caps).map(|refcell| refcell.into_inner());
        let attrs = Rc::into_inner(self.attrs).map(|refcell| refcell.into_inner());
        WalkerArtifact { caps, attrs }
    }

    pub fn merge(&mut self, artifact: WalkerArtifact<'src>) {
        let WalkerArtifact { caps, attrs } = artifact;
        if let Some(caps) = caps {
            for (name, span) in caps {
                if self.defs.contains(name) {
                    continue;
                }
                for defs in self.master_defs.iter().rev() {
                    if defs.contains(name) {
                        continue;
                    }
                }
                self.caps
                    .borrow_mut()
                    .entry(name)
                    .or_insert_with(|| span.clone());
            }
        }
        if let Some(attrs) = attrs {
            self.attrs.borrow_mut().extend(attrs);
        }
    }
}

impl<'src> WalkerArtifact<'src> {
    pub fn captures(&self) -> Vec<(&'src str, Span)> {
        if let Some(caps) = &self.caps {
            let mut res = caps
                .iter()
                .map(|(name, span)| (*name, span.clone()))
                .collect::<Vec<_>>();
            res.sort_unstable_by_key(|(name, _)| *name);
            res
        } else {
            Vec::new()
        }
    }

    pub fn take_attributes(&mut self) -> Vec<(&'src str, Vec<Span>)> {
        // TODO: expectのメッセージ変える
        let attrs = self
            .attrs
            .take()
            .expect("`attributes` should only be collected once.");
        let mut res = attrs
            .into_iter()
            .map(|(name, spans)| {
                let mut spans = spans;
                spans.sort_unstable_by_key(|span| span.start);
                (name, spans)
            })
            .collect::<Vec<_>>();
        res.sort_unstable_by_key(|(_, spans)| spans[0].start);
        res
    }
}
