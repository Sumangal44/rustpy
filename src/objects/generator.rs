use crate::objects::PyObject;
use crate::vm::frame::Frame;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct PyGenerator {
    pub frame: Rc<RefCell<Frame>>,
}

impl PyGenerator {
    pub fn new(frame: Frame) -> Self {
        Self {
            frame: Rc::new(RefCell::new(frame)),
        }
    }
}

impl Debug for PyGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<generator object>")
    }
}

impl PyObject for PyGenerator {
    fn get_type(&self) -> &'static str {
        "generator"
    }

    fn repr(&self) -> String {
        "<generator object>".to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(Self {
            frame: Rc::clone(&self.frame),
        }))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut vm = crate::vm::VirtualMachine::new();
        match vm.run(&mut self.frame.borrow_mut()) {
            Ok(Some(val)) => Ok(Some(val)),
            Ok(None) => Ok(None),
            Err(e) => {
                if e == "StopIteration" || e.contains("StopIteration") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}
