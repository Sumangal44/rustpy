use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyBool {
    pub value: bool,
}

impl PyBool {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl PyObject for PyBool {
    fn get_type(&self) -> &'static str {
        "bool"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if self.value {
            "True".to_string()
        } else {
            "False".to_string()
        }
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        self.value
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value == b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            let other_i = i.as_i64().unwrap_or(0);
            Some(Rc::new(PyBool::new(self_int == other_i)))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f == f.value)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value != b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            let other_i = i.as_i64().unwrap_or(0);
            Some(Rc::new(PyBool::new(self_int != other_i)))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f != f.value)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value < b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            Some(Rc::new(PyBool::new(self_int < i.as_i64().unwrap_or(0))))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f < f.value)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value <= b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            Some(Rc::new(PyBool::new(self_int <= i.as_i64().unwrap_or(0))))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f <= f.value)))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value > b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            Some(Rc::new(PyBool::new(self_int > i.as_i64().unwrap_or(0))))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f > f.value)))
        } else {
            None
        }
    }

    fn hash(&self) -> Result<i64, String> {
        Ok(if self.value { 1 } else { 0 })
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBool>() {
            Some(Rc::new(PyBool::new(self.value >= b.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let self_int = if self.value { 1i64 } else { 0i64 };
            Some(Rc::new(PyBool::new(self_int >= i.as_i64().unwrap_or(0))))
        } else if let Some(f) = other
            .as_any()
            .downcast_ref::<crate::objects::float::PyFloat>()
        {
            let self_f = if self.value { 1.0 } else { 0.0 };
            Some(Rc::new(PyBool::new(self_f >= f.value)))
        } else {
            None
        }
    }
}
