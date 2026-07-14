use super::PyObject;
use super::bound_method::PyBoundMethod;
use super::class::PyClass;
use super::classmethod::PyClassMethod;
use super::int::PyInt;
use super::native_function::PyNativeFunction;
use super::property::PyProperty;
use super::staticmethod::PyStaticMethod;
use super::string::PyString;
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

    fn bind_function(&self, func: Rc<dyn PyObject>) -> Rc<dyn PyObject> {
        let bound = PyBoundMethod::new(self.clone(), Rc::clone(&func));
        Rc::new(bound)
    }

    fn resolve_class_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let val = self.class.get_attr(attr)?;

        if let Some(prop) = val.as_any().downcast_ref::<PyProperty>() {
            return prop.call_getter(Rc::new(self.clone()) as Rc<dyn PyObject>);
        }
        if let Some(_sm) = val.as_any().downcast_ref::<PyStaticMethod>() {
            let sm = val.as_any().downcast_ref::<PyStaticMethod>().unwrap();
            return Ok(Rc::clone(&sm.func));
        }
        if let Some(_cm) = val.as_any().downcast_ref::<PyClassMethod>() {
            let cm = val.as_any().downcast_ref::<PyClassMethod>().unwrap();
            return Ok(self.bind_function(Rc::clone(&cm.func)));
        }

        if val.as_any().is::<crate::objects::function::PyFunction>()
            || val.as_any().is::<crate::objects::native_function::PyNativeFunction>()
        {
            return Ok(self.bind_function(val));
        }

        Ok(val)
    }

    pub fn call_dunder(&self, name: &str, args: Vec<Rc<dyn PyObject>>) -> Result<Rc<dyn PyObject>, String> {
        let method = self.get_attr(name)?;
        if let Some(bound) = method.as_any().downcast_ref::<PyBoundMethod>() {
            if let Some(native) = bound.func.as_any().downcast_ref::<PyNativeFunction>() {
                let mut all_args = vec![Rc::new(self.clone()) as Rc<dyn PyObject>];
                all_args.extend(args);
                return (native.func)(all_args, std::collections::HashMap::new());
            }
            return Err(format!("NotImplementedError: calling {} on user-defined function not supported", name));
        } else if let Some(native) = method.as_any().downcast_ref::<PyNativeFunction>() {
            return (native.func)(args, std::collections::HashMap::new());
        }
        Err(format!("TypeError: '{}' object is not callable", method.get_type()))
    }

    pub fn len(&self) -> Result<usize, String> {
        let result = self.call_dunder("__len__", vec![])?;
        if let Some(i) = result.as_any().downcast_ref::<PyInt>() {
            Ok(i.to_usize().unwrap_or(0))
        } else {
            Err("TypeError: __len__ must return an int".to_string())
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
        "instance"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if let Ok(result) = self.call_dunder("__repr__", vec![]) {
            return result.str();
        }
        format!("<{} object at {:p}>", self.class.name, self)
    }

    fn str(&self) -> String {
        if let Ok(result) = self.call_dunder("__str__", vec![]) {
            return result.str();
        }
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        if let Ok(result) = self.call_dunder("__bool__", vec![]) {
            return result.is_truthy();
        }
        if let Ok(result) = self.call_dunder("__len__", vec![]) {
            if let Some(i) = result.as_any().downcast_ref::<PyInt>() {
                return i.as_i64().unwrap_or(0) != 0;
            }
        }
        true
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        if let Ok(result) = self.call_dunder("__contains__", vec![Rc::clone(&other)]) {
            Ok(result.is_truthy())
        } else {
            let iter = self.get_iter()?;
            while let Some(item) = iter.get_next()? {
                if let Some(eq_result) = item.eq(Rc::clone(&other)) {
                    if eq_result.is_truthy() {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        if attr == "__dict__" {
            let attrs = self.attributes.borrow();
            let mut pairs = Vec::new();
            for (k, v) in attrs.iter() {
                pairs.push((Rc::new(PyString::new(k.clone())) as Rc<dyn PyObject>, Rc::clone(v)));
            }
            return Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)));
        }

        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            return Ok(Rc::clone(val));
        }

        // Check class and its MRO with descriptor protocol
        self.resolve_class_attr(attr)
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        self.attributes.borrow_mut().insert(attr.to_string(), value);
        Ok(())
    }

    fn del_attr(&self, name: &str) -> Result<(), String> {
        self.attributes.borrow_mut().remove(name);
        Ok(())
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        if let Ok(iter_method) = self.call_dunder("__iter__", vec![]) {
            return Ok(iter_method);
        }
        Err(format!("TypeError: '{}' object is not iterable", self.class.name))
    }
}
