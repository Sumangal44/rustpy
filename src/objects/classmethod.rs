use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyClassMethod {
    pub func: Rc<dyn PyObject>,
}

impl PyClassMethod {
    pub fn new(func: Rc<dyn PyObject>) -> Self {
        Self { func }
    }
}

impl std::fmt::Debug for PyClassMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<classmethod {}>", self.func.repr())
    }
}

impl PyObject for PyClassMethod {
    fn get_type(&self) -> &'static str {
        "classmethod"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<classmethod {}>", self.func.repr())
    }

    fn is_truthy(&self) -> bool {
        true
    }
}
