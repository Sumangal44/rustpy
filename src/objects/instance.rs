use super::PyObject;
use super::bound_method::PyBoundMethod;
use super::class::PyClass;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyInstance {
    pub class: Rc<PyClass>,
    pub attributes: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
}

impl PyInstance {
    pub fn new(class: Rc<PyClass>) -> Self {
        Self {
            class,
            attributes: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl std::fmt::Debug for PyInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyInstance {
    fn get_type(&self) -> &'static str {
        // Technically it should return the class name, but we return a static str for trait compatibility.
        // We can just return "instance" or leak a string if we wanted, but let's stick to "instance" for now.
        "instance"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<{} object at {:p}>", self.class.name, self)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        // 1. Check instance dictionary
        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            return Ok(Rc::clone(val));
        }

        // 2. Check class dictionary
        let class_attrs = self.class.attributes.borrow();
        if let Some(val) = class_attrs.get(attr) {
            // If it's a function, bind it!
            if val.as_any().is::<crate::objects::function::PyFunction>()
                || val
                    .as_any()
                    .is::<crate::objects::native_function::PyNativeFunction>()
            {
                // Return a bound method
                // Wait, we need an Rc<PyInstance> to create a bound method, but we only have &self.
                // This is a classic Rust issue. We can't clone `self` easily because we don't have the Rc wrapper.
                // However, we are implementing `PyObject`.
                // Let's change PyBoundMethod to hold `Rc<RefCell<HashMap<...>>>` ? No, it needs the instance object.
                // Alternatively, we can pass `Rc<dyn PyObject>` to `get_attr`.
                // Let's modify the PyObject trait `get_attr` to take `self_rc: Rc<dyn PyObject>` ? No, that changes the trait.
                // Let's panic for a moment here to think: what if `PyBoundMethod` just holds a clone of PyInstance?
                // `PyInstance` is `Clone` and its internal attributes are in an `Rc<RefCell>`.
                // So cloning `PyInstance` is cheap and maintains the same state!
                let bound = PyBoundMethod::new(self.clone(), Rc::clone(val));
                return Ok(Rc::new(bound));
            }
            return Ok(Rc::clone(val));
        }

        Err(format!(
            "AttributeError: '{}' object has no attribute '{}'",
            self.class.name, attr
        ))
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        self.attributes.borrow_mut().insert(attr.to_string(), value);
        Ok(())
    }
}
