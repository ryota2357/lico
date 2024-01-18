use foundation::{ast::*, TextSpan};

type Result<T> = std::result::Result<T, Error>;

mod error;
pub use error::Error;

mod tools;
use tools::*;

mod compile;
pub use compile::compile;
