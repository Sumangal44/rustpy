use crate::compiler::opcodes::Opcode;
use crate::objects::PyObject;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CodeObject {
    pub instructions: Vec<Opcode>,
    pub constants: Vec<Rc<dyn PyObject>>,
    pub names: Vec<String>,
}

impl CodeObject {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
        }
    }
}
