use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

// The base trait that all Python objects must implement
pub trait PyObject: Debug + Any {
    fn get_type(&self) -> &'static str;

    // Default implementations for standard magic methods
    fn as_any(&self) -> &dyn Any;

    fn repr(&self) -> String {
        format!("<{} object at {:p}>", self.get_type(), self)
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        true
    }

    // Mathematical operations (default to NotImplemented error behavior via None)
    fn add(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn sub(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn mul(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
}

pub mod int;
