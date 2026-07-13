use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PyFloat {
    pub value: f64,
}

impl PyFloat {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl PyObject for PyFloat {
    fn get_type(&self) -> &'static str {
        "float"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let r = format!("{}", self.value);
        if !r.contains('.') && !r.contains('e') && !r.contains('E') {
            format!("{}.0", r)
        } else {
            r
        }
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        self.value != 0.0
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value + f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value + i.value as f64)))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value - f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value - i.value as f64)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value * f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value * i.value as f64)))
        } else {
            None
        }
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new(self.value / f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(PyFloat::new(self.value / i.value as f64)))
        } else {
            None
        }
    }

    fn floordiv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new((self.value / f.value).floor())))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(PyFloat::new((self.value / i.value as f64).floor())))
        } else {
            None
        }
    }

    fn modulo(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new(self.value % f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(PyFloat::new(self.value % i.value as f64)))
        } else {
            None
        }
    }

    fn pow(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value.powf(f.value))))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value.powi(i.value as i32))))
        } else {
            None
        }
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyFloat::new(-self.value)))
    }

    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyFloat::new(self.value)))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == i.value as f64)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != i.value as f64)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < i.value as f64)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= i.value as f64)))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > i.value as f64)))
        } else {
            None
        }
    }

    fn hash(&self) -> Result<i64, String> {
        Ok(self.value.to_bits() as i64)
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= i.value as f64)))
        } else {
            None
        }
    }
}
