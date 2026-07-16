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
    pub filename: String,
    pub is_generator: bool,
    pub is_async: bool,
    pub arg_count: usize,
    #[allow(dead_code)]
    pub default_count: usize,
    pub posonly_count: usize,
    pub kwonly_params: Vec<String>,
    pub vararg: Option<String>,
    pub kwarg: Option<String>,
    pub nonlocal_names: Vec<String>,
}

impl CodeObject {
    pub fn new(name: String) -> Self {
        Self {
            name,
            filename: String::new(),
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            is_generator: false,
            is_async: false,
            arg_count: 0,
            default_count: 0,
            posonly_count: 0,
            kwonly_params: Vec::new(),
            vararg: None,
            kwarg: None,
            nonlocal_names: Vec::new(),
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
