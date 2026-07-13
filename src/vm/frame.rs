use crate::compiler::code::CodeObject;
use crate::objects::PyObject;
use crate::runtime::Environment;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Block {
    SetupExcept {
        handler_ip: usize,
        stack_size: usize,
    },
    SetupWith {
        handler_ip: usize,
        stack_size: usize,
        exit_func: Rc<dyn PyObject>,
    },
}

pub struct Frame {
    pub code: CodeObject,
    pub ip: usize,
    pub stack: Vec<Rc<dyn PyObject>>,
    pub env: Rc<RefCell<Environment>>,
    pub block_stack: Vec<Block>,
}

impl Frame {
    pub fn new(code: CodeObject, env: Rc<RefCell<Environment>>) -> Self {
        Self {
            code,
            ip: 0,
            stack: Vec::new(),
            env,
            block_stack: Vec::new(),
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

    pub fn last(&self) -> Result<Rc<dyn PyObject>, String> {
        self.stack
            .last()
            .cloned()
            .ok_or_else(|| "Peek from empty stack".to_string())
    }
}
