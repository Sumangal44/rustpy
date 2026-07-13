use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyString {
    pub value: String,
}

impl PyString {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl PyObject for PyString {
    fn get_type(&self) -> &'static str {
        "str"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("'{}'", self.value)
    }

    fn str(&self) -> String {
        self.value.clone()
    }

    fn is_truthy(&self) -> bool {
        !self.value.is_empty()
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_str) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(PyString::new(format!(
                "{}{}",
                self.value, other_str.value
            ))))
        } else {
            None // NotImplemented
        }
    }
}
