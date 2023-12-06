#[derive(Default, Debug, PartialEq)]
pub struct Global {}

impl Global {
    pub const fn new() -> Self {
        Self {}
    }
}
