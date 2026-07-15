use super::PyObject;
use super::native_function::PyNativeFunction;
use num_bigint::BigInt;
use num_traits::{Zero, One, ToPrimitive, Signed};
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyInt {
    pub value: BigInt,
}

impl PyInt {
    pub fn new(value: BigInt) -> Self {
        Self { value }
    }

    pub fn from_i64(value: i64) -> Self {
        Self { value: BigInt::from(value) }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.value.to_i64()
    }

    pub fn to_usize(&self) -> Option<usize> {
        self.value.to_usize()
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
        !self.value.is_zero()
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value + &i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(
                self.value.to_f64().unwrap_or(0.0) + f.value,
            )))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let val = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::complex::PyComplex::new(val + c.real, c.imag)))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value - &i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(
                self.value.to_f64().unwrap_or(0.0) - f.value,
            )))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let val = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::complex::PyComplex::new(val - c.real, -c.imag)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value * &i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            Some(Rc::new(crate::objects::float::PyFloat::new(
                self.value.to_f64().unwrap_or(0.0) * f.value,
            )))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let val = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::complex::PyComplex::new(val * c.real, val * c.imag)))
        } else {
            None
        }
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value.is_zero() { return None; }
            let a = self.value.to_f64().unwrap_or(0.0);
            let b = i.value.to_f64().unwrap_or(1.0);
            Some(Rc::new(crate::objects::float::PyFloat::new(a / b)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new(
                self.value.to_f64().unwrap_or(0.0) / f.value,
            )))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let denom = c.real * c.real + c.imag * c.imag;
            if denom == 0.0 { return None; }
            let a = self.value.to_f64().unwrap_or(0.0);
            let real = a * c.real / denom;
            let imag = -a * c.imag / denom;
            Some(Rc::new(crate::objects::complex::PyComplex::new(real, imag)))
        } else {
            None
        }
    }

    fn floordiv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value.is_zero() { return None; }
            Some(Rc::new(PyInt::new(&self.value / &i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            let a = self.value.to_f64().unwrap_or(0.0);
            let b = f.value;
            Some(Rc::new(crate::objects::float::PyFloat::new((a / b).floor())))
        } else {
            None
        }
    }

    fn modulo(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value.is_zero() { return None; }
            Some(Rc::new(PyInt::new(&self.value % &i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            if f.value == 0.0 { return None; }
            Some(Rc::new(crate::objects::float::PyFloat::new(
                self.value.to_f64().unwrap_or(0.0) % f.value,
            )))
        } else {
            None
        }
    }

    fn pow(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if i.value.is_negative() {
                let a = self.value.to_f64().unwrap_or(0.0);
                let b = i.value.to_f64().unwrap_or(1.0);
                Some(Rc::new(crate::objects::float::PyFloat::new(a.powf(b))))
            } else if let Some(exp) = i.value.to_usize() {
                Some(Rc::new(PyInt::new(self.value.pow(exp as u32))))
            } else {
                None
            }
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::float::PyFloat::new(a.powf(f.value))))
        } else {
            None
        }
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(-&self.value)))
    }

    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(self.value.clone())))
    }

    fn abs_op(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(self.value.abs())))
    }

    fn bitand(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value & &i.value)))
        } else {
            None
        }
    }

    fn bitor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value | &i.value)))
        } else {
            None
        }
    }

    fn bitxor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(PyInt::new(&self.value ^ &i.value)))
        } else {
            None
        }
    }

    fn lshift(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if let Some(shift) = i.value.to_usize() {
                Some(Rc::new(PyInt::new(self.value.clone() << shift)))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn rshift(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            if let Some(shift) = i.value.to_usize() {
                Some(Rc::new(PyInt::new(self.value.clone() >> shift)))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn invert(&self) -> Option<Rc<dyn PyObject>> {
        Some(Rc::new(PyInt::new(!&self.value)))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a == f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == b_val)))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a == c.real && c.imag == 0.0)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a != f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != b_val)))
        } else if let Some(c) = other.as_any().downcast_ref::<crate::objects::complex::PyComplex>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a != c.real || c.imag != 0.0)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a < f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < b_val)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a <= f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= b_val)))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a > f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > b_val)))
        } else {
            None
        }
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= i.value)))
        } else if let Some(f) = other.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
            let a = self.value.to_f64().unwrap_or(0.0);
            Some(Rc::new(crate::objects::bool::PyBool::new(a >= f.value)))
        } else if let Some(b) = other.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
            let b_val = if b.value { BigInt::one() } else { BigInt::zero() };
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= b_val)))
        } else {
            None
        }
    }

    fn hash(&self) -> Result<i64, String> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.value.hash(&mut hasher);
        Ok(hasher.finish() as i64)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__format__" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("__format__".to_string(), move |args| {
                    let spec = if args.is_empty() { String::new() } else { args[0].str() };
                    if spec.is_empty() {
                        return Ok(Rc::new(crate::objects::string::PyString::new(val.to_string())));
                    }

                    // Extract type character if any (last char in spec)
                    let mut type_char = 'd';
                    if let Some(last_char) = spec.chars().last() {
                        if last_char.is_alphabetic() {
                            type_char = last_char;
                        }
                    }

                    let raw_str = match type_char {
                        'x' => format!("{:x}", val),
                        'X' => format!("{:X}", val),
                        'o' => format!("{:o}", val),
                        'b' => format!("{:b}", val),
                        'd' | 'n' => val.to_string(),
                        _ => return Err(format!("ValueError: Unknown format code '{}' for object of type 'int'", type_char)),
                    };

                    match crate::objects::string::format_align_width(&raw_str, &spec, '>') {
                        Ok(res) => Ok(Rc::new(crate::objects::string::PyString::new(res))),
                        Err(e) => Err(e),
                    }
                })))
            }
            _ => Err(format!("AttributeError: 'int' object has no attribute '{}'", attr)),
        }
    }
}
