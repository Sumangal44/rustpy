use super::PyObject;
use crate::objects::string::PyString;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyModule {
    pub name: String,
    pub dict: Rc<RefCell<Vec<(String, Rc<dyn PyObject>)>>>,
}

impl PyModule {
    pub fn new(name: String) -> Self {
        Self {
            name: name.clone(),
            dict: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn set_attr_inner(&self, attr: &str, value: Rc<dyn PyObject>) {
        let mut d = self.dict.borrow_mut();
        for (k, v) in d.iter_mut() {
            if k == attr {
                *v = value;
                return;
            }
        }
        d.push((attr.to_string(), value));
    }
}

impl std::fmt::Debug for PyModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<module '{}'>", self.name)
    }
}

impl PyObject for PyModule {
    fn get_type(&self) -> &'static str {
        "module"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<module '{}'>", self.name)
    }

    fn str(&self) -> String {
        format!("<module '{}'>", self.name)
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let d = self.dict.borrow();
        for (k, v) in d.iter() {
            if k == attr {
                return Ok(Rc::clone(v));
            }
        }
        // Support __name__, __dict__, __file__
        match attr {
            "__name__" => Ok(Rc::new(PyString::new(self.name.clone()))),
            "__dict__" => {
                let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
                for (k, v) in d.iter() {
                    pairs.push((
                        Rc::new(PyString::new(k.clone())) as Rc<dyn PyObject>,
                        Rc::clone(v),
                    ));
                }
                Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
            }
            "__file__" => Ok(Rc::new(crate::objects::none::PyNone)),
            _ => Err(format!(
                "AttributeError: module '{}' has no attribute '{}'",
                self.name, attr
            )),
        }
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        self.set_attr_inner(attr, value);
        Ok(())
    }

    fn del_attr(&self, name: &str) -> Result<(), String> {
        let mut d = self.dict.borrow_mut();
        d.retain(|(k, _)| k != name);
        Ok(())
    }
}
