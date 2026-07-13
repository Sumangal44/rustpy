use super::PyObject;
use std::any::Any;
use std::rc::Rc;

pub type NativeFunc = Rc<dyn Fn(Vec<Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String>>;

#[derive(Clone)]
pub struct PyNativeFunction {
    pub name: String,
    pub func: NativeFunc,
}

impl PyNativeFunction {
    pub fn new<F>(name: String, func: F) -> Self
    where
        F: Fn(Vec<Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String> + 'static,
    {
        Self {
            name,
            func: Rc::new(func),
        }
    }
}

impl std::fmt::Debug for PyNativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl PyObject for PyNativeFunction {
    fn get_type(&self) -> &'static str {
        "builtin_function_or_method"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<built-in function {}>", self.name)
    }

    fn is_truthy(&self) -> bool {
        true
    }
}
