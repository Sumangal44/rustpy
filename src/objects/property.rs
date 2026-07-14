use super::PyObject;
use super::native_function::PyNativeFunction;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyProperty {
    pub fget: Option<Rc<dyn PyObject>>,
    #[allow(dead_code)]
    pub fset: Option<Rc<dyn PyObject>>,
    #[allow(dead_code)]
    pub fdel: Option<Rc<dyn PyObject>>,
}

impl PyProperty {
    pub fn new(
        fget: Option<Rc<dyn PyObject>>,
        fset: Option<Rc<dyn PyObject>>,
        fdel: Option<Rc<dyn PyObject>>,
    ) -> Self {
        Self { fget, fset, fdel }
    }

    pub fn call_getter(&self, instance: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(fget) = &self.fget {
            if let Some(native) = fget.as_any().downcast_ref::<PyNativeFunction>() {
                return (native.func)(vec![instance], std::collections::HashMap::new());
            }
            return Err("TypeError: property getter must be a native callable or use the VM".to_string());
        }
        Err("AttributeError: unreadable attribute".to_string())
    }
}

impl std::fmt::Debug for PyProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<property object>")
    }
}

impl PyObject for PyProperty {
    fn get_type(&self) -> &'static str {
        "property"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        "<property object>".to_string()
    }

    fn is_truthy(&self) -> bool {
        true
    }
}
