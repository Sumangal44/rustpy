use super::PyObject;
use std::any::Any;
use std::rc::Rc;

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

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__class__" => {
                Ok(Rc::new(crate::objects::typeobj::PyType::new(&self.exc_type)) as Rc<dyn PyObject>)
            }
            "args" => {
                if let Some(msg) = &self.message {
                    let s = Rc::new(crate::objects::string::PyString::new(msg.clone())) as Rc<dyn PyObject>;
                    Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![s])))
                } else {
                    Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![])))
                }
            }
            "exc_type" => {
                Ok(Rc::new(crate::objects::string::PyString::new(self.exc_type.clone())) as Rc<dyn PyObject>)
            }
            _ => Err(format!("AttributeError: '{}' object has no attribute '{}'", self.exc_type, attr)),
        }
    }
}
