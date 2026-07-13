use crate::objects::PyObject;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    variables: HashMap<String, Rc<dyn PyObject>>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_enclosed(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn set(&mut self, name: String, value: Rc<dyn PyObject>) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Rc<dyn PyObject>> {
        if let Some(val) = self.variables.get(name) {
            Some(Rc::clone(val))
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}
