use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyClass {
    pub name: String,
    pub attributes: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
}

impl PyClass {
    pub fn new(name: String, attributes: HashMap<String, Rc<dyn PyObject>>) -> Self {
        Self {
            name,
            attributes: Rc::new(RefCell::new(attributes)),
        }
    }
}

impl std::fmt::Debug for PyClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyClass {
    fn get_type(&self) -> &'static str {
        "type"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<class '{}'>", self.name)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            Ok(Rc::clone(val))
        } else {
            Err(format!(
                "AttributeError: type object '{}' has no attribute '{}'",
                self.name, attr
            ))
        }
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        self.attributes.borrow_mut().insert(attr.to_string(), value);
        Ok(())
    }
}
