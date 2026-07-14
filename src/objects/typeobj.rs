use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct PyType {
    pub name: String,
}

impl PyType {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl PyObject for PyType {
    fn get_type(&self) -> &'static str {
        "type"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<class '{}'>", self.name)
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__name__" => Ok(Rc::new(crate::objects::string::PyString::new(self.name.clone()))),
            "__module__" => Ok(Rc::new(crate::objects::string::PyString::new("builtins".to_string()))),
            _ => Err(format!("AttributeError: type object '{}' has no attribute '{}'", self.name, attr)),
        }
    }
}
