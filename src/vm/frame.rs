use crate::compiler::code::CodeObject;
use crate::objects::PyObject;
use crate::runtime::Environment;
use std::rc::Rc;

pub struct Frame {
    pub code: CodeObject,
    pub ip: usize,
    pub stack: Vec<Rc<dyn PyObject>>,
    pub env: Environment,
}

impl Frame {
    pub fn new(code: CodeObject, env: Environment) -> Self {
        Self {
            code,
            ip: 0,
            stack: Vec::new(),
            env,
        }
    }

    pub fn push(&mut self, obj: Rc<dyn PyObject>) {
        self.stack.push(obj);
    }

    pub fn pop(&mut self) -> Result<Rc<dyn PyObject>, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Pop from empty stack".to_string())
    }
}
