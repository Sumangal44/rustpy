use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyList {
    pub elements: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
}

impl PyList {
    pub fn new(elements: Vec<Rc<dyn PyObject>>) -> Self {
        Self {
            elements: Rc::new(RefCell::new(elements)),
        }
    }
}

impl std::fmt::Debug for PyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyList {
    fn get_type(&self) -> &'static str {
        "list"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let elements = self.elements.borrow();
        let mut out = String::new();
        out.push('[');
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&elem.repr());
        }
        out.push(']');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.elements.borrow().is_empty()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx_obj) = key.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let elements = self.elements.borrow();
            let mut idx = idx_obj.value;
            if idx < 0 {
                idx += elements.len() as i64;
            }
            if idx >= 0 && (idx as usize) < elements.len() {
                Ok(Rc::clone(&elements[idx as usize]))
            } else {
                Err("IndexError: list index out of range".to_string())
            }
        } else {
            Err(format!(
                "TypeError: list indices must be integers, not {}",
                key.get_type()
            ))
        }
    }
}
