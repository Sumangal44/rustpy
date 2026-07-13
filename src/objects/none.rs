use super::PyObject;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyNone;

impl PyNone {
    pub fn new() -> Self {
        Self
    }
}

impl PyObject for PyNone {
    fn get_type(&self) -> &'static str {
        "NoneType"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        "None".to_string()
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        false
    }

    fn hash(&self) -> Result<i64, String> {
        Ok(0)
    }
}
