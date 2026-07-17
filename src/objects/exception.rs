use super::PyObject;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

fn build_exception_mro() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();

    m.insert("BaseException", vec!["BaseException"]);
    m.insert("GeneratorExit", vec!["GeneratorExit", "BaseException"]);
    m.insert("KeyboardInterrupt", vec!["KeyboardInterrupt", "BaseException"]);
    m.insert("SystemExit", vec!["SystemExit", "BaseException"]);
    m.insert("Exception", vec!["Exception", "BaseException"]);

    m.insert("ArithmeticError", vec!["ArithmeticError", "Exception", "BaseException"]);
    m.insert("FloatingPointError", vec!["FloatingPointError", "ArithmeticError", "Exception", "BaseException"]);
    m.insert("OverflowError", vec!["OverflowError", "ArithmeticError", "Exception", "BaseException"]);
    m.insert("ZeroDivisionError", vec!["ZeroDivisionError", "ArithmeticError", "Exception", "BaseException"]);

    m.insert("AssertionError", vec!["AssertionError", "Exception", "BaseException"]);
    m.insert("AttributeError", vec!["AttributeError", "Exception", "BaseException"]);
    m.insert("BufferError", vec!["BufferError", "Exception", "BaseException"]);
    m.insert("EOFError", vec!["EOFError", "Exception", "BaseException"]);

    m.insert("ImportError", vec!["ImportError", "Exception", "BaseException"]);
    m.insert("ModuleNotFoundError", vec!["ModuleNotFoundError", "ImportError", "Exception", "BaseException"]);

    m.insert("LookupError", vec!["LookupError", "Exception", "BaseException"]);
    m.insert("IndexError", vec!["IndexError", "LookupError", "Exception", "BaseException"]);
    m.insert("KeyError", vec!["KeyError", "LookupError", "Exception", "BaseException"]);

    m.insert("MemoryError", vec!["MemoryError", "Exception", "BaseException"]);

    m.insert("NameError", vec!["NameError", "Exception", "BaseException"]);
    m.insert("UnboundLocalError", vec!["UnboundLocalError", "NameError", "Exception", "BaseException"]);

    m.insert("OSError", vec!["OSError", "Exception", "BaseException"]);
    m.insert("BlockingIOError", vec!["BlockingIOError", "OSError", "Exception", "BaseException"]);
    m.insert("ChildProcessError", vec!["ChildProcessError", "OSError", "Exception", "BaseException"]);
    m.insert("ConnectionError", vec!["ConnectionError", "OSError", "Exception", "BaseException"]);
    m.insert("BrokenPipeError", vec!["BrokenPipeError", "ConnectionError", "OSError", "Exception", "BaseException"]);
    m.insert("ConnectionAbortedError", vec!["ConnectionAbortedError", "ConnectionError", "OSError", "Exception", "BaseException"]);
    m.insert("ConnectionRefusedError", vec!["ConnectionRefusedError", "ConnectionError", "OSError", "Exception", "BaseException"]);
    m.insert("ConnectionResetError", vec!["ConnectionResetError", "ConnectionError", "OSError", "Exception", "BaseException"]);
    m.insert("FileExistsError", vec!["FileExistsError", "OSError", "Exception", "BaseException"]);
    m.insert("FileNotFoundError", vec!["FileNotFoundError", "OSError", "Exception", "BaseException"]);
    m.insert("InterruptedError", vec!["InterruptedError", "OSError", "Exception", "BaseException"]);
    m.insert("IsADirectoryError", vec!["IsADirectoryError", "OSError", "Exception", "BaseException"]);
    m.insert("NotADirectoryError", vec!["NotADirectoryError", "OSError", "Exception", "BaseException"]);
    m.insert("PermissionError", vec!["PermissionError", "OSError", "Exception", "BaseException"]);
    m.insert("ProcessLookupError", vec!["ProcessLookupError", "OSError", "Exception", "BaseException"]);
    m.insert("TimeoutError", vec!["TimeoutError", "OSError", "Exception", "BaseException"]);

    m.insert("ReferenceError", vec!["ReferenceError", "Exception", "BaseException"]);

    m.insert("RuntimeError", vec!["RuntimeError", "Exception", "BaseException"]);
    m.insert("NotImplementedError", vec!["NotImplementedError", "RuntimeError", "Exception", "BaseException"]);
    m.insert("RecursionError", vec!["RecursionError", "RuntimeError", "Exception", "BaseException"]);

    m.insert("StopIteration", vec!["StopIteration", "Exception", "BaseException"]);
    m.insert("StopAsyncIteration", vec!["StopAsyncIteration", "Exception", "BaseException"]);

    m.insert("SyntaxError", vec!["SyntaxError", "Exception", "BaseException"]);
    m.insert("IndentationError", vec!["IndentationError", "SyntaxError", "Exception", "BaseException"]);
    m.insert("TabError", vec!["TabError", "IndentationError", "SyntaxError", "Exception", "BaseException"]);

    m.insert("SystemError", vec!["SystemError", "Exception", "BaseException"]);
    m.insert("TypeError", vec!["TypeError", "Exception", "BaseException"]);

    m.insert("ValueError", vec!["ValueError", "Exception", "BaseException"]);
    m.insert("UnicodeError", vec!["UnicodeError", "ValueError", "Exception", "BaseException"]);
    m.insert("UnicodeDecodeError", vec!["UnicodeDecodeError", "UnicodeError", "ValueError", "Exception", "BaseException"]);
    m.insert("UnicodeEncodeError", vec!["UnicodeEncodeError", "UnicodeError", "ValueError", "Exception", "BaseException"]);
    m.insert("UnicodeTranslateError", vec!["UnicodeTranslateError", "UnicodeError", "ValueError", "Exception", "BaseException"]);

    m.insert("Warning", vec!["Warning", "Exception", "BaseException"]);
    m.insert("BytesWarning", vec!["BytesWarning", "Warning", "Exception", "BaseException"]);
    m.insert("DeprecationWarning", vec!["DeprecationWarning", "Warning", "Exception", "BaseException"]);
    m.insert("EncodingWarning", vec!["EncodingWarning", "Warning", "Exception", "BaseException"]);
    m.insert("FutureWarning", vec!["FutureWarning", "Warning", "Exception", "BaseException"]);
    m.insert("ImportWarning", vec!["ImportWarning", "Warning", "Exception", "BaseException"]);
    m.insert("PendingDeprecationWarning", vec!["PendingDeprecationWarning", "Warning", "Exception", "BaseException"]);
    m.insert("ResourceWarning", vec!["ResourceWarning", "Warning", "Exception", "BaseException"]);
    m.insert("RuntimeWarning", vec!["RuntimeWarning", "Warning", "Exception", "BaseException"]);
    m.insert("SyntaxWarning", vec!["SyntaxWarning", "Warning", "Exception", "BaseException"]);
    m.insert("UnicodeWarning", vec!["UnicodeWarning", "Warning", "Exception", "BaseException"]);
    m.insert("UserWarning", vec!["UserWarning", "Warning", "Exception", "BaseException"]);

    m
}

static EXCEPTION_MRO: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(build_exception_mro);

pub fn exception_mro(exc_type: &str) -> Vec<&'static str> {
    if let Some(mro) = EXCEPTION_MRO.get(exc_type) {
        mro.clone()
    } else {
        vec!["Exception", "BaseException"]
    }
}

pub fn is_exception_subclass(sub: &str, parent: &str) -> bool {
    if sub == parent {
        return true;
    }
    if parent == "BaseException" {
        return true;
    }
    let mro = exception_mro(sub);
    mro.contains(&parent)
}

#[derive(Debug, Clone)]
pub struct PyException {
    pub exc_type: String,
    pub message: Option<String>,
    pub traceback: Option<String>,
    pub args: Vec<Rc<dyn PyObject>>,
}

impl PyException {
    pub fn new(exc_type: String, message: Option<String>) -> Self {
        Self {
            exc_type,
            message,
            traceback: None,
            args: Vec::new(),
        }
    }

    pub fn with_args(exc_type: String, args: Vec<Rc<dyn PyObject>>) -> Self {
        let message = args.first().map(|a| a.str());
        Self {
            exc_type,
            message,
            traceback: None,
            args,
        }
    }

    pub fn mro(&self) -> Vec<&'static str> {
        exception_mro(&self.exc_type)
    }

    pub fn is_subclass_of(&self, parent: &str) -> bool {
        is_exception_subclass(&self.exc_type, parent)
    }
}

impl PyObject for PyException {
    fn get_type(&self) -> &'static str {
        "Exception"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if let Some(msg) = &self.message {
            if msg.contains('"') || msg.contains('\'') || msg.contains(' ') {
                format!("{}(\"{}\")", self.exc_type, msg)
            } else {
                format!("{}(\"{}\")", self.exc_type, msg)
            }
        } else {
            format!("{}()", self.exc_type)
        }
    }

    fn str(&self) -> String {
        if let Some(msg) = &self.message {
            msg.clone()
        } else {
            "".to_string()
        }
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__class__" => Ok(
                Rc::new(crate::objects::typeobj::PyType::new(&self.exc_type)) as Rc<dyn PyObject>,
            ),
            "args" => {
                if !self.args.is_empty() {
                    Ok(Rc::new(crate::objects::tuple::PyTuple::new(self.args.clone())))
                } else if let Some(msg) = &self.message {
                    let s = Rc::new(crate::objects::string::PyString::new(msg.clone()))
                        as Rc<dyn PyObject>;
                    Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![s])))
                } else {
                    Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![])))
                }
            }
            "__traceback__" => {
                if let Some(tb) = &self.traceback {
                    Ok(Rc::new(crate::objects::string::PyString::new(tb.clone()))
                        as Rc<dyn PyObject>)
                } else {
                    Ok(Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>)
                }
            }
            "with_traceback" => {
                let exc = self.clone();
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only(
                    "with_traceback".to_string(),
                    move |args| {
                        if args.len() != 1 {
                            return Err("TypeError: with_traceback() takes exactly 1 argument".to_string());
                        }
                        let mut new_exc = exc.clone();
                        new_exc.traceback = Some(args[0].str());
                        Ok(Rc::new(new_exc))
                    },
                )) as Rc<dyn PyObject>)
            }
            _ => Err(format!(
                "AttributeError: '{}' object has no attribute '{}'",
                self.exc_type, attr
            )),
        }
    }

    fn set_attr(&self, _attr: &str, _value: Rc<dyn PyObject>) -> Result<(), String> {
        Ok(())
    }

    fn del_attr(&self, name: &str) -> Result<(), String> {
        Err(format!(
            "AttributeError: '{}' object has no attribute '{}'",
            self.exc_type, name
        ))
    }
}
