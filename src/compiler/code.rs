use super::PyObject;
use crate::compiler::opcodes::Opcode;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CodeObject {
    pub instructions: Vec<Opcode>,
    pub constants: Vec<Rc<dyn PyObject>>,
    pub names: Vec<String>,
    pub name: String,
    pub is_generator: bool,
}

impl CodeObject {
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            is_generator: false,
        }
    }
}

impl PyObject for CodeObject {
    fn get_type(&self) -> &'static str {
        "code"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<code object {} at {:p}>", self.name, self)
    }
}
