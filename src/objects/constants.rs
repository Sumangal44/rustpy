use super::PyObject;
use std::any::Any;

#[derive(Debug)]
pub struct PyNotImplemented;

impl PyObject for PyNotImplemented {
    fn get_type(&self) -> &'static str {
        "NotImplementedType"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        "NotImplemented".to_string()
    }
}

#[derive(Debug)]
pub struct PyEllipsis;

impl PyObject for PyEllipsis {
    fn get_type(&self) -> &'static str {
        "ellipsis"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        "Ellipsis".to_string()
    }
}
