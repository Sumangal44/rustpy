use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyInt {
    pub value: i64,
}

impl PyInt {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

impl PyObject for PyInt {
    fn get_type(&self) -> &'static str {
        "int"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        self.value.to_string()
    }

    fn str(&self) -> String {
        self.value.to_string()
    }

    fn is_truthy(&self) -> bool {
        self.value != 0
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_int) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value + other_int.value)))
        } else {
            None // NotImplemented
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_int) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value - other_int.value)))
        } else {
            None // NotImplemented
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_int) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value * other_int.value)))
        } else {
            None // NotImplemented
        }
    }
}
