use super::PyObject;
use std::any::Any;

#[derive(Debug)]
pub struct PyException {
    pub exc_type: String,
    pub message: Option<String>,
}

impl PyException {
    pub fn new(exc_type: String, message: Option<String>) -> Self {
        Self { exc_type, message }
    }
}

impl PyObject for PyException {
    fn get_type(&self) -> &'static str {
        "exception"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if let Some(msg) = &self.message {
            format!("{}(\"{}\")", self.exc_type, msg)
        } else {
            format!("{}()", self.exc_type)
        }
    }

    fn str(&self) -> String {
        if let Some(msg) = &self.message {
            msg.clone()
        } else {
            "".to_string()
        }
    }
}
