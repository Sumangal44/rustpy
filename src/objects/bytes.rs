use super::PyObject;
use crate::objects::bool::PyBool;
use crate::objects::int::PyInt;
use crate::objects::list::PyList;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use crate::objects::tuple::PyTuple;
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

fn whitespace_bytes() -> &'static [u8] {
    b" \t\n\r\x0b\x0c"
}

fn get_start_end(val: &[u8], args: &[Rc<dyn PyObject>], arg_offset: usize) -> (usize, usize) {
    let len = val.len();
    let start = if args.len() > arg_offset {
        if let Some(i) = args[arg_offset].as_any().downcast_ref::<PyInt>() {
            let s = i.as_i64().unwrap_or(0);
            if s < 0 {
                (len as i64 + s).max(0) as usize
            } else {
                (s as usize).min(len)
            }
        } else {
            0
        }
    } else {
        0
    };
    let end = if args.len() > arg_offset + 1 {
        if let Some(i) = args[arg_offset + 1].as_any().downcast_ref::<PyInt>() {
            let e = i.as_i64().unwrap_or(len as i64);
            if e < 0 {
                (len as i64 + e).max(0) as usize
            } else {
                (e as usize).min(len)
            }
        } else {
            len
        }
    } else {
        len
    };
    (start.min(len), end.max(start).min(len))
}

fn is_whitespace_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
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

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let other_bytes = other.as_any().downcast_ref::<PyBytes>()?;
        let mut result = self.value.clone();
        result.extend(other_bytes.value.iter());
        Some(Rc::new(PyBytes::new(result)))
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let n = other.as_any().downcast_ref::<PyInt>()?;
        let count = n.as_i64().unwrap_or(0);
        if count <= 0 {
            return Some(Rc::new(PyBytes::new(Vec::new())));
        }
        let mut result = Vec::new();
        for _ in 0..count {
            result.extend(self.value.iter());
        }
        Some(Rc::new(PyBytes::new(result)))
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBytes>() {
            Some(Rc::new(PyBool::new(self.value == b.value)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(b) = other.as_any().downcast_ref::<PyBytes>() {
            Some(Rc::new(PyBool::new(self.value != b.value)))
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
        if let Some(idx_obj) = key.as_any().downcast_ref::<PyInt>() {
            let mut idx = idx_obj.as_i64().unwrap_or(0);
            let len = self.value.len() as i64;
            if idx < 0 {
                idx += len;
            }
            if idx >= 0 && idx < len {
                Ok(Rc::new(PyInt::from_i64(self.value[idx as usize] as i64)))
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
                let start = if slice.start.is_some() { raw_start as i64 } else { length as i64 - 1 };
                let stop = if slice.stop.is_some() { raw_stop as i64 } else { -1i64 };
                let mut i = start;
                while i > stop {
                    result.push(self.value[i as usize]);
                    let next = i + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next;
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
        } else if let Some(i) = other.as_any().downcast_ref::<PyInt>() {
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
            "capitalize" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("capitalize".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: capitalize() takes no arguments (1 given)".to_string());
                    }
                    if val.is_empty() {
                        return Ok(Rc::new(PyBytes::new(Vec::new())));
                    }
                    let mut result = val.clone();
                    if result[0].is_ascii_lowercase() {
                        result[0] = result[0] - 32;
                    }
                    for b in result[1..].iter_mut() {
                        if b.is_ascii_uppercase() {
                            *b = *b + 32;
                        }
                    }
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "center" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("center".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: center() takes 1-2 arguments ({} given)".to_string());
                    }
                    let width = args[0].as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize().unwrap_or(0);
                    let fillbyte = if args.len() > 1 {
                        let fb = args[1].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: a bytes-like object is required".to_string())?;
                        fb.value.get(0).copied().unwrap_or(b' ')
                    } else {
                        b' '
                    };
                    if width <= val.len() {
                        Ok(Rc::new(PyBytes::new(val.clone())))
                    } else {
                        let padding = width - val.len();
                        let left = padding / 2;
                        let right = padding - left;
                        let mut result = vec![fillbyte; left];
                        result.extend(val.iter());
                        result.extend(std::iter::repeat(fillbyte).take(right));
                        Ok(Rc::new(PyBytes::new(result)))
                    }
                })))
            }
            "count" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("count".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: count() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    let cnt = if sub.is_empty() || slice.len() < sub.len() {
                        0
                    } else {
                        slice.windows(sub.len()).filter(|w| *w == sub.as_slice()).count()
                    };
                    Ok(Rc::new(PyInt::from_i64(cnt as i64)))
                })))
            }
            "decode" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("decode".to_string(), move |args| {
                    if args.len() > 2 {
                        return Err("TypeError: decode() takes at most 2 arguments ({} given)".to_string());
                    }
                    let encoding = if args.is_empty() { "utf-8".to_string() } else { args[0].str() };
                    let bytes = val.clone();
                    crate::encoding::decode(&bytes, &encoding).map(|s| Rc::new(PyString::new(s)) as Rc<dyn PyObject>)
                })))
            }
            "endswith" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("endswith".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: endswith() takes 1-3 arguments ({} given)".to_string());
                    }
                    let suffix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    Ok(Rc::new(PyBool::new(slice.ends_with(&suffix))))
                })))
            }
            "expandtabs" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("expandtabs".to_string(), move |args| {
                    if args.len() > 1 {
                        return Err("TypeError: expandtabs() takes at most 1 argument ({} given)".to_string());
                    }
                    let tabsize = if args.is_empty() {
                        8usize
                    } else {
                        args[0].as_any().downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .to_usize().unwrap_or(8)
                    };
                    let mut result = Vec::new();
                    let mut col = 0usize;
                    for &b in &val {
                        if b == b'\t' {
                            let spaces = tabsize - (col % tabsize);
                            result.extend(std::iter::repeat(b' ').take(spaces));
                            col += spaces;
                        } else {
                            result.push(b);
                            col += 1;
                            if b == b'\n' || b == b'\r' {
                                col = 0;
                            }
                        }
                    }
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "find" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("find".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: find() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64(start as i64)));
                    }
                    match slice.windows(sub.len()).position(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Ok(Rc::new(PyInt::from_i64(-1))),
                    }
                })))
            }
            "hex" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("hex".to_string(), move |args| {
                    if args.len() != 0 {
                        return Err("TypeError: hex() takes no arguments (1 given)".to_string());
                    }
                    let hex: String = val.iter().map(|b| format!("{:02x}", b)).collect();
                    Ok(Rc::new(PyString::new(hex)))
                })))
            }
            "index" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("index".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: index() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64(start as i64)));
                    }
                    match slice.windows(sub.len()).position(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Err("ValueError: subsection not found".to_string()),
                    }
                })))
            }
            "isalnum" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("isalnum".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isalnum() takes no arguments (1 given)".to_string());
                    }
                    Ok(Rc::new(PyBool::new(!val.is_empty() && val.iter().all(|b| b.is_ascii_alphanumeric()))))
                })))
            }
            "isalpha" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("isalpha".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isalpha() takes no arguments (1 given)".to_string());
                    }
                    Ok(Rc::new(PyBool::new(!val.is_empty() && val.iter().all(|b| b.is_ascii_alphabetic()))))
                })))
            }
            "isdigit" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("isdigit".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isdigit() takes no arguments (1 given)".to_string());
                    }
                    Ok(Rc::new(PyBool::new(!val.is_empty() && val.iter().all(|b| b.is_ascii_digit()))))
                })))
            }
            "islower" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("islower".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: islower() takes no arguments (1 given)".to_string());
                    }
                    let mut has_lower = false;
                    for &b in &val {
                        if b.is_ascii_uppercase() {
                            return Ok(Rc::new(PyBool::new(false)));
                        }
                        if b.is_ascii_lowercase() {
                            has_lower = true;
                        }
                    }
                    Ok(Rc::new(PyBool::new(has_lower)))
                })))
            }
            "isspace" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("isspace".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isspace() takes no arguments (1 given)".to_string());
                    }
                    Ok(Rc::new(PyBool::new(!val.is_empty() && val.iter().all(|&b| is_whitespace_byte(b)))))
                })))
            }
            "istitle" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("istitle".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: istitle() takes no arguments (1 given)".to_string());
                    }
                    if val.is_empty() {
                        return Ok(Rc::new(PyBool::new(false)));
                    }
                    let mut cased = false;
                    let mut prev_cased = false;
                    for &b in &val {
                        if b.is_ascii_uppercase() {
                            if prev_cased {
                                return Ok(Rc::new(PyBool::new(false)));
                            }
                            cased = true;
                            prev_cased = true;
                        } else if b.is_ascii_lowercase() {
                            if !prev_cased {
                                return Ok(Rc::new(PyBool::new(false)));
                            }
                            cased = true;
                            prev_cased = true;
                        } else {
                            prev_cased = false;
                        }
                    }
                    Ok(Rc::new(PyBool::new(cased)))
                })))
            }
            "isupper" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("isupper".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isupper() takes no arguments (1 given)".to_string());
                    }
                    let mut has_upper = false;
                    for &b in &val {
                        if b.is_ascii_lowercase() {
                            return Ok(Rc::new(PyBool::new(false)));
                        }
                        if b.is_ascii_uppercase() {
                            has_upper = true;
                        }
                    }
                    Ok(Rc::new(PyBool::new(has_upper)))
                })))
            }
            "join" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("join".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: join() takes exactly one argument ({} given)".to_string());
                    }
                    let iterable = &args[0];
                    let iter = iterable.get_iter()?;
                    let mut result = Vec::new();
                    let mut first = true;
                    while let Some(item) = iter.get_next()? {
                        if !first {
                            result.extend(val.iter());
                        }
                        first = false;
                        if let Some(b) = item.as_any().downcast_ref::<PyBytes>() {
                            result.extend(b.value.iter());
                        } else {
                            return Err("TypeError: sequence item in bytes join must be bytes, not '".to_string() + item.get_type() + "'");
                        }
                    }
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "ljust" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("ljust".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: ljust() takes 1-2 arguments ({} given)".to_string());
                    }
                    let width = args[0].as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize().unwrap_or(0);
                    let fillbyte = if args.len() > 1 {
                        let fb = args[1].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: a bytes-like object is required".to_string())?;
                        fb.value.get(0).copied().unwrap_or(b' ')
                    } else {
                        b' '
                    };
                    if width <= val.len() {
                        Ok(Rc::new(PyBytes::new(val.clone())))
                    } else {
                        let mut result = val.clone();
                        result.extend(std::iter::repeat(fillbyte).take(width - val.len()));
                        Ok(Rc::new(PyBytes::new(result)))
                    }
                })))
            }
            "lower" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("lower".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: lower() takes no arguments (1 given)".to_string());
                    }
                    let result: Vec<u8> = val.iter().map(|&b| {
                        if b.is_ascii_uppercase() { b + 32 } else { b }
                    }).collect();
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "lstrip" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("lstrip".to_string(), move |args| {
                    if args.len() > 1 {
                        return Err("TypeError: lstrip() takes at most 1 argument (2 given)".to_string());
                    }
                    let chars = if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        whitespace_bytes().to_vec()
                    } else {
                        args[0].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: expected a bytes object".to_string())?
                            .value.clone()
                    };
                    let mut start = 0;
                    while start < val.len() && chars.contains(&val[start]) {
                        start += 1;
                    }
                    Ok(Rc::new(PyBytes::new(val[start..].to_vec())))
                })))
            }
            "maketrans" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("maketrans".to_string(), move |args| {
                    if args.len() != 2 {
                        return Err("TypeError: maketrans() takes exactly 2 arguments ({} given)".to_string());
                    }
                    let from_b = args[0].as_any().downcast_ref::<PyBytes>()
                        .ok_or_else(|| "TypeError: expected a bytes object".to_string())?;
                    let to_b = args[1].as_any().downcast_ref::<PyBytes>()
                        .ok_or_else(|| "TypeError: expected a bytes object".to_string())?;
                    if from_b.value.len() != to_b.value.len() {
                        return Err("ValueError: the first maketrans argument must be the same length as the second".to_string());
                    }
                    let mut table: Vec<u8> = (0..=255u16).map(|i| i as u8).collect();
                    for (f, t) in from_b.value.iter().zip(to_b.value.iter()) {
                        table[*f as usize] = *t;
                    }
                    Ok(Rc::new(PyBytes::new(table)))
                })))
            }
            "partition" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("partition".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: partition() takes exactly one argument ({} given)".to_string());
                    }
                    let sep = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".to_string());
                    }
                    let sep_len = sep.len();
                    match val.windows(sep_len).position(|w| w == sep.as_slice()) {
                        Some(pos) => {
                            let head = Rc::new(PyBytes::new(val[..pos].to_vec())) as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyBytes::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyBytes::new(val[pos + sep_len..].to_vec())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyBytes::new(Vec::new())) as Rc<dyn PyObject>;
                            let head = Rc::new(PyBytes::new(val.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, empty.clone(), empty])))
                        }
                    }
                })))
            }
            "removeprefix" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("removeprefix".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: removeprefix() takes exactly one argument ({} given)".to_string());
                    }
                    let prefix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    if val.starts_with(&prefix) {
                        Ok(Rc::new(PyBytes::new(val[prefix.len()..].to_vec())))
                    } else {
                        Ok(Rc::new(PyBytes::new(val.clone())))
                    }
                })))
            }
            "removesuffix" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("removesuffix".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: removesuffix() takes exactly one argument ({} given)".to_string());
                    }
                    let suffix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    if val.ends_with(&suffix) {
                        Ok(Rc::new(PyBytes::new(val[..val.len() - suffix.len()].to_vec())))
                    } else {
                        Ok(Rc::new(PyBytes::new(val.clone())))
                    }
                })))
            }
            "replace" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("replace".to_string(), move |args| {
                    if args.len() < 2 || args.len() > 3 {
                        return Err("TypeError: replace() takes 2-3 arguments ({} given)".to_string());
                    }
                    let old = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let new = if let Some(b) = args[1].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let count = if args.len() > 2 {
                        args[2].as_any().downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64().unwrap_or(-1)
                    } else {
                        -1
                    };
                    if old.is_empty() {
                        let mut result = Vec::new();
                        let limit = if count < 0 { val.len() + 1 } else { (count as usize).min(val.len() + 1) };
                        for i in 0..limit {
                            if i > 0 {
                                result.extend(new.iter());
                            }
                            if i < val.len() {
                                result.push(val[i]);
                            }
                        }
                        if limit < val.len() + 1 {
                            result.extend(val[limit - 1..].iter());
                        }
                        return Ok(Rc::new(PyBytes::new(result)));
                    }
                    let mut result = Vec::new();
                    let mut i = 0;
                    let mut replacements = 0i64;
                    while i <= val.len() {
                        if (count >= 0 && replacements >= count) || i + old.len() > val.len() {
                            result.extend(val[i..].iter());
                            break;
                        }
                        if val[i..].starts_with(&old) {
                            result.extend(new.iter());
                            i += old.len();
                            replacements += 1;
                        } else {
                            result.push(val[i]);
                            i += 1;
                        }
                    }
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "rfind" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rfind".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: rfind() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64((start + slice.len()) as i64)));
                    }
                    match slice.windows(sub.len()).rposition(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Ok(Rc::new(PyInt::from_i64(-1))),
                    }
                })))
            }
            "rindex" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rindex".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: rindex() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64((start + slice.len()) as i64)));
                    }
                    match slice.windows(sub.len()).rposition(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Err("ValueError: subsection not found".to_string()),
                    }
                })))
            }
            "rjust" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rjust".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: rjust() takes 1-2 arguments ({} given)".to_string());
                    }
                    let width = args[0].as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize().unwrap_or(0);
                    let fillbyte = if args.len() > 1 {
                        let fb = args[1].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: a bytes-like object is required".to_string())?;
                        fb.value.get(0).copied().unwrap_or(b' ')
                    } else {
                        b' '
                    };
                    if width <= val.len() {
                        Ok(Rc::new(PyBytes::new(val.clone())))
                    } else {
                        let mut result = vec![fillbyte; width - val.len()];
                        result.extend(val.iter());
                        Ok(Rc::new(PyBytes::new(result)))
                    }
                })))
            }
            "rpartition" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rpartition".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: rpartition() takes exactly one argument ({} given)".to_string());
                    }
                    let sep = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".to_string());
                    }
                    let sep_len = sep.len();
                    match val.windows(sep_len).rposition(|w| w == sep.as_slice()) {
                        Some(pos) => {
                            let head = Rc::new(PyBytes::new(val[..pos].to_vec())) as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyBytes::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyBytes::new(val[pos + sep_len..].to_vec())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyBytes::new(Vec::new())) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyBytes::new(val.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![empty.clone(), empty, tail])))
                        }
                    }
                })))
            }
            "rsplit" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rsplit".to_string(), move |args| {
                    if args.len() > 2 {
                        return Err("TypeError: rsplit() takes at most 2 arguments ({} given)".to_string());
                    }
                    let sep = if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        None
                    } else {
                        Some(args[0].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: expected a bytes object".to_string())?
                            .value.clone())
                    };
                    let maxsplit = if args.len() > 1 {
                        args[1].as_any().downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64().unwrap_or(-1)
                    } else {
                        -1
                    };
                    let parts = match sep {
                        None => {
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            let mut i = val.len();
                            while i > 0 {
                                while i > 0 && is_whitespace_byte(val[i - 1]) {
                                    i -= 1;
                                }
                                if i == 0 { break; }
                                let end = i;
                                while i > 0 && !is_whitespace_byte(val[i - 1]) {
                                    i -= 1;
                                }
                                result.push(val[i..end].to_vec());
                                if result.len() >= limit { break; }
                            }
                            result.reverse();
                            if result.is_empty() && !val.is_empty() {
                                // no non-whitespace found
                            }
                            if result.is_empty() && val.is_empty() {
                                result.push(Vec::new());
                            }
                            result
                        }
                        Some(ref sep_bytes) => {
                            if sep_bytes.is_empty() {
                                return Err("ValueError: empty separator".to_string());
                            }
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let mut remaining = val.clone();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            for _ in 0..limit {
                                match remaining.windows(sep_bytes.len()).rposition(|w| w == sep_bytes.as_slice()) {
                                    Some(pos) => {
                                        result.push(remaining[pos + sep_bytes.len()..].to_vec());
                                        remaining = remaining[..pos].to_vec();
                                    }
                                    None => break,
                                }
                            }
                            result.push(remaining);
                            result.reverse();
                            result
                        }
                    };
                    let list_items: Vec<Rc<dyn PyObject>> = parts.into_iter()
                        .map(|p| Rc::new(PyBytes::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                })))
            }
            "rstrip" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("rstrip".to_string(), move |args| {
                    if args.len() > 1 {
                        return Err("TypeError: rstrip() takes at most 1 argument (2 given)".to_string());
                    }
                    let chars = if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        whitespace_bytes().to_vec()
                    } else {
                        args[0].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: expected a bytes object".to_string())?
                            .value.clone()
                    };
                    let mut end = val.len();
                    while end > 0 && chars.contains(&val[end - 1]) {
                        end -= 1;
                    }
                    Ok(Rc::new(PyBytes::new(val[..end].to_vec())))
                })))
            }
            "split" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("split".to_string(), move |args| {
                    if args.len() > 2 {
                        return Err("TypeError: split() takes at most 2 arguments ({} given)".to_string());
                    }
                    let sep = if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        None
                    } else {
                        Some(args[0].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: expected a bytes object".to_string())?
                            .value.clone())
                    };
                    let maxsplit = if args.len() > 1 {
                        args[1].as_any().downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64().unwrap_or(-1)
                    } else {
                        -1
                    };
                    let parts = match sep {
                        None => {
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            let mut i = 0;
                            let mut splits = 0usize;
                            while i < val.len() {
                                while i < val.len() && is_whitespace_byte(val[i]) {
                                    i += 1;
                                }
                                if i >= val.len() { break; }
                                let start = i;
                                while i < val.len() && !is_whitespace_byte(val[i]) {
                                    i += 1;
                                }
                                result.push(val[start..i].to_vec());
                                splits += 1;
                                if splits >= limit {
                                    while i < val.len() && is_whitespace_byte(val[i]) {
                                        i += 1;
                                    }
                                    if i < val.len() {
                                        result.push(val[i..].to_vec());
                                    }
                                    break;
                                }
                            }
                            if result.is_empty() {
                                result.push(Vec::new());
                            }
                            result
                        }
                        Some(ref sep_bytes) => {
                            if sep_bytes.is_empty() {
                                return Err("ValueError: empty separator".to_string());
                            }
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let mut remaining = val.clone();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            for _ in 0..limit {
                                match remaining.windows(sep_bytes.len()).position(|w| w == sep_bytes.as_slice()) {
                                    Some(pos) => {
                                        result.push(remaining[..pos].to_vec());
                                        remaining = remaining[pos + sep_bytes.len()..].to_vec();
                                    }
                                    None => break,
                                }
                            }
                            result.push(remaining);
                            result
                        }
                    };
                    let list_items: Vec<Rc<dyn PyObject>> = parts.into_iter()
                        .map(|p| Rc::new(PyBytes::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                })))
            }
            "splitlines" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("splitlines".to_string(), move |args| {
                    if args.len() > 1 {
                        return Err("TypeError: splitlines() takes at most 1 argument ({} given)".to_string());
                    }
                    let keepends = if args.is_empty() {
                        false
                    } else {
                        args[0].as_any().downcast_ref::<PyBool>()
                            .map(|b| b.value)
                            .unwrap_or(false)
                    };
                    let mut result: Vec<Vec<u8>> = Vec::new();
                    let mut i = 0;
                    while i < val.len() {
                        let start = i;
                        while i < val.len() && val[i] != b'\n' && val[i] != b'\r' {
                            i += 1;
                        }
                        if i >= val.len() {
                            result.push(val[start..].to_vec());
                            break;
                        }
                        if val[i] == b'\r' && i + 1 < val.len() && val[i + 1] == b'\n' {
                            if keepends {
                                result.push(val[start..i + 2].to_vec());
                            } else {
                                result.push(val[start..i].to_vec());
                            }
                            i += 2;
                        } else {
                            if keepends {
                                result.push(val[start..i + 1].to_vec());
                            } else {
                                result.push(val[start..i].to_vec());
                            }
                            i += 1;
                        }
                    }
                    let list_items: Vec<Rc<dyn PyObject>> = result.into_iter()
                        .map(|p| Rc::new(PyBytes::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                })))
            }
            "startswith" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("startswith".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: startswith() takes 1-3 arguments ({} given)".to_string());
                    }
                    let prefix = if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes object".to_string());
                    };
                    let (start, end) = get_start_end(&val, &args, 1);
                    let slice = &val[start..end];
                    Ok(Rc::new(PyBool::new(slice.starts_with(&prefix))))
                })))
            }
            "strip" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("strip".to_string(), move |args| {
                    if args.len() > 1 {
                        return Err("TypeError: strip() takes at most 1 argument (2 given)".to_string());
                    }
                    let chars = if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        whitespace_bytes().to_vec()
                    } else {
                        args[0].as_any().downcast_ref::<PyBytes>()
                            .ok_or_else(|| "TypeError: expected a bytes object".to_string())?
                            .value.clone()
                    };
                    let mut start = 0;
                    while start < val.len() && chars.contains(&val[start]) {
                        start += 1;
                    }
                    let mut end = val.len();
                    while end > start && chars.contains(&val[end - 1]) {
                        end -= 1;
                    }
                    Ok(Rc::new(PyBytes::new(val[start..end].to_vec())))
                })))
            }
            "swapcase" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("swapcase".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: swapcase() takes no arguments (1 given)".to_string());
                    }
                    let result: Vec<u8> = val.iter().map(|&b| {
                        if b.is_ascii_uppercase() { b + 32 }
                        else if b.is_ascii_lowercase() { b - 32 }
                        else { b }
                    }).collect();
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "title" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("title".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: title() takes no arguments (1 given)".to_string());
                    }
                    let mut result = Vec::with_capacity(val.len());
                    let mut at_start = true;
                    for &b in &val {
                        if at_start {
                            if b.is_ascii_lowercase() {
                                result.push(b - 32);
                            } else {
                                result.push(b);
                            }
                        } else {
                            if b.is_ascii_uppercase() {
                                result.push(b + 32);
                            } else {
                                result.push(b);
                            }
                        }
                        at_start = b.is_ascii_alphanumeric();
                    }
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "translate" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("translate".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: translate() takes exactly one argument ({} given)".to_string());
                    }
                    if args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        return Ok(Rc::new(PyBytes::new(val.clone())));
                    }
                    let table = args[0].as_any().downcast_ref::<PyBytes>()
                        .ok_or_else(|| "TypeError: expected a bytes object".to_string())?;
                    if table.value.len() != 256 {
                        return Err("ValueError: translation table must be 256 bytes long".to_string());
                    }
                    let result: Vec<u8> = val.iter().map(|&b| table.value[b as usize]).collect();
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "upper" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("upper".to_string(), move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: upper() takes no arguments (1 given)".to_string());
                    }
                    let result: Vec<u8> = val.iter().map(|&b| {
                        if b.is_ascii_lowercase() { b - 32 } else { b }
                    }).collect();
                    Ok(Rc::new(PyBytes::new(result)))
                })))
            }
            "zfill" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only("zfill".to_string(), move |args| {
                    if args.len() != 1 {
                        return Err("TypeError: zfill() takes exactly one argument ({} given)".to_string());
                    }
                    let width = args[0].as_any().downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize().unwrap_or(0);
                    if width <= val.len() {
                        return Ok(Rc::new(PyBytes::new(val.clone())));
                    }
                    let sign_prefix = if !val.is_empty() && (val[0] == b'+' || val[0] == b'-') {
                        vec![val[0]]
                    } else {
                        Vec::new()
                    };
                    let padding = width - val.len();
                    let mut result = sign_prefix.clone();
                    result.extend(std::iter::repeat(b'0').take(padding));
                    if !sign_prefix.is_empty() {
                        result.extend(&val[1..]);
                    } else {
                        result.extend(val.iter());
                    }
                    Ok(Rc::new(PyBytes::new(result)))
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
            let item = Rc::new(PyInt::from_i64(self.value[*idx] as i64));
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
