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
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Statement<'src>, ParserError<'tokens, 'src>> + Clone
{
    let expr = expression(block.clone());

    let control = control_statement(block.clone(), expr.clone()).map(Statement::Control);
    let attribute = attribute_statement().map(Statement::Attribute);
    let variable = variable_statement(block, expr.clone()).map(Statement::Variable);
    let call = call_statement(expr).map(Statement::Call);

    choice((control, attribute, variable, call))
}

impl<'a> TreeWalker<'a> for Statement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Statement::Control(stat) => stat.analyze(tracker),
            Statement::Attribute(stat) => stat.analyze(tracker),
            Statement::Variable(stat) => stat.analyze(tracker),
            Statement::Call(stat) => stat.analyze(tracker),
        }
    }
}
