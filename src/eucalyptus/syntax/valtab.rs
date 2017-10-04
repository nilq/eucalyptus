use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(Rc<String>),
    Char(char),
    Array(Vec<Rc<Value>>),
    Function(Vec<Rc<String>>, Vec<Statement>),
    Nil,
}

pub struct ValTab {
    parent: Option<Rc<ValTab>>,
    types: RefCell<Vec<Value>>,
}

impl ValTab {
    pub fn new(parent: Rc<ValTab>, types: &Vec<Value>) -> ValTab {
        ValTab {
            parent: Some(parent),
            types:  RefCell::new(types.clone()),
        }
    }

    pub fn new_global() -> ValTab {
        ValTab {
            parent: None,
            types:  RefCell::new(Vec::new()),
        }
    }

    pub fn new_partial(parent: Rc<ValTab>, types: &[Value], size: usize) -> ValTab {
        let mut stack = types.to_vec();
        for _ in 0 .. size - types.len() {
            stack.push(Value::Nil)
        }

        ValTab {
            parent: Some(parent),
            types: RefCell::new(stack),
        }
    }

    pub fn set_value(&self, index: usize, env_index: usize, t: Value) -> RunResult<()> {
        if env_index == 0 {
            let mut types = self.types.borrow_mut();
            match types.get_mut(index) {
                Some(v) => {
                    *v = t;
                    Ok(())
                },
                None => Err(RunError::new(&format!("can't set value of invalid value index: {}", index))),
            }
        } else {
            match self.parent {
                Some(ref p) => p.set_value(index, env_index - 1, t),
                None => Err(RunError::new(&format!("can't set value with invalid env index: {}", env_index))),
            }
        }
    }

    pub fn get_value(&self, index: usize, env_index: usize) -> RunResult<Value> {
        if env_index == 0 {
            match self.types.borrow().get(index) {
                Some(v) => Ok(v.clone()),
                None    => Err(RunError::new(&format!("can't get value of invalid value index: {}", index))),
            }
        } else {
            match self.parent {
                Some(ref p) => p.get_value(index, env_index - 1),
                None => Err(RunError::new(&format!("can't get value with invalid value index: {}", index))),
            }
        }
    }

    pub fn visualize(&self, env_index: usize) {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                p.visualize(env_index - 1);
                println!("------------------------------");
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            println!("({} : {}) = {:?}", i, env_index, v)
        }
    }

    fn dump(&self, f: &mut fmt::Formatter, env_index: usize) -> fmt::Result {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                try!(p.dump(f, env_index - 1));
                try!(writeln!(f, "------------------------------"));
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            try!(writeln!(f, "({} : {}) = {:?}", i, env_index, v))
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.types.borrow().len()
    }

    pub fn grow(&self) {
        self.types.borrow_mut().push(Value::Nil)
    }
}

impl fmt::Debug for ValTab {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}
