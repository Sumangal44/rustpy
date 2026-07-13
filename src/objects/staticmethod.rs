use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyStaticMethod {
    pub func: Rc<dyn PyObject>,
}

impl PyStaticMethod {
    pub fn new(func: Rc<dyn PyObject>) -> Self {
        Self { func }
    }
}

impl std::fmt::Debug for PyStaticMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<staticmethod {}>", self.func.repr())
    }
}

impl PyObject for PyStaticMethod {
    fn get_type(&self) -> &'static str {
        "staticmethod"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<staticmethod {}>", self.func.repr())
    }

    fn is_truthy(&self) -> bool {
        true
    }
}
