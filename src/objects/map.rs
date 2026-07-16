use super::PyObject;
use super::native_function::PyNativeFunction;
use std::any::Any;
use std::rc::Rc;

#[allow(dead_code)]
pub struct PyMap {
    func: Rc<dyn PyObject>,
    iter: Rc<dyn PyObject>,
}

impl PyMap {
    #[allow(dead_code)]
    pub fn new(func: Rc<dyn PyObject>, iter: Rc<dyn PyObject>) -> Self {
        Self { func, iter }
    }
}

impl std::fmt::Debug for PyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<map object at {:p}>", self)
    }
}

impl PyObject for PyMap {
    fn get_type(&self) -> &'static str {
        "map"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<map object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        // map objects are their own iterators; caller should use get_next
        Ok(Rc::new(PyMap {
            func: Rc::clone(&self.func),
            iter: Rc::clone(&self.iter),
        }))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        if let Some(item) = self.iter.get_next()? {
            // Call func with item
            let result = if let Some(native) = self.func.as_any().downcast_ref::<PyNativeFunction>() {
                (native.func)(vec![item], std::collections::HashMap::new())?
            } else {
                return Err("TypeError: map() argument must be callable".to_string());
            };
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
