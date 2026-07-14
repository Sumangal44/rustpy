use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyBytes {
    pub value: Vec<u8>,
}

impl PyBytes {
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }
}

fn bytes_repr_bytes(val: &[u8]) -> String {
    let mut out = String::from("b'");
    for &b in val {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\'' => out.push_str("\\'"),
            0x0a => out.push_str("\\n"),
            0x0d => out.push_str("\\r"),
            0x09 => out.push_str("\\t"),
            0x20..=0x7e => out.push(b as char),
            _ => out.push_str(&format!("\\x{:02x}", b)),
        }
    }
    out.push('\'');
    out
}

impl PyObject for PyBytes {
    fn get_type(&self) -> &'static str {
        "bytes"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        bytes_repr_bytes(&self.value)
    }

    fn str(&self) -> String {
        bytes_repr_bytes(&self.value)
    }

    fn is_truthy(&self) -> bool {
        !self.value.is_empty()
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBytes>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == b.value)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBytes>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != b.value)))
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

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx_obj) = key.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let mut idx = idx_obj.as_i64().unwrap_or(0);
            let len = self.value.len() as i64;
            if idx < 0 {
                idx += len;
            }
            if idx >= 0 && idx < len {
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(self.value[idx as usize] as i64)))
            } else {
                Err("IndexError: bytes index out of range".to_string())
            }
        } else if let Some(slice) = key.as_any().downcast_ref::<crate::objects::slice::PySlice>() {
            let length = self.value.len();
            let (raw_start, raw_stop, step) = slice.resolve(length);
            let mut result = Vec::new();
            if step > 0 {
                let mut i = raw_start;
                while i < raw_stop {
                    result.push(self.value[i]);
                    i = (i as i64 + step) as usize;
                }
            } else if step < 0 {
                let start = if slice.start.is_some() { raw_start } else { length - 1 };
                let stop = if slice.stop.is_some() { raw_stop } else { 0 };
                let mut i = start;
                loop {
                    result.push(self.value[i]);
                    if i == stop { break; }
                    let next = i as i64 + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next as usize;
                }
            }
            Ok(Rc::new(PyBytes::new(result)))
        } else {
            Err(format!(
                "TypeError: bytes indices must be integers or slices, not {}",
                key.get_type()
            ))
        }
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        if let Some(b) = other.as_any().downcast_ref::<PyBytes>() {
            Ok(self.value.windows(b.value.len()).any(|w| w == b.value.as_slice()))
        } else if let Some(i) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let val = i.as_i64().unwrap_or(-1);
            if val < 0 || val > 255 {
                Ok(false)
            } else {
                Ok(self.value.contains(&(val as u8)))
            }
        } else {
            Err(format!(
                "TypeError: 'in <bytes>' requires bytes or int as left operand, not '{}'",
                other.get_type()
            ))
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyBytesIterator {
            value: self.value.clone(),
            index: std::cell::RefCell::new(0),
        }))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let val = self.value.clone();
        match attr {
            "decode" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("decode".to_string(), move |args| {
                    if args.len() != 0 {
                        return Err("TypeError: decode() takes no arguments ({} given)".to_string());
                    }
                    match String::from_utf8(val.clone()) {
                        Ok(s) => Ok(Rc::new(crate::objects::string::PyString::new(s))),
                        Err(e) => Err(format!("UnicodeDecodeError: 'utf-8' codec cannot decode byte at position {}", e.utf8_error().valid_up_to())),
                    }
                })))
            }
            "hex" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("hex".to_string(), move |args| {
                    if args.len() != 0 {
                        return Err("TypeError: hex() takes no arguments ({} given)".to_string());
                    }
                    let hex: String = val.iter().map(|b| format!("{:02x}", b)).collect();
                    Ok(Rc::new(crate::objects::string::PyString::new(hex)))
                })))
            }
            "count" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("count".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: count() takes exactly one argument ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let cnt = if sub.is_empty() {
                        0
                    } else {
                        val.windows(sub.len()).filter(|w| *w == sub.as_slice()).count()
                    };
                    Ok(Rc::new(crate::objects::int::PyInt::from_i64(cnt as i64)))
                })))
            }
            "index" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("index".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: index() takes exactly one argument ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    val.windows(sub.len())
                        .position(|w| w == sub.as_slice())
                        .map(|pos| Rc::new(crate::objects::int::PyInt::from_i64(pos as i64)) as Rc<dyn PyObject>)
                        .ok_or_else(|| "ValueError: subsection not found".to_string())
                })))
            }
            "startswith" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("startswith".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: startswith() takes exactly one argument ({} given)".to_string());
                    }
                    let prefix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    Ok(Rc::new(crate::objects::bool::PyBool::new(val.starts_with(&prefix))))
                })))
            }
            "endswith" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("endswith".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: endswith() takes exactly one argument ({} given)".to_string());
                    }
                    let suffix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    Ok(Rc::new(crate::objects::bool::PyBool::new(val.ends_with(&suffix))))
                })))
            }
            "find" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("find".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: find() takes exactly one argument ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    match val.windows(sub.len()).position(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(crate::objects::int::PyInt::from_i64(pos as i64))),
                        None => Ok(Rc::new(crate::objects::int::PyInt::from_i64(-1))),
                    }
                })))
            }
            _ => Err(format!("AttributeError: 'bytes' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyBytesIterator {
    pub value: Vec<u8>,
    pub index: std::cell::RefCell<usize>,
}

impl std::fmt::Debug for PyBytesIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyBytesIterator {
    fn get_type(&self) -> &'static str {
        "bytes_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<bytes_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        if *idx < self.value.len() {
            let item = Rc::new(crate::objects::int::PyInt::from_i64(self.value[*idx] as i64));
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
