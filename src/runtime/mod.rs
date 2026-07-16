use crate::objects::PyObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    variables: HashMap<String, Rc<dyn PyObject>>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: HashMap::new(),
            parent: None,
        }))
    }

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn set(&mut self, name: String, value: Rc<dyn PyObject>) {
        self.variables.insert(name, value);
    }

    pub fn get_all_locals(&self) -> HashMap<String, Rc<dyn PyObject>> {
        self.variables.clone()
    }

    pub fn get(&self, name: &str) -> Option<Rc<dyn PyObject>> {
        if let Some(val) = self.variables.get(name) {
            Some(Rc::clone(val))
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        if self.variables.remove(name).is_some() {
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().remove(name)
        } else {
            false
        }
    }

    pub fn set_nonlocal(&mut self, name: String, value: Rc<dyn PyObject>) {
        if let Some(parent) = &self.parent {
            if parent.borrow().variables.contains_key(&name) {
                parent.borrow_mut().variables.insert(name, value);
            } else {
                parent.borrow_mut().set_nonlocal(name, value);
            }
        } else {
            self.variables.insert(name, value);
        }
    }

    pub fn set_root(&mut self, name: String, value: Rc<dyn PyObject>) {
        if let Some(parent) = &self.parent {
            if parent.borrow().parent.is_none() {
                self.variables.insert(name, value);
            } else {
                parent.borrow_mut().set_root(name, value);
            }
        } else {
            self.variables.insert(name, value);
        }
    }

    pub fn get_root(&self, name: &str) -> Option<Rc<dyn PyObject>> {
        if let Some(parent) = &self.parent {
            if parent.borrow().parent.is_none() {
                if let Some(val) = self.variables.get(name) {
                    Some(Rc::clone(val))
                } else {
                    parent.borrow().variables.get(name).map(|v| Rc::clone(v))
                }
            } else {
                parent.borrow().get_root(name)
            }
        } else {
            self.variables.get(name).map(|v| Rc::clone(v))
        }
    }
}
