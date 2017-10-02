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
    Operation(Operation),
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub left:  Rc<Expression>,
    pub op:    Operand,
    pub right: Rc<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Rc<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Pow,
    Mul, Div, Mod,
    Add, Sub,
    Equal, NEqual,
    Lt, Gt, LtEqual, GtEqual,
    Not,
}

impl Operand {
    pub fn from_str(v: &str) -> Option<(Operand, u8)> {
        match v {
            "^"   => Some((Operand::Pow, 0)),
            "*"   => Some((Operand::Mul, 1)),
            "/"   => Some((Operand::Div, 1)),
            "%"   => Some((Operand::Mod, 1)),
            "+"   => Some((Operand::Add, 2)),
            "-"   => Some((Operand::Sub, 2)),
            "=="  => Some((Operand::Equal, 3)),
            "!="  => Some((Operand::NEqual, 3)),
            "<"   => Some((Operand::Lt, 4)),
            ">"   => Some((Operand::Gt, 4)),
            "<="  => Some((Operand::LtEqual, 4)),
            ">="  => Some((Operand::GtEqual, 4)),
            "!"   => Some((Operand::Not, 4)),
            _     => None,
        }
    }
}
