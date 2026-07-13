use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

pub trait PyObject: Debug + Any {
    fn get_type(&self) -> &'static str;

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

    fn add(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn sub(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn mul(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Err(format!(
            "TypeError: '{}' object is not iterable",
            self.get_type()
        ))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        Err(format!(
            "TypeError: '{}' object is not an iterator",
            self.get_type()
        ))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        Err(format!(
            "AttributeError: '{}' object has no attribute '{}'",
            self.get_type(),
            attr
        ))
    }

    fn set_attr(&self, attr: &str, _value: Rc<dyn PyObject>) -> Result<(), String> {
        Err(format!(
            "AttributeError: '{}' object has no attribute '{}' (or is read-only)",
            self.get_type(),
            attr
        ))
    }

    fn get_item(&self, _key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        Err(format!(
            "TypeError: '{}' object is not subscriptable",
            self.get_type()
        ))
    }
}

pub mod bool;
pub mod bound_method;
pub mod class;
pub mod dict;
pub mod function;
pub mod instance;
pub mod int;
pub mod list;
pub mod native_function;
pub mod none;
pub mod string;
