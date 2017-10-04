use std::rc::Rc;

use super::*;

pub trait Visitor {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()>;
}

pub trait Evaluator {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value>;
}

pub trait Typer {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<Type>;
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
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        match *self {
            Expression::Block(ref statements) => {
                for s in statements {
                    s.visit(sym, env, val)?
                }
                Ok(())
            },

            Expression::Array(ref body) => {
                for v in body {
                    v.visit(sym, env, val)?
                }
                Ok(())
            }

            Expression::Identifier(ref id) => match sym.get_name(&*id) {
                Some(_) => Ok(()),
                None    => Err(RunError::new(&format!("{}: undeclared use", id))),
            },

            Expression::Operation(ref operation) => operation.visit(sym, env, val),
            Expression::Call(ref call)           => call.visit(sym, env, val),
            Expression::Index(ref index)         => index.visit(sym, env, val),

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
            
            Expression::Block(ref statements) => {
                match statements.last() {
                    Some(s) => Ok(s.eval(sym, env)?),
                    None    => Err(RunError::new(&format!("found empty block"))),
                }
            },

            Expression::Array(ref content) => {
                let mut stack = Vec::new();

                for c in content {
                    stack.push(Rc::new(c.eval(sym, env)?))
                }

                Ok(Value::Array(stack))
            },
            
            Expression::Index(ref index) => index.eval(sym, env),

            Expression::Identifier(ref id) => match sym.get_name(&*id) {
                Some((a, b)) => Ok(env.get_value(a, b)?),
                None         => Err(RunError::new(&format!("{}: undeclared use", id))),
            },

            Expression::Operation(ref operation) => operation.eval(sym, env),
            Expression::Lambda(ref lambda)       => lambda.eval(sym, env),
            Expression::Call(ref call)           => call.eval(sym, env),

            _ => Ok(Value::Nil),
        }
    }
}

impl Typer for Expression {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<Type> {
        match *self {
            Expression::Number(_)          => Ok(Type::Number),
            Expression::Str(_)             => Ok(Type::Str),
            Expression::Char(_)            => Ok(Type::Char),
            Expression::Bool(_)            => Ok(Type::Bool),
            Expression::Array(ref content) => {
                let mut types = Vec::new();

                for c in content {
                    types.push(Rc::new(c.get_type(sym, env, val)?))
                }

                Ok(Type::Array(types))
            }
            Expression::Identifier(ref n) => match sym.get_name(&*n) {
                Some((i, env_index)) => {
                    Ok(env.get_type(i, env_index).unwrap())
                },
                None => Err(RunError::new(&format!("unexpected use of: {}", n))),
            },
            Expression::Operation(ref operation) => operation.get_type(sym, env, val),
            Expression::Index(ref index)         => index.get_type(sym, env, val),
            _ => Ok(Type::Undefined),
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
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        self.left.visit(sym, env, val)?;
        self.right.visit(sym, env, val)
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

impl Typer for Operation {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<Type> {
        match self.op {
            Operand::Pow => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}^{:?}): failed to operate", a, b)))
            },
            
            Operand::Mul => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}*{:?}): failed to operate", a, b)))
            },
            
            Operand::Div => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}/{:?}): failed to operate", a, b)))
            },
            
            Operand::Mod => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}%{:?}): failed to operate", a, b)))
            },
            
            Operand::Add => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}+{:?}): failed to operate", a, b)))
            },
            
            Operand::Sub => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Number),
                (Type::Any, Type::Number)    => Ok(Type::Any),
                (Type::Number, Type::Any)    => Ok(Type::Any),
                (a, b) => Err(RunError::new(&format!("({:?}-{:?}): failed to operate", a, b)))
            },
            
            Operand::Lt => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Bool),
                (Type::Str, Type::Str)       => Ok(Type::Bool),
                (Type::Char, Type::Char)     => Ok(Type::Bool),
                (a, b) => Err(RunError::new(&format!("({:?}<{:?}): failed to compare", a, b)))
            },
            
            Operand::Gt => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Bool),
                (Type::Str, Type::Str)       => Ok(Type::Bool),
                (Type::Char, Type::Char)     => Ok(Type::Bool),
                (a, b) => Err(RunError::new(&format!("({:?}>{:?}): failed to compare", a, b)))
            },
            
            Operand::LtEqual => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Bool),
                (Type::Str, Type::Str)       => Ok(Type::Bool),
                (Type::Char, Type::Char)     => Ok(Type::Bool),
                (a, b) => Err(RunError::new(&format!("({:?}<={:?}): failed to compare", a, b)))
            },

            Operand::GtEqual => match (self.left.get_type(sym, env, val)?, self.right.get_type(sym, env, val)?) {
                (Type::Number, Type::Number) => Ok(Type::Bool),
                (Type::Str, Type::Str)       => Ok(Type::Bool),
                (Type::Char, Type::Char)     => Ok(Type::Bool),
                (a, b) => Err(RunError::new(&format!("({:?}>={:?}): failed to compare", a, b)))
            },

            _ => Ok(Type::Undefined),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub params: Vec<Rc<String>>,
    pub body:   Rc<Expression>,
}

impl Evaluator for Lambda {
    fn eval(&self, _: &Rc<SymTab>, _: &Rc<ValTab>) -> RunResult<Value> {
        let body = match *self.body {
            Expression::Block(ref s) => s.clone(),
            ref e => vec![Statement::Expression(Rc::new(e.clone()))],
        };
        
        Ok(
            Value::Function(
                self.params.clone(),
                body,
            )
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Rc<Expression>,
    pub args:   Vec<Rc<Expression>>,
}

impl Visitor for Call {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        self.callee.visit(sym, env, val)?;

        for arg in self.args.iter() {
            arg.visit(sym, env, val)?
        }
        
        Ok(())
    }
}

impl Evaluator for Call {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match self.callee.eval(sym, env)? {
            Value::Function(params, body) => {
                let local_sym = Rc::new(SymTab::new(sym.clone(), &params));
                
                let mut arg_vals = Vec::new();
                
                for a in self.args.clone() {
                    arg_vals.push(a.eval(sym, env)?)
                }
                
                let local_env = Rc::new(ValTab::new(env.clone(), &arg_vals));

                Ok(Expression::Block(body).eval(&local_sym, &local_env)?)
            },
            _ => Ok(Value::Nil),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub id:    Rc<Expression>,
    pub index: Rc<Expression>,
}

impl Visitor for Index {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        self.id.visit(sym, env, val)?;
        self.index.visit(sym, env, val)
    }
}

impl Evaluator for Index {
    fn eval(&self, sym: &Rc<SymTab>, env: &Rc<ValTab>) -> RunResult<Value> {
        match self.id.eval(sym, env)? {
            Value::Array(content) => match self.index.eval(sym, env)? {
                Value::Number(n) => Ok((&*content.clone().remove(n as usize).clone()).clone()),
                c => Err(RunError::new(&format!("{:?}: invalid index", c))),
            },
            _ => Ok(Value::Nil)
        }
    }
}

impl Typer for Index {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<Type> {
        match self.id.get_type(sym, env, val)? {
            Type::Array(content) => match self.index.eval(sym, val)? {
                Value::Number(n) => Ok((&*content.clone().remove(n as usize).clone()).clone()),
                c => Err(RunError::new(&format!("{:?}: invalid index", c))),
            },
            Type::Any => Ok(Type::Any),
            _ => Err(RunError::new(&format!("{:?}: can't index", self.id))),
        }
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
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        match *self {
            Statement::Expression(ref e)    => e.visit(sym, env, val),
            Statement::Binding(ref binding) => binding.visit(sym, env, val),
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

impl Typer for Statement {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<Type> {
        match *self {
            Statement::Expression(ref e)    => e.get_type(sym, env, val),
            _ => Ok(Type::Undefined),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub left:  Rc<Expression>,
    pub right: Rc<Expression>,
}

impl Visitor for Binding {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>, val: &Rc<ValTab>) -> RunResult<()> {
        match *self.left {
            Expression::Identifier(ref name) => {
                let index = sym.add_name(name);
                if index >= env.size() {
                    env.grow();
                }
                
                if let Err(e) = env.set_type(index, 0, self.right.get_type(sym, env, val)?) {
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
