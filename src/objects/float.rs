use super::PyObject;
use super::native_function::PyNativeFunction;
use num_traits::{ToPrimitive, Zero};
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
            Some(Rc::new(PyFloat::new(self.value + i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            Some(Rc::new(crate::objects::complex::PyComplex::new(self.value + c.real, c.imag)))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value - f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value - i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            Some(Rc::new(crate::objects::complex::PyComplex::new(self.value - c.real, -c.imag)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value * f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(PyFloat::new(self.value * i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            Some(Rc::new(crate::objects::complex::PyComplex::new(self.value * c.real, self.value * c.imag)))
        } else {
            None
        }
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new(self.value / f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value.is_zero() { return None; }
            Some(Rc::new(PyFloat::new(self.value / i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let denom = c.real * c.real + c.imag * c.imag;
            if denom == 0.0 { return None; }
            let real = self.value * c.real / denom;
            let imag = -self.value * c.imag / denom;
            Some(Rc::new(crate::objects::complex::PyComplex::new(real, imag)))
        } else {
            None
        }
    }

    fn floordiv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new((self.value / f.value).floor())))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value.is_zero() { return None; }
            Some(Rc::new(PyFloat::new((self.value / i.value.to_f64().unwrap_or(0.0)).floor())))
        } else {
            None
        }
    }

    fn modulo(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(PyFloat::new(self.value % f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            if i.value.is_zero() { return None; }
            Some(Rc::new(PyFloat::new(self.value % i.value.to_f64().unwrap_or(0.0))))
        } else {
            None
        }
    }

    fn pow(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(PyFloat::new(self.value.powf(f.value))))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let exp = i.as_i64().unwrap_or(0) as i32;
            Some(Rc::new(PyFloat::new(self.value.powi(exp))))
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

    fn abs_op(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyFloat::new(self.value.abs())))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == c.real && c.imag == 0.0)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != i.value.to_f64().unwrap_or(0.0))))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != c.real || c.imag != 0.0)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < i.value.to_f64().unwrap_or(0.0))))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= i.value.to_f64().unwrap_or(0.0))))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > i.value.to_f64().unwrap_or(0.0))))
        } else {
            None
        }
    }

    fn hash(&self) -> Result<i64, String> {
        Ok(self.value.to_bits() as i64)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__format__" => {
                let val = self.value;
                Ok(Rc::new(PyNativeFunction::new_pos_only("__format__".to_string(), move |args| {
                    let spec = if args.is_empty() { String::new() } else { args[0].str() };
                    if spec.is_empty() {
                        let r = format!("{}", val);
                        let final_r = if !r.contains('.') && !r.contains('e') && !r.contains('E') {
                            format!("{}.0", r)
                        } else { r };
                        return Ok(Rc::new(crate::objects::string::PyString::new(final_r)));
                    }

                    // Extract type character if any (last char in spec)
                    let mut type_char = 'g';
                    if let Some(last_char) = spec.chars().last() {
                        if last_char.is_alphabetic() || last_char == '%' {
                            type_char = last_char;
                        }
                    }

                    // Find if precision (.num) is present in spec
                    let mut precision = None;
                    if let Some(dot_idx) = spec.find('.') {
                        // Extract digits after dot until type_char
                        let end_idx = if spec.chars().last().map(|c| c.is_alphabetic() || c == '%').unwrap_or(false) {
                            spec.len() - 1
                        } else {
                            spec.len()
                        };
                        if end_idx > dot_idx + 1 {
                            if let Ok(prec) = spec[dot_idx + 1..end_idx].parse::<usize>() {
                                precision = Some(prec);
                            }
                        }
                    }

                    let raw_str = match type_char {
                        'f' | 'F' => {
                            if let Some(prec) = precision {
                                format!("{:.*}", prec, val)
                            } else {
                                format!("{:.6}", val) // default precision in python is 6 for f
                            }
                        }
                        'e' => {
                            if let Some(prec) = precision {
                                format!("{:.*e}", prec, val)
                            } else {
                                format!("{:e}", val)
                            }
                        }
                        'E' => {
                            if let Some(prec) = precision {
                                format!("{:.*E}", prec, val)
                            } else {
                                format!("{:E}", val)
                            }
                        }
                        '%' => {
                            if let Some(prec) = precision {
                                format!("{:.*}%", prec, val * 100.0)
                            } else {
                                format!("{}%", val * 100.0)
                            }
                        }
                        _ => {
                            // Default formatting
                            let r = format!("{}", val);
                            if !r.contains('.') && !r.contains('e') && !r.contains('E') {
                                format!("{}.0", r)
                            } else { r }
                        }
                    };

                    match crate::objects::string::format_align_width(&raw_str, &spec, '>') {
                        Ok(res) => Ok(Rc::new(crate::objects::string::PyString::new(res))),
                        Err(e) => Err(e),
                    }
                })))
            }
            _ => Err(format!("AttributeError: 'float' object has no attribute '{}'", attr)),
        }
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(f) = other.as_any().downcast_ref::<PyFloat>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= f.value)))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= i.value.to_f64().unwrap_or(0.0))))
        } else {
            None
        }
    }
}
