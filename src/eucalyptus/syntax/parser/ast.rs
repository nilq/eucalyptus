use super::{ParserResult, ParserError};

use std::rc::Rc;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    Bool(bool),
    Str(Rc<String>),
    Char(char),
    Identifier(Rc<String>),
}


pub enum Statement {
    Expression(Rc<Expression>),
}
