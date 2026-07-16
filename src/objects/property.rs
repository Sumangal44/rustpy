use super::PyObject;
use super::function::PyFunction;
use super::native_function::PyNativeFunction;
use crate::runtime::Environment;
use crate::vm::VirtualMachine;
use crate::vm::frame::Frame;
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
            if let Some(func) = fget.as_any().downcast_ref::<PyFunction>() {
                let code = func.code.clone();
                let env = Environment::new_enclosed(func.env.clone());
                env.borrow_mut().set("self".to_string(), instance);
                let mut vm = VirtualMachine::new();
                let mut frame = Frame::new(code, env);
                match vm.run(&mut frame) {
                    Ok(val) => {
                        return Ok(val.unwrap_or_else(|| Rc::new(crate::objects::none::PyNone)));
                    }
                    Err(e) => return Err(e),
                }
            }
            return Err("TypeError: property getter must be callable".to_string());
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
