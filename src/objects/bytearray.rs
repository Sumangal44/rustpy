use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::int::PyInt;
use crate::objects::string::PyString;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

fn bytearray_repr_bytes(val: &[u8]) -> String {
    let mut out = String::from("bytearray(b'");
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
    out.push_str("')");
    out
}

#[derive(Clone)]
pub struct PyByteArray {
    pub value: Rc<RefCell<Vec<u8>>>,
}

impl PyByteArray {
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }
}

impl std::fmt::Debug for PyByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyByteArray {
    fn get_type(&self) -> &'static str {
        "bytearray"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        bytearray_repr_bytes(&self.value.borrow())
    }

    fn is_truthy(&self) -> bool {
        !self.value.borrow().is_empty()
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let other_ba = other.as_any().downcast_ref::<PyByteArray>()?;
        let mut result = self.value.borrow().clone();
        result.extend(other_ba.value.borrow().iter());
        Some(Rc::new(PyByteArray::new(result)))
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let n = other.as_any().downcast_ref::<PyInt>()?;
        let count = n.as_i64().unwrap_or(0);
        if count <= 0 {
            return Some(Rc::new(PyByteArray::new(Vec::new())));
        }
        let val = self.value.borrow();
        let mut result = Vec::new();
        for _ in 0..count {
            result.extend(val.iter());
        }
        Some(Rc::new(PyByteArray::new(result)))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let other_ba = other.as_any().downcast_ref::<PyByteArray>()?;
        Some(Rc::new(crate::objects::bool::PyBool::new(*self.value.borrow() == *other_ba.value.borrow())))
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        let val = self.value.borrow();
        if let Some(idx_obj) = key.as_any().downcast_ref::<PyInt>() {
            let mut idx = idx_obj.as_i64().unwrap_or(0);
            let len = val.len() as i64;
            if idx < 0 {
                idx += len;
            }
            if idx >= 0 && idx < len {
                Ok(Rc::new(PyInt::from_i64(val[idx as usize] as i64)))
            } else {
                Err("IndexError: bytearray index out of range".to_string())
            }
        } else if let Some(slice) = key.as_any().downcast_ref::<crate::objects::slice::PySlice>() {
            let length = val.len();
            let (raw_start, raw_stop, step) = slice.resolve(length);
            let mut result = Vec::new();
            if step > 0 {
                let mut i = raw_start;
                while i < raw_stop {
                    result.push(val[i]);
                    i = (i as i64 + step) as usize;
                }
            } else if step < 0 {
                let start = if slice.start.is_some() { raw_start } else { length - 1 };
                let stop = if slice.stop.is_some() { raw_stop } else { 0 };
                let mut i = start;
                loop {
                    result.push(val[i]);
                    if i == stop { break; }
                    let next = i as i64 + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next as usize;
                }
            }
            Ok(Rc::new(PyByteArray::new(result)))
        } else {
            Err(format!(
                "TypeError: bytearray indices must be integers or slices, not {}",
                key.get_type()
            ))
        }
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
            let val = i.as_i64().unwrap_or(-1);
            if val < 0 || val > 255 {
                Ok(false)
            } else {
                Ok(self.value.borrow().contains(&(val as u8)))
            }
        } else {
            Err(format!(
                "TypeError: 'in <bytearray>' requires int as left operand, not '{}'",
                other.get_type()
            ))
        }
    }

    fn hash(&self) -> Result<i64, String> {
        Err(format!("TypeError: unhashable type: 'bytearray'"))
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyByteArrayIterator {
            value: Rc::clone(&self.value),
            index: RefCell::new(0),
        }))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let val = Rc::clone(&self.value);
        match attr {
            "append" => Ok(Rc::new(PyNativeFunction::new_pos_only("append".to_string(), move |args| {
                if args.len() != 1 {
                    return Err("TypeError: bytearray.append() takes exactly one argument".to_string());
                }
                let n = args[0].as_any().downcast_ref::<PyInt>()
                    .ok_or_else(|| "TypeError: bytearray.append() argument must be int".to_string())?;
                let v = n.as_i64().unwrap_or(0);
                if v < 0 || v > 255 {
                    return Err("ValueError: byte must be in range(0, 256)".to_string());
                }
                val.borrow_mut().push(v as u8);
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "extend" => Ok(Rc::new(PyNativeFunction::new_pos_only("extend".to_string(), move |args| {
                if args.len() != 1 {
                    return Err("TypeError: bytearray.extend() takes exactly one argument".to_string());
                }
                let iter = args[0].get_iter()?;
                let mut arr = val.borrow_mut();
                while let Some(item) = iter.get_next()? {
                    let n = item.as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: bytearray.extend() argument must be iterable of ints".to_string())?;
                    let v = n.as_i64().unwrap_or(0);
                    if v < 0 || v > 255 {
                        return Err("ValueError: byte must be in range(0, 256)".to_string());
                    }
                    arr.push(v as u8);
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "insert" => Ok(Rc::new(PyNativeFunction::new_pos_only("insert".to_string(), move |args| {
                if args.len() != 2 {
                    return Err("TypeError: bytearray.insert() takes exactly 2 arguments".to_string());
                }
                let idx_obj = args[0].as_any().downcast_ref::<PyInt>()
                    .ok_or_else(|| "TypeError: bytearray.insert() index must be int".to_string())?;
                let n = args[1].as_any().downcast_ref::<PyInt>()
                    .ok_or_else(|| "TypeError: bytearray.insert() value must be int".to_string())?;
                let v = n.as_i64().unwrap_or(0);
                if v < 0 || v > 255 {
                    return Err("ValueError: byte must be in range(0, 256)".to_string());
                }
                let mut arr = val.borrow_mut();
                let len = arr.len() as i64;
                let mut i = idx_obj.as_i64().unwrap_or(0);
                if i < 0 {
                    i = 0.max(len + i);
                }
                let pos = (i as usize).min(arr.len());
                arr.insert(pos, v as u8);
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "pop" => Ok(Rc::new(PyNativeFunction::new_pos_only("pop".to_string(), move |args| {
                if args.len() > 1 {
                    return Err("TypeError: bytearray.pop() takes at most 1 argument".to_string());
                }
                let mut arr = val.borrow_mut();
                if arr.is_empty() {
                    return Err("IndexError: pop from empty bytearray".to_string());
                }
                let idx = if args.is_empty() {
                    arr.len() - 1
                } else {
                    let n = args[0].as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: bytearray.pop() index must be int".to_string())?;
                    let mut i = n.as_i64().unwrap_or(0);
                    if i < 0 {
                        i = arr.len() as i64 + i;
                    }
                    if i < 0 || i as usize >= arr.len() {
                        return Err("IndexError: pop index out of range".to_string());
                    }
                    i as usize
                };
                let val_removed = arr.remove(idx);
                Ok(Rc::new(PyInt::from_i64(val_removed as i64)))
            }))),
            "remove" => Ok(Rc::new(PyNativeFunction::new_pos_only("remove".to_string(), move |args| {
                if args.len() != 1 {
                    return Err("TypeError: bytearray.remove() takes exactly one argument".to_string());
                }
                let n = args[0].as_any().downcast_ref::<PyInt>()
                    .ok_or_else(|| "TypeError: bytearray.remove() argument must be int".to_string())?;
                let v = n.as_i64().unwrap_or(0) as u8;
                let mut arr = val.borrow_mut();
                let pos = arr.iter().position(|&x| x == v);
                match pos {
                    Some(p) => { arr.remove(p); Ok(Rc::new(crate::objects::none::PyNone::new())) },
                    None => Err("ValueError: bytearray.remove(x): x not in bytearray".to_string()),
                }
            }))),
            "reverse" => Ok(Rc::new(PyNativeFunction::new_pos_only("reverse".to_string(), move |args| {
                if args.len() != 0 {
                    return Err("TypeError: bytearray.reverse() takes no arguments".to_string());
                }
                val.borrow_mut().reverse();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "clear" => Ok(Rc::new(PyNativeFunction::new_pos_only("clear".to_string(), move |args| {
                if args.len() != 0 {
                    return Err("TypeError: bytearray.clear() takes no arguments".to_string());
                }
                val.borrow_mut().clear();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "decode" => Ok(Rc::new(PyNativeFunction::new_pos_only("decode".to_string(), move |args| {
                if !args.is_empty() {
                    return Err("TypeError: decode() takes no arguments ({} given)".to_string());
                }
                match String::from_utf8(val.borrow().clone()) {
                    Ok(s) => Ok(Rc::new(PyString::new(s))),
                    Err(e) => Err(format!("UnicodeDecodeError: 'utf-8' codec cannot decode byte at position {}", e.utf8_error().valid_up_to())),
                }
            }))),
            _ => Err(format!("AttributeError: 'bytearray' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyByteArrayIterator {
    pub value: Rc<RefCell<Vec<u8>>>,
    pub index: RefCell<usize>,
}

impl std::fmt::Debug for PyByteArrayIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyByteArrayIterator {
    fn get_type(&self) -> &'static str {
        "bytearray_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<bytearray_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        let arr = self.value.borrow();
        if *idx < arr.len() {
            let item = Rc::new(PyInt::from_i64(arr[*idx] as i64));
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
