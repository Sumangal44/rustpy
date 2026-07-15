use crate::objects::PyObject;
use crate::objects::none::PyNone;
use crate::vm::frame::Frame;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct PyGenerator {
    pub frame: Rc<RefCell<Frame>>,
    pub started: Rc<RefCell<bool>>,
    pub closed: Rc<RefCell<bool>>,
}

impl PyGenerator {
    pub fn new(frame: Frame) -> Self {
        Self {
            frame: Rc::new(RefCell::new(frame)),
            started: Rc::new(RefCell::new(false)),
            closed: Rc::new(RefCell::new(false)),
        }
    }

    pub fn send(&self, value: Rc<dyn PyObject>) -> Result<Option<Rc<dyn PyObject>>, String> {
        if *self.closed.borrow() {
            return Err("StopIteration".to_string());
        }
        if !*self.started.borrow() {
            *self.started.borrow_mut() = true;
            if value.get_type() != "NoneType" {
                return Err("TypeError: can't send non-None value to a just-started generator".to_string());
            }
            // First call to send(None) just starts the generator
            return self.resume_inner(None);
        }
        self.resume_inner(Some(value))
    }

    fn resume_inner(&self, send_value: Option<Rc<dyn PyObject>>) -> Result<Option<Rc<dyn PyObject>>, String> {
        if *self.closed.borrow() {
            return Ok(None);
        }
        *self.started.borrow_mut() = true;
        let mut vm = crate::vm::VirtualMachine::new();
        let (res, has_returned) = {
            let mut frame = self.frame.borrow_mut();
            if frame.ip > 0 && !frame.stack.is_empty() {
                frame.pop()?;
                frame.push(send_value.unwrap_or_else(|| Rc::new(PyNone::new())));
            }
            let run_res = vm.run(&mut frame);
            let ret = frame.return_value.is_some();
            (run_res, ret)
        };
        match res {
            Ok(Some(val)) => {
                if has_returned {
                    *self.closed.borrow_mut() = true;
                    Ok(None)
                } else {
                    Ok(Some(val))
                }
            }
            Ok(None) => {
                *self.closed.borrow_mut() = true;
                Ok(None)
            }
            Err(e) => {
                *self.closed.borrow_mut() = true;
                if e == "StopIteration" || e.contains("StopIteration") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
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

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let self_rc = Rc::new(PyGenerator {
            frame: Rc::clone(&self.frame),
            started: Rc::clone(&self.started),
            closed: Rc::clone(&self.closed),
        }) as Rc<dyn PyObject>;
        match attr {
            "__next__" => {
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only("__next__".to_string(), {
                    let gen_rc = self_rc.clone();
                    move |_args| {
                        let gen_obj = gen_rc.as_any().downcast_ref::<PyGenerator>().unwrap();
                        match gen_obj.resume_inner(None) {
                            Ok(Some(val)) => Ok(val),
                            Ok(None) => Err("StopIteration".to_string()),
                            Err(e) => Err(e),
                        }
                    }
                })) as Rc<dyn PyObject>)
            }
            "send" => {
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only("send".to_string(), {
                    let gen_rc = self_rc.clone();
                    move |args| {
                        if args.is_empty() {
                            return Err("TypeError: send() takes exactly one argument".to_string());
                        }
                        let gen_obj = gen_rc.as_any().downcast_ref::<PyGenerator>().unwrap();
                        match gen_obj.send(Rc::clone(&args[0])) {
                            Ok(Some(val)) => Ok(val),
                            Ok(None) => Err("StopIteration".to_string()),
                            Err(e) => Err(e),
                        }
                    }
                })) as Rc<dyn PyObject>)
            }
            "throw" => {
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only("throw".to_string(), {
                    let gen_rc = self_rc.clone();
                    move |args| {
                        if args.is_empty() {
                            return Err("TypeError: throw() requires at least 1 argument".to_string());
                        }
                        let exc_obj = Rc::clone(&args[0]);
                        let gen_obj = gen_rc.as_any().downcast_ref::<PyGenerator>().unwrap();
                        // Inject the exception into the frame and resume
                        {
                            let mut frame = gen_obj.frame.borrow_mut();
                            frame.pending_exception = Some(exc_obj);
                        }
                        match gen_obj.resume_inner(None) {
                            Ok(Some(val)) => Ok(val),
                            Ok(None) => Err("StopIteration".to_string()),
                            Err(e) => Err(e),
                        }
                    }
                })) as Rc<dyn PyObject>)
            }
            "close" => {
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only("close".to_string(), {
                    let gen_rc = self_rc.clone();
                    move |_args| {
                        let gen_obj = gen_rc.as_any().downcast_ref::<PyGenerator>().unwrap();
                        // If generator is already closed, nothing to do
                        if *gen_obj.closed.borrow() {
                            return Ok(Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>);
                        }
                        // If generator hasn't started, mark as closed and return None
                        if !*gen_obj.started.borrow() {
                            *gen_obj.closed.borrow_mut() = true;
                            return Ok(Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>);
                        }
                        // Inject GeneratorExit as a string error that resume_inner will propagate
                        {
                            let mut frame = gen_obj.frame.borrow_mut();
                            frame.pending_exception = Some(
                                Rc::new(crate::objects::string::PyString::new("GeneratorExit".to_string()))
                                    as Rc<dyn PyObject>
                            );
                        }
                        match gen_obj.resume_inner(None) {
                            Ok(None) => Ok(Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>),
                            Ok(Some(_)) => Err("RuntimeError: generator ignored GeneratorExit".to_string()),
                            Err(ref e) if e.contains("GeneratorExit") || e.contains("StopIteration") => {
                                Ok(Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>)
                            }
                            Err(e) => Err(e),
                        }
                    }
                })) as Rc<dyn PyObject>)
            }
            _ => Err(format!("AttributeError: 'generator' object has no attribute '{}'", attr)),
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(Self {
            frame: Rc::clone(&self.frame),
            started: Rc::clone(&self.started),
            closed: Rc::clone(&self.closed),
        }))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        self.resume_inner(None)
    }
}
