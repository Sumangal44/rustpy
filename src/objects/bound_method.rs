use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyBoundMethod {
    pub instance: Rc<dyn PyObject>,
    pub func: Rc<dyn PyObject>,
}

impl PyBoundMethod {
    pub fn new(instance: Rc<dyn PyObject>, func: Rc<dyn PyObject>) -> Self {
        Self { instance, func }
    }
}

impl std::fmt::Debug for PyBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyBoundMethod {
    fn get_type(&self) -> &'static str {
        "method"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<bound method of {}>", self.instance.repr())
    }
}
