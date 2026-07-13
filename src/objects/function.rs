use super::PyObject;
use crate::compiler::code::CodeObject;
use crate::runtime::Environment;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyFunction {
    pub name: String,
    pub params: Vec<String>,
    pub code: CodeObject,
    pub env: Rc<RefCell<Environment>>,
}

impl PyFunction {
    pub fn new(
        name: String,
        params: Vec<String>,
        code: CodeObject,
        env: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            code,
            env,
        }
    }
}

impl std::fmt::Debug for PyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function {} at {:p}>", self.name, self)
    }
}

impl PyObject for PyFunction {
    fn get_type(&self) -> &'static str {
        "function"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<function {} at {:p}>", self.name, self)
    }

    fn is_truthy(&self) -> bool {
        true
    }
}
