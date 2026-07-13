use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyBool {
    pub value: bool,
}

impl PyBool {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl PyObject for PyBool {
    fn get_type(&self) -> &'static str {
        "bool"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if self.value {
            "True".to_string()
        } else {
            "False".to_string()
        }
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        self.value
    }
}
