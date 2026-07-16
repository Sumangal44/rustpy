use crate::objects::PyObject;
use crate::vm::frame::Frame;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct PyCoroutine {
    pub frame: Rc<RefCell<Frame>>,
}

impl PyCoroutine {
    pub fn new(frame: Frame) -> Self {
        Self {
            frame: Rc::new(RefCell::new(frame)),
        }
    }
}

impl Debug for PyCoroutine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<coroutine object>")
    }
}

impl PyObject for PyCoroutine {
    fn get_type(&self) -> &'static str {
        "coroutine"
    }

    fn repr(&self) -> String {
        "<coroutine object>".to_string()
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
        let res = vm.run(&mut self.frame.borrow_mut());
        match res {
            Ok(Some(val)) => {
                if self.frame.borrow().return_value.is_some() {
                    Ok(None)
                } else {
                    Ok(Some(val))
                }
            }
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

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__await__" => {
                let self_clone = Rc::new(Self {
                    frame: Rc::clone(&self.frame),
                }) as Rc<dyn PyObject>;
                Ok(Rc::new(
                    crate::objects::native_function::PyNativeFunction::new_pos_only(
                        "__await__".to_string(),
                        move |_args| {
                            let iter = self_clone.get_iter()?;
                            Ok(iter)
                        },
                    ),
                ) as Rc<dyn PyObject>)
            }
            _ => Err(format!(
                "AttributeError: 'coroutine' object has no attribute '{}'",
                attr
            )),
        }
    }
}
