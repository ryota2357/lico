mod common;

use common::do_test;
// use lexer::Token;

#[test]
#[should_panic]
fn invalid_attribute() {
    do_test("@ attr", vec![]);
}
