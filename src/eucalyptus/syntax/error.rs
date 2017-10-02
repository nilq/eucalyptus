use std::fmt;

#[derive(Debug)]
pub enum RunErrorValue {
    Constant(String),
}

#[derive(Debug)]
pub struct RunError {
    value: RunErrorValue,
}

impl RunError {
    pub fn new(value: &str) -> RunError {
        RunError {
            value:    RunErrorValue::Constant(value.to_owned()),
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            RunErrorValue::Constant(ref s) => write!(f, "{}", s),
        }
    }
}
