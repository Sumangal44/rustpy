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
    pub defaults: Vec<Rc<dyn PyObject>>,
    pub posonly_count: usize,
    pub kwonly_params: Vec<String>,
    pub kwonly_defaults: Vec<Rc<dyn PyObject>>,
}

impl PyFunction {
    pub fn new(
        name: String,
        params: Vec<String>,
        code: CodeObject,
        env: Rc<RefCell<Environment>>,
        defaults: Vec<Rc<dyn PyObject>>,
        posonly_count: usize,
        kwonly_params: Vec<String>,
        kwonly_defaults: Vec<Rc<dyn PyObject>>,
    ) -> Self {
        Self {
            name,
            params,
            code,
            env,
            defaults,
            posonly_count,
            kwonly_params,
            kwonly_defaults,
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
