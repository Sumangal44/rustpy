use super::PyObject;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyInt {
    pub value: i64,
}

impl PyInt {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

impl PyObject for PyInt {
    fn get_type(&self) -> &'static str {
        "int"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        self.value.to_string()
    }

    fn str(&self) -> String {
        self.value.to_string()
    }

    fn is_truthy(&self) -> bool {
        self.value != 0
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value + i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 + f.value)))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value - i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 - f.value)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(self.value * i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 * f.value)))
        } else {
            None
        }
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 / i.value as f64)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 / f.value)))
        } else {
            None
        }
    }

    fn floordiv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(PyInt::new(self.value / i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new((self.value as f64 / f.value).floor())))
        } else {
            None
        }
    }

    fn modulo(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value == 0 { return None; }
            Some(Rc::new(PyInt::new(self.value % i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new(self.value as f64 % f.value)))
        } else {
            None
        }
    }

    fn pow(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value < 0 {
                Some(Rc::new(crate::objects::float::PyFloat::new((self.value as f64).powi(i.value as i32))))
            } else {
                Some(Rc::new(PyInt::new(self.value.pow(i.value as u32))))
            }
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new((self.value as f64).powf(f.value))))
        } else {
            None
        }
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(-self.value)))
    }

    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(self.value)))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value as f64 == f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == b_val)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value as f64 != f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != b_val)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new((self.value as f64) < f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < b_val)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new((self.value as f64) <= f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= b_val)))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new((self.value as f64) > f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > b_val)))
        } else {
            None
        }
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new((self.value as f64) >= f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { 1i64 } else { 0i64 };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= b_val)))
        } else {
            None
        }
    }
}
