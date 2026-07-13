use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyDict {
    // We restrict keys to strings for simplicity in Phase 9
    pub entries: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
}

impl PyDict {
    pub fn new(entries: HashMap<String, Rc<dyn PyObject>>) -> Self {
        Self {
            entries: Rc::new(RefCell::new(entries)),
        }
    }
}

impl std::fmt::Debug for PyDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyDict {
    fn get_type(&self) -> &'static str {
        "dict"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let entries = self.entries.borrow();
        let mut out = String::new();
        out.push('{');
        for (i, (k, v)) in entries.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("'{}': {}", k, v.repr()));
        }
        out.push('}');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.entries.borrow().is_empty()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(str_key) = key
            .as_any()
            .downcast_ref::<crate::objects::string::PyString>()
        {
            let entries = self.entries.borrow();
            if let Some(val) = entries.get(&str_key.value) {
                Ok(Rc::clone(val))
            } else {
                Err(format!("KeyError: '{}'", str_key.value))
            }
        } else {
            Err(format!(
                "TypeError: unhashable type: '{}' (Only strings supported as keys currently)",
                key.get_type()
            ))
        }
    }
}
