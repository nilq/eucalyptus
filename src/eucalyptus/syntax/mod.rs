pub mod lexer;
pub mod parser;
pub mod symtab;
pub mod typetab;
pub mod valtab;
pub mod error;

pub use self::lexer::*;
pub use self::parser::*;
pub use self::symtab::*;
pub use self::typetab::*;
pub use self::valtab::*;
pub use self::error::*;

pub type RunResult<T> = Result<T, RunError>;
