use super::PyObject;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

pub type NativeFunc = Rc<dyn Fn(Vec<Rc<dyn PyObject>>, HashMap<String, Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String>>;

#[derive(Clone)]
pub struct PyNativeFunction {
    pub name: String,
    pub func: NativeFunc,
}

impl PyNativeFunction {
    pub fn new<F>(name: String, func: F) -> Self
    where
        F: Fn(Vec<Rc<dyn PyObject>>, HashMap<String, Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String> + 'static,
    {
        Self {
            name,
            func: Rc::new(func),
        }
    }

    pub fn new_pos_only<F>(name: String, func: F) -> Self
    where
        F: Fn(Vec<Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String> + 'static,
    {
        Self {
            name,
            func: Rc::new(move |args, _kwargs| func(args)),
        }
    }
}

impl std::fmt::Debug for PyNativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl PyObject for PyNativeFunction {
    fn get_type(&self) -> &'static str {
        "builtin_function_or_method"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<built-in function {}>", self.name)
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        if self.name == "dict" && attr == "fromkeys" {
            let self_clone = self.clone();
            return Ok(Rc::new(PyNativeFunction::new_pos_only("fromkeys".to_string(), move |args| {
                if args.is_empty() {
                    return Err("TypeError: fromkeys() takes at least 1 argument (0 given)".to_string());
                }
                let iterable = Rc::clone(&args[0]);
                let value = if args.len() > 1 { Rc::clone(&args[1]) } else { Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject> };
                let it = iterable.get_iter()?;
                let mut pairs = Vec::new();
                while let Some(k) = it.get_next()? {
                    pairs.push((k, Rc::clone(&value)));
                }
                Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
            })) as Rc<dyn PyObject>);
        }
        Err(format!(
            "AttributeError: 'builtin_function_or_method' object has no attribute '{}'",
            attr
        ))
    }
}
