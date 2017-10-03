use std::rc::Rc;

use super::*;

pub trait Visitor {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()>;
}

pub trait Evaluator {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Block(Vec<Statement>),
    Number(f64),
    Bool(bool),
    Str(Rc<String>),
    Char(char),
    Array(Vec<Rc<Expression>>),
    Identifier(Rc<String>),
    Operation(Operation),
    Lambda(Lambda),
    Call(Call),
    Index(Index),
    EOF,
}

impl Visitor for Expression {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        match *self {
            Expression::Block(ref statements) => {
                for s in statements {
                    s.visit(&sym, &env)?
                }
                Ok(())
            },
            
            Expression::Array(ref body) => {
                for v in body {
                    v.visit(sym, env)?
                }
                Ok(())
            }

            Expression::Identifier(ref id) => match sym.get_name(&*id) {
                Some(_) => Ok(()),
                None    => Err(RunError::new(&format!("{}: undeclared use", id))),
            },

            Expression::Operation(ref operation) => operation.visit(sym, env),
            Expression::Lambda(ref lambda)       => lambda.visit(sym, env),
            Expression::Call(ref call)           => call.visit(sym, env),
            Expression::Index(ref index)         => index.visit(sym, env),

            _ => Ok(()),
        }
    }
}

impl Evaluator for Expression {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match *self {
            Expression::Number(n)  => Ok(Value::Number(n)),
            Expression::Bool(n)    => Ok(Value::Bool(n)),
            Expression::Str(ref n) => Ok(Value::Str(n.clone())),
            Expression::Char(n)    => Ok(Value::Char(n)),

            Expression::Array(ref content) => {
                let mut stack = Vec::new();

                for c in content {
                    stack.push(Rc::new(c.eval(sym, env)?))
                }

                Ok(Value::Array(stack))
            },

            Expression::Identifier(ref id) => match sym.get_name(&*id) {
                Some((a, b)) => Ok(env.get_value(a, b)?),
                None         => Ok(Value::Nil),
            },

            Expression::Operation(ref operation) => operation.eval(sym, env),

            _ => Ok(Value::Nil),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub left:  Rc<Expression>,
    pub op:    Operand,
    pub right: Rc<Expression>,
}

impl Visitor for Operation {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        self.left.visit(sym, env)?;
        self.right.visit(sym, env)
    }
}

impl Evaluator for Operation {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match self.op {
            Operand::Pow => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(b))),
                _ => Ok(Value::Nil),
            },
            
            Operand::Mul => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Ok(Value::Nil),
            },
            
            Operand::Div => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                _ => Ok(Value::Nil),
            },
            
            Operand::Mod => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                _ => Ok(Value::Nil),
            },
            
            Operand::Add => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Array(a), b)                 => {
                    let mut c = a.clone();
                    c.push(Rc::new(b));
                    Ok(Value::Array(c))
                },
                _ => Ok(Value::Nil),
            },
            
            Operand::Sub => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Ok(Value::Nil),
            },
            
            Operand::Equal => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (a, b) => Ok(Value::Bool(a == b)),
            },
            
            Operand::NEqual => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (a, b) => Ok(Value::Bool(a != b)),
            },
            
            Operand::Lt => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                (Value::Str(a), Value::Str(b))       => Ok(Value::Bool(a < b)),
                (Value::Char(a), Value::Char(b))     => Ok(Value::Bool(a < b)),
                (Value::Array(a), Value::Array(b))   => Ok(Value::Bool(a.len() < b.len())),
                _ => Ok(Value::Nil),
            },
            
            Operand::Gt => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                (Value::Str(a), Value::Str(b))       => Ok(Value::Bool(a > b)),
                (Value::Char(a), Value::Char(b))     => Ok(Value::Bool(a > b)),
                (Value::Array(a), Value::Array(b))   => Ok(Value::Bool(a.len() > b.len())),
                _ => Ok(Value::Nil),
            },
            
            Operand::LtEqual => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                (Value::Str(a), Value::Str(b))       => Ok(Value::Bool(a <= b)),
                (Value::Char(a), Value::Char(b))     => Ok(Value::Bool(a <= b)),
                (Value::Array(a), Value::Array(b))   => Ok(Value::Bool(a.len() <= b.len())),
                _ => Ok(Value::Nil),
            },

            Operand::GtEqual => match (self.left.eval(sym, env)?, self.right.eval(sym, env)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                (Value::Str(a), Value::Str(b))       => Ok(Value::Bool(a >= b)),
                (Value::Array(a), Value::Array(b))   => Ok(Value::Bool(a.len() >= b.len())),
                _ => Ok(Value::Nil),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub params: Vec<Rc<String>>,
    pub body:   Rc<Expression>,
}

impl Visitor for Lambda {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        let local_sym = Rc::new(SymTab::new(sym.clone(), &self.params));
        let local_env = Rc::new(TypeTab::new(env.clone(), &self.params.iter().map(|_| Type::Any).collect()));

        self.body.visit(&local_sym, &local_env)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Rc<Expression>,
    pub args:   Vec<Rc<Expression>>,
}

impl Visitor for Call {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        self.callee.visit(sym, env)?;

        for arg in self.args.iter() {
            arg.visit(sym, env)?
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub id:    Rc<Expression>,
    pub index: Rc<Expression>,
}

impl Visitor for Index {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        self.id.visit(sym, env)?;
        self.index.visit(sym, env)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Rc<Expression>),
    Binding(Binding),
    Function(Function),
    Assignment(Assignment),
}

impl Visitor for Statement {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        match *self {
            Statement::Expression(ref e)    => e.visit(sym, env),
            Statement::Binding(ref binding) => binding.visit(sym, env),
            _ => Ok(()),
        }
    }
}

impl Evaluator for Statement {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match *self {
            Statement::Expression(ref e)    => e.eval(sym, env),
            Statement::Binding(ref binding) => binding.eval(sym, env),
            _ => Ok(Value::Nil),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub left:  Rc<Expression>,
    pub right: Rc<Expression>,
}

impl Visitor for Binding {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> RunResult<()> {
        match *self.left {
            Expression::Identifier(ref name) => {
                let index = sym.add_name(name);
                if index >= env.size() {
                    env.grow();
                }
                
                if let Err(e) = env.set_type(index, 0, Type::Any) {
                    Err(RunError::new(&format!("{}: error setting type", e)))
                } else {
                    Ok(())
                }
            }
            
            ref e => Err(RunError::new(&format!("{:?}: unexpected binding", e)))
        }
    }
}

impl Evaluator for Binding {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match *self.left {
            Expression::Identifier(ref name) => {
                let index = sym.add_name(name);
                if index >= env.size() {
                    env.grow();
                }

                if let Err(e) = env.set_value(index, 0, self.right.eval(sym, env)?) {
                    Err(RunError::new(&format!("{}: error setting value", e)))
                } else {
                    Ok(Value::Nil)
                }
            }
            
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name:   Rc<String>,
    pub params: Vec<Rc<String>>,
    pub body:   Rc<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub left:  Rc<Expression>,
    pub right: Rc<Expression>,
}

impl Evaluator for Assignment {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match *self.left {
            Expression::Identifier(ref name) => {
                let (a, b) = match sym.get_name(name) {
                    Some((a, b)) => (a, b),
                    None         => return Err(RunError::new(&format!("{}: undeclared variable", name))),
                };

                if let Err(e) = env.set_value(a, b, self.right.eval(sym, env)?) {
                    Err(RunError::new(&format!("{}: error setting value", e)))
                } else {
                    Ok(Value::Nil)
                }
            }
            
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Pow,
    Mul, Div, Mod,
    Add, Sub,
    Equal, NEqual,
    Lt, Gt, LtEqual, GtEqual,
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
            _     => None,
        }
    }
}
