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

    fn get_item(&self, _key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        Err(format!(
            "TypeError: '{}' object is not subscriptable",
            self.get_type()
        ))
    }
}

pub mod bool;
pub mod dict;
pub mod function;
pub mod int;
pub mod list;
pub mod native_function;
pub mod none;
pub mod string;
