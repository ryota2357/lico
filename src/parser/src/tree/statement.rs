use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'src> {
    Control(ControlStatement<'src>),
    Attribute(AttributeStatement<'src>),
    Variable(VariableStatement<'src>),
    Call(CallStatement<'src>),
}

/// <Statement> ::= <ControlStatement> | <AttributeStatement> | <VariableStatement> | <CallStatement>
pub(super) fn statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, (Statement<'src>, Span), ParserError<'src>> + Clone
{
    let expr = expression(block.clone());

    let control = control_statement(block.clone(), expr.clone()).map(Statement::Control);
    let attribute = attribute_statement().map(Statement::Attribute);
    let variable = variable_statement(block, expr.clone()).map(Statement::Variable);
    let call = call_statement(expr).map(Statement::Call);

    choice((control, attribute, variable, call))
        .map_with(|statement, extra| (statement, extra.span().into()))
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Statement<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        match self {
            Statement::Control(control) => walker.go(control),
            Statement::Attribute(attribute) => walker.go(attribute),
            Statement::Variable(variable) => walker.go(variable),
            Statement::Call(call) => walker.go(call),
        }
    }
}
