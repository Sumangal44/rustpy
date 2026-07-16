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
    fn truediv(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn floordiv(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn modulo(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn pow(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }

    fn matmul(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn rmatmul(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn bitand(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn bitor(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn bitxor(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn lshift(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn rshift(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn invert(&self) -> Option<Rc<dyn PyObject>> {
        None
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        None
    }

    fn abs_op(&self) -> Option<Rc<dyn PyObject>> {
        None
    }

    fn eq(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn ne(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn lt(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn le(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn gt(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        None
    }
    fn ge(&self, _other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
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

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        let iter = self.get_iter()?;
        while let Some(item) = iter.get_next()? {
            if let Some(eq_result) = item.eq(Rc::clone(&other)) {
                if eq_result.is_truthy() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
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

    fn del_attr(&self, name: &str) -> Result<(), String> {
        Err(format!(
            "AttributeError: '{}' object has no attribute '{}'",
            self.get_type(),
            name
        ))
    }

    fn get_item(&self, _key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        Err(format!(
            "TypeError: '{}' object is not subscriptable",
            self.get_type()
        ))
    }

    fn del_item(&self, _key: Rc<dyn PyObject>) -> Result<(), String> {
        Err(format!(
            "TypeError: '{}' object does not support item deletion",
            self.get_type()
        ))
    }

    fn hash(&self) -> Result<i64, String> {
        Err(format!("TypeError: unhashable type: '{}'", self.get_type()))
    }

}

pub mod bool;
pub mod bound_method;
pub mod bytearray;
pub mod bytes;
pub mod class;
pub mod classmethod;
pub mod complex;
pub mod constants;
pub mod coroutine;
pub mod dict;
pub mod exception;
pub mod file;
pub mod float;
pub mod function;
pub mod generator;
pub mod instance;
pub mod int;
pub mod list;
pub mod map;
pub mod memoryview;
pub mod module;
pub mod native_function;
pub mod none;
pub mod property;
pub mod range;
pub mod set;
pub mod slice;
pub mod staticmethod;
pub mod string;
pub mod tuple;
pub mod typeobj;
