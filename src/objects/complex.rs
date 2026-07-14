use super::PyObject;
use num_traits::Zero;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct PyComplex {
    pub real: f64,
    pub imag: f64,
}

impl PyComplex {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }
}

fn fmt_complex_float(v: f64) -> String {
    format!("{}", v)
}

impl PyObject for PyComplex {
    fn get_type(&self) -> &'static str {
        "complex"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let real_str = fmt_complex_float(self.real);
        let imag_str = fmt_complex_float(self.imag);
        if self.real == 0.0 && self.real.signum() >= 0.0 {
            format!("{}j", imag_str)
        } else if self.imag >= 0.0 {
            format!("({}+{}j)", real_str, imag_str)
        } else {
            format!("({}{}j)", real_str, imag_str)
        }
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        self.real != 0.0 || self.imag != 0.0
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(c) = other.as_any().downcast_ref::<PyComplex>() {
            Some(Rc::new(PyComplex::new(self.real + c.real, self.imag + c.imag)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(PyComplex::new(self.real + f.value, self.imag)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let val = i.as_i64().unwrap_or(0) as f64;
            Some(Rc::new(PyComplex::new(self.real + val, self.imag)))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(c) = other.as_any().downcast_ref::<PyComplex>() {
            Some(Rc::new(PyComplex::new(self.real - c.real, self.imag - c.imag)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(PyComplex::new(self.real - f.value, self.imag)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let val = i.as_i64().unwrap_or(0) as f64;
            Some(Rc::new(PyComplex::new(self.real - val, self.imag)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(c) = other.as_any().downcast_ref::<PyComplex>() {
            let real = self.real * c.real - self.imag * c.imag;
            let imag = self.real * c.imag + self.imag * c.real;
            Some(Rc::new(PyComplex::new(real, imag)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(PyComplex::new(self.real * f.value, self.imag * f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let val = i.as_i64().unwrap_or(0) as f64;
            Some(Rc::new(PyComplex::new(self.real * val, self.imag * val)))
        } else {
            None
        }
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(c) = other.as_any().downcast_ref::<PyComplex>() {
            let denom = c.real * c.real + c.imag * c.imag;
            if denom == 0.0 { return None; }
            let real = (self.real * c.real + self.imag * c.imag) / denom;
            let imag = (self.imag * c.real - self.real * c.imag) / denom;
            Some(Rc::new(PyComplex::new(real, imag)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyComplex::new(self.real / f.value, self.imag / f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value.is_zero() { return None; }
            let val = i.as_i64().unwrap_or(1) as f64;
            Some(Rc::new(PyComplex::new(self.real / val, self.imag / val)))
        } else {
            None
        }
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyComplex::new(-self.real, -self.imag)))
    }

    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyComplex::new(self.real, self.imag)))
    }

    fn abs_op(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(crate::objects::float::PyFloat::new((self.real * self.real + self.imag * self.imag).sqrt())))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(c) = other.as_any().downcast_ref::<PyComplex>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.real == c.real && self.imag == c.imag)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.real == f.value && self.imag == 0.0)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let val = i.as_i64().unwrap_or(0) as f64;
            Some(Rc::new(crate::objects::bool::PyBool::new(self.real == val && self.imag == 0.0)))
        } else {
            None
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "real" => Ok(Rc::new(crate::objects::float::PyFloat::new(self.real))),
            "imag" => Ok(Rc::new(crate::objects::float::PyFloat::new(self.imag))),
            _ => Err(format!(
                "AttributeError: 'complex' object has no attribute '{}'",
                attr
            )),
        }
    }
}
