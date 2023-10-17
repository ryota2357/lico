use chumsky::prelude::*;
use lexer::Token;
use std::collections::{HashMap, HashSet};

type Span = SimpleSpan<usize>;
type ParserError<'tokens, 'src> = extra::Err<Rich<'tokens, Token<'src>, Span>>;
type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

mod program;
pub use program::*;

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

mod local;
pub use local::*;

mod misc;
pub use misc::*;

trait TreeWalker<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>);
}

pub(crate) fn analyze_tree(program: &mut Program<'_>) {
    let mut tracker = Tracker::new();

    tracker.push_new_definition_scope();
    program.analyze(&mut tracker);
    tracker.pop_current_definition_scope();
}

struct Tracker<'a> {
    scoped_defs: Vec<HashSet<&'a str>>,
    scoped_caps: Vec<HashSet<&'a str>>,
    all_attr: HashMap<&'a str, Vec<Span>>,
}

impl<'a> Tracker<'a> {
    fn new() -> Self {
        Self {
            scoped_defs: vec![],
            scoped_caps: vec![],
            all_attr: HashMap::new(),
        }
    }

    fn add_definition(&mut self, name: &'a str) {
        if let Some(current_defs) = self.scoped_defs.last_mut() {
            current_defs.insert(name);
        } else {
            unreachable!();
        }
    }

    fn push_new_definition_scope(&mut self) {
        self.scoped_defs.push(HashSet::new());
    }

    fn pop_current_definition_scope(&mut self) -> HashSet<&'a str> {
        match self.scoped_defs.pop() {
            Some(x) => x,
            None => unreachable!(),
        }
    }

    fn add_capture(&mut self, name: &'a str) {
        if let Some(current_defs) = self.scoped_defs.last_mut() {
            if current_defs.contains(name) {
                return;
            }
            for cap in self.scoped_caps.iter_mut() {
                cap.insert(name);
            }
        } else {
            unreachable!();
        }
    }

    fn begin_new_capture_section(&mut self) {
        self.scoped_caps.push(HashSet::new());
    }

    fn end_current_capture_section(&mut self) -> Vec<&'a str> {
        match self.scoped_caps.pop() {
            Some(x) => x.into_iter().collect(),
            None => unreachable!(),
        }
    }

    fn add_attribute(&mut self, name: &'a str, pos: Span) {
        self.all_attr.entry(name).or_default().push(pos);
    }

    fn get_all_attributes(&self) -> Vec<(&'a str, Vec<Span>)> {
        self.all_attr.clone().into_iter().collect()
    }
}
