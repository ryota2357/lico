use super::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Walker<'walker, 'src: 'walker> {
    master_defs: Vec<&'walker FxHashSet<&'src str>>,
    defs: FxHashSet<&'src str>,
    caps: Rc<RefCell<FxHashMap<&'src str, TextSpan>>>,
    attrs: Rc<RefCell<FxHashMap<&'src str, Vec<TextSpan>>>>,
}

#[derive(Debug)]
pub struct WalkerArtifact<'src> {
    caps: Option<FxHashMap<&'src str, TextSpan>>,
    attrs: Option<FxHashMap<&'src str, Vec<TextSpan>>>,
}

pub trait Walkable<'walker, 'src: 'walker> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>);
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

    pub fn record_attribute(&mut self, name: &'src str, span: TextSpan) {
        self.attrs.borrow_mut().entry(name).or_default().push(span);
    }

    pub fn record_variable_usage(&mut self, name: &'src str, span: TextSpan) {
        if self.defs.contains(name) {
            return;
        }
        for defs in self.master_defs.iter().rev() {
            if defs.contains(name) {
                return;
            }
        }
        self.caps.borrow_mut().entry(name).or_insert(span);
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
                self.caps.borrow_mut().entry(name).or_insert(span);
            }
        }
        if let Some(attrs) = attrs {
            self.attrs.borrow_mut().extend(attrs);
        }
    }
}

impl<'src> WalkerArtifact<'src> {
    pub fn captures(&self) -> Vec<(&'src str, TextSpan)> {
        if let Some(caps) = &self.caps {
            let mut res = caps
                .iter()
                .map(|(name, span)| (*name, *span))
                .collect::<Vec<_>>();
            res.sort_unstable_by_key(|(name, _)| *name);
            res
        } else {
            Vec::new()
        }
    }

    pub fn take_attributes(&mut self) -> Vec<(&'src str, Vec<TextSpan>)> {
        if let Some(attrs) = self.attrs.take() {
            let mut res = attrs
                .into_iter()
                .map(|(name, spans)| {
                    let mut spans = spans;
                    spans.sort_unstable_by_key(|span| span.start());
                    (name, spans)
                })
                .collect::<Vec<_>>();
            res.sort_unstable_by_key(|(_, spans)| spans[0].start());
            res
        } else {
            panic!("`attributes` should only be collected once.");
        }
    }
}

mod walkable_impl {
    use super::*;

    impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Block<'src> {
        fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
            for (statement, _) in self.0.iter_mut() {
                walker.go(statement);
            }
        }
    }

    impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Statement<'src> {
        fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
            match self {
                Statement::Var {
                    name: (name, _),
                    expr: (expr, _),
                } => {
                    walker.record_variable_definition(name);
                    walker.go(expr);
                }
                Statement::Func {
                    name: (name, _),
                    args,
                    body,
                } => {
                    walker.record_variable_definition(name);
                    let result = {
                        let mut walker = Walker::new();
                        for (_, arg, _) in args {
                            walker.record_variable_definition(arg);
                        }
                        walker.go(&mut body.block);
                        let result = walker.finish();
                        body.captures = result.captures();
                        result
                    };
                    walker.merge(result);
                }
                Statement::FieldFunc {
                    table: (table, table_span),
                    fields: _,
                    args,
                    body,
                } => {
                    walker.record_variable_usage(table, *table_span);
                    let result = {
                        let mut walker = Walker::new();
                        for (_, arg, _) in args {
                            walker.record_variable_definition(arg);
                        }
                        walker.go(&mut body.block);
                        let result = walker.finish();
                        body.captures = result.captures();
                        result
                    };
                    walker.merge(result);
                }
                Statement::Assign {
                    name: (name, name_span),
                    expr: (expr, _),
                } => {
                    walker.go(expr);
                    walker.record_variable_usage(name, *name_span);
                }
                Statement::FieldAssign {
                    table: (table, _),
                    field: (field, _),
                    expr: (expr, _),
                } => {
                    walker.go(expr);
                    walker.go(table);
                    walker.go(field);
                }
                Statement::If {
                    cond: (cond, _),
                    body,
                    elifs,
                    else_,
                } => {
                    walker.go(cond);
                    walker.fork().go(body);
                    for ((cond, _), body) in elifs {
                        walker.go(cond);
                        walker.fork().go(body);
                    }
                    if let Some(else_) = else_ {
                        walker.fork().go(else_);
                    }
                }
                Statement::For {
                    value: (value, _),
                    iter: (iter, _),
                    body,
                } => {
                    walker.go(iter);
                    walker.record_variable_definition(value);
                    walker.fork().go(body);
                }
                Statement::While {
                    cond: (cond, _),
                    body,
                } => {
                    walker.go(cond);
                    walker.fork().go(body);
                }
                Statement::Do { body } => {
                    walker.fork().go(body);
                }
                Statement::Return { value } => {
                    if let Some((value, _)) = value {
                        walker.go(value);
                    }
                }
                Statement::Continue => {}
                Statement::Break => {}
                Statement::Call {
                    expr: (expr, _),
                    args,
                } => {
                    walker.go(expr);
                    for (expr, _) in args {
                        walker.go(expr);
                    }
                }
                Statement::MethodCall {
                    expr: (expr, _),
                    name: _,
                    args,
                } => {
                    walker.go(expr);
                    for (expr, _) in args {
                        walker.go(expr);
                    }
                }
                Statement::Attribute {
                    name: (name, name_span),
                    args: _,
                } => {
                    walker.record_attribute(name, *name_span);
                }
                Statement::Error => {}
            }
        }
    }

    impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Expression<'src> {
        fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
            walker_accept_expression(self, walker);
        }
    }
    impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Box<Expression<'src>> {
        fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
            walker_accept_expression(self, walker);
        }
    }
    fn walker_accept_expression<'walker, 'src: 'walker>(
        expr: &mut Expression<'src>,
        walker: &mut Walker<'walker, 'src>,
    ) {
        match expr {
            Expression::Unary {
                op: _,
                expr: (expr, _),
            } => {
                walker.go(expr);
            }
            Expression::Binary {
                op: _,
                lhs: (lhs, _),
                rhs: (rhs, _),
            } => {
                walker.go(lhs);
                walker.go(rhs);
            }
            Expression::Local(name, span) => {
                walker.record_variable_usage(name, *span);
            }
            Expression::Primitive(_, _) => {}
            Expression::TableObject(table) => {
                for (key, (value, _)) in table.iter_mut() {
                    match key {
                        TableFieldKey::Ident(_, _) => {}
                        TableFieldKey::Expr(expr, _) => {
                            walker.go(expr);
                        }
                    }
                    walker.go(value);
                }
            }
            Expression::ArrayObject(array) => {
                for (expr, _) in array.iter_mut() {
                    walker.go(expr);
                }
            }
            Expression::FunctionObject(func) => {
                let result = {
                    let mut waker = Walker::new();
                    for (_, arg, _) in func.args.iter() {
                        waker.record_variable_definition(arg);
                    }
                    waker.go(&mut func.body.block);
                    let result = waker.finish();
                    func.body.captures = result.captures();
                    result
                };
                walker.merge(result);
            }
            Expression::Call {
                expr: (expr, _),
                args,
            } => {
                walker.go(expr);
                for (arg, _) in args {
                    walker.go(arg);
                }
            }
            Expression::MethodCall {
                expr: (expr, _),
                name: _,
                args,
            } => {
                walker.go(expr);
                for (arg, _) in args {
                    walker.go(arg);
                }
            }
            Expression::IndexAccess {
                expr: (expr, _),
                accessor: (accesser, _),
            } => {
                walker.go(expr);
                walker.go(accesser);
            }
            Expression::DotAccess {
                expr: (expr, _),
                accessor: _,
            } => {
                walker.go(expr);
            }
            Expression::Error => panic!("Error expression found."),
        }
    }
}
