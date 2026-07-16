use super::PyObject;
use crate::objects::bool::PyBool;
use crate::objects::int::PyInt;
use crate::objects::list::PyList;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use crate::objects::tuple::PyTuple;
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

fn whitespace_bytes() -> &'static [u8] {
    b" \t\n\r\x0b\x0c"
}

fn is_whitespace_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn get_start_end_from_borrowed(
    val: &[u8],
    args: &[Rc<dyn PyObject>],
    arg_offset: usize,
) -> (usize, usize) {
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
        Some(Rc::new(PyBool::new(
            *self.value.borrow() == *other_ba.value.borrow(),
        )))
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
        } else if let Some(slice) = key
            .as_any()
            .downcast_ref::<crate::objects::slice::PySlice>()
        {
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
                let start = if slice.start.is_some() {
                    raw_start as i64
                } else {
                    length as i64 - 1
                };
                let stop = if slice.stop.is_some() {
                    raw_stop as i64
                } else {
                    -1i64
                };
                let mut i = start;
                while i > stop {
                    result.push(val[i as usize]);
                    let next = i + step;
                    if next < 0 || next as usize >= length {
                        break;
                    }
                    i = next;
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
            "append" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "append".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: bytearray.append() takes exactly one argument".to_string()
                        );
                    }
                    let n = args[0].as_any().downcast_ref::<PyInt>().ok_or_else(|| {
                        "TypeError: bytearray.append() argument must be int".to_string()
                    })?;
                    let v = n.as_i64().unwrap_or(0);
                    if v < 0 || v > 255 {
                        return Err("ValueError: byte must be in range(0, 256)".to_string());
                    }
                    val.borrow_mut().push(v as u8);
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "capitalize" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "capitalize".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err(
                            "TypeError: capitalize() takes no arguments (1 given)".to_string()
                        );
                    }
                    let mut result = val.borrow().clone();
                    if result.is_empty() {
                        return Ok(Rc::new(PyByteArray::new(result)));
                    }
                    if result[0].is_ascii_lowercase() {
                        result[0] = result[0] - 32;
                    }
                    for b in result[1..].iter_mut() {
                        if b.is_ascii_uppercase() {
                            *b = *b + 32;
                        }
                    }
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "center" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "center".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err(
                            "TypeError: center() takes 1-2 arguments ({} given)".to_string()
                        );
                    }
                    let width = args[0]
                        .as_any()
                        .downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize()
                        .unwrap_or(0);
                    let fillbyte =
                        if args.len() > 1 {
                            let fb = args[1].as_any().downcast_ref::<PyByteArray>().ok_or_else(
                                || "TypeError: a bytes-like object is required".to_string(),
                            )?;
                            fb.value.borrow().get(0).copied().unwrap_or(b' ')
                        } else {
                            b' '
                        };
                    let borrowed = val.borrow();
                    if width <= borrowed.len() {
                        Ok(Rc::new(PyByteArray::new(borrowed.clone())))
                    } else {
                        let padding = width - borrowed.len();
                        let left = padding / 2;
                        let right = padding - left;
                        let mut result = vec![fillbyte; left];
                        result.extend(borrowed.iter());
                        result.extend(std::iter::repeat(fillbyte).take(right));
                        Ok(Rc::new(PyByteArray::new(result)))
                    }
                },
            ))),
            "clear" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "clear".to_string(),
                move |args| {
                    if args.len() != 0 {
                        return Err("TypeError: bytearray.clear() takes no arguments".to_string());
                    }
                    val.borrow_mut().clear();
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "copy" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "copy".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: copy() takes no arguments (1 given)".to_string());
                    }
                    Ok(Rc::new(PyByteArray::new(val.borrow().clone())))
                },
            ))),
            "count" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "count".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: count() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    let cnt = if sub.is_empty() || slice.len() < sub.len() {
                        0
                    } else {
                        slice
                            .windows(sub.len())
                            .filter(|w| *w == sub.as_slice())
                            .count()
                    };
                    Ok(Rc::new(PyInt::from_i64(cnt as i64)))
                },
            ))),
            "decode" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "decode".to_string(),
                move |args| {
                    if args.len() > 2 {
                        return Err(
                            "TypeError: decode() takes at most 2 arguments ({} given)".to_string()
                        );
                    }
                    let encoding = if args.is_empty() {
                        "utf-8".to_string()
                    } else {
                        args[0].str()
                    };
                    let bytes = val.borrow().clone();
                    crate::encoding::decode(&bytes, &encoding)
                        .map(|s| Rc::new(PyString::new(s)) as Rc<dyn PyObject>)
                },
            ))),
            "endswith" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "endswith".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err(
                            "TypeError: endswith() takes 1-3 arguments ({} given)".to_string()
                        );
                    }
                    let suffix = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    Ok(Rc::new(PyBool::new(slice.ends_with(&suffix))))
                },
            ))),
            "expandtabs" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "expandtabs".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: expandtabs() takes at most 1 argument ({} given)"
                                .to_string(),
                        );
                    }
                    let tabsize = if args.is_empty() {
                        8usize
                    } else {
                        args[0]
                            .as_any()
                            .downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .to_usize()
                            .unwrap_or(8)
                    };
                    let borrowed = val.borrow();
                    let mut result = Vec::new();
                    let mut col = 0usize;
                    for &b in borrowed.iter() {
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
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "extend" => {
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "extend".to_string(),
                    move |args| {
                        if args.len() != 1 {
                            return Err("TypeError: bytearray.extend() takes exactly one argument"
                                .to_string());
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
                        Ok(Rc::new(PyNone::new()))
                    },
                )))
            }
            "find" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "find".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: find() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64(start as i64)));
                    }
                    match slice.windows(sub.len()).position(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Ok(Rc::new(PyInt::from_i64(-1))),
                    }
                },
            ))),
            "hex" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "hex".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: hex() takes no arguments (1 given)".to_string());
                    }
                    let hex: String = val.borrow().iter().map(|b| format!("{:02x}", b)).collect();
                    Ok(Rc::new(PyString::new(hex)))
                },
            ))),
            "index" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "index".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: index() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64(start as i64)));
                    }
                    match slice.windows(sub.len()).position(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Err("ValueError: subsection not found".to_string()),
                    }
                },
            ))),
            "insert" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "insert".to_string(),
                move |args| {
                    if args.len() != 2 {
                        return Err(
                            "TypeError: bytearray.insert() takes exactly 2 arguments".to_string()
                        );
                    }
                    let idx_obj = args[0].as_any().downcast_ref::<PyInt>().ok_or_else(|| {
                        "TypeError: bytearray.insert() index must be int".to_string()
                    })?;
                    let n = args[1].as_any().downcast_ref::<PyInt>().ok_or_else(|| {
                        "TypeError: bytearray.insert() value must be int".to_string()
                    })?;
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
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "isalnum" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isalnum".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isalnum() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    Ok(Rc::new(PyBool::new(
                        !borrowed.is_empty() && borrowed.iter().all(|b| b.is_ascii_alphanumeric()),
                    )))
                },
            ))),
            "isalpha" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isalpha".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isalpha() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    Ok(Rc::new(PyBool::new(
                        !borrowed.is_empty() && borrowed.iter().all(|b| b.is_ascii_alphabetic()),
                    )))
                },
            ))),
            "isdigit" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isdigit".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isdigit() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    Ok(Rc::new(PyBool::new(
                        !borrowed.is_empty() && borrowed.iter().all(|b| b.is_ascii_digit()),
                    )))
                },
            ))),
            "islower" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "islower".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: islower() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    let mut has_lower = false;
                    for &b in borrowed.iter() {
                        if b.is_ascii_uppercase() {
                            return Ok(Rc::new(PyBool::new(false)));
                        }
                        if b.is_ascii_lowercase() {
                            has_lower = true;
                        }
                    }
                    Ok(Rc::new(PyBool::new(has_lower)))
                },
            ))),
            "isspace" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isspace".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isspace() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    Ok(Rc::new(PyBool::new(
                        !borrowed.is_empty() && borrowed.iter().all(|&b| is_whitespace_byte(b)),
                    )))
                },
            ))),
            "istitle" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "istitle".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: istitle() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    if borrowed.is_empty() {
                        return Ok(Rc::new(PyBool::new(false)));
                    }
                    let mut cased = false;
                    let mut prev_cased = false;
                    for &b in borrowed.iter() {
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
                },
            ))),
            "isupper" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isupper".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isupper() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    let mut has_upper = false;
                    for &b in borrowed.iter() {
                        if b.is_ascii_lowercase() {
                            return Ok(Rc::new(PyBool::new(false)));
                        }
                        if b.is_ascii_uppercase() {
                            has_upper = true;
                        }
                    }
                    Ok(Rc::new(PyBool::new(has_upper)))
                },
            ))),
            "join" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "join".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: join() takes exactly one argument ({} given)".to_string()
                        );
                    }
                    let iterable = &args[0];
                    let iter = iterable.get_iter()?;
                    let sep = val.borrow().clone();
                    let mut result = Vec::new();
                    let mut first = true;
                    while let Some(item) = iter.get_next()? {
                        if !first {
                            result.extend(sep.iter());
                        }
                        first = false;
                        if let Some(b) = item.as_any().downcast_ref::<PyByteArray>() {
                            result.extend(b.value.borrow().iter());
                        } else if let Some(b) = item
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            result.extend(b.value.iter());
                        } else {
                            return Err(
                                "TypeError: sequence item in bytes join must be bytes, not '"
                                    .to_string()
                                    + item.get_type()
                                    + "'",
                            );
                        }
                    }
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "ljust" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "ljust".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: ljust() takes 1-2 arguments ({} given)".to_string());
                    }
                    let width = args[0]
                        .as_any()
                        .downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize()
                        .unwrap_or(0);
                    let fillbyte =
                        if args.len() > 1 {
                            let fb = args[1].as_any().downcast_ref::<PyByteArray>().ok_or_else(
                                || "TypeError: a bytes-like object is required".to_string(),
                            )?;
                            fb.value.borrow().get(0).copied().unwrap_or(b' ')
                        } else {
                            b' '
                        };
                    let borrowed = val.borrow();
                    if width <= borrowed.len() {
                        Ok(Rc::new(PyByteArray::new(borrowed.clone())))
                    } else {
                        let mut result = borrowed.clone();
                        result.extend(std::iter::repeat(fillbyte).take(width - borrowed.len()));
                        Ok(Rc::new(PyByteArray::new(result)))
                    }
                },
            ))),
            "lower" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "lower".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: lower() takes no arguments (1 given)".to_string());
                    }
                    let result: Vec<u8> = val
                        .borrow()
                        .iter()
                        .map(|&b| if b.is_ascii_uppercase() { b + 32 } else { b })
                        .collect();
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "lstrip" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "lstrip".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: lstrip() takes at most 1 argument (2 given)".to_string()
                        );
                    }
                    let chars =
                        if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                            whitespace_bytes().to_vec()
                        } else if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                            b.value.borrow().clone()
                        } else if let Some(b) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            b.value.clone()
                        } else {
                            return Err("TypeError: expected a bytes-like object".to_string());
                        };
                    let borrowed = val.borrow();
                    let mut start = 0;
                    while start < borrowed.len() && chars.contains(&borrowed[start]) {
                        start += 1;
                    }
                    Ok(Rc::new(PyByteArray::new(borrowed[start..].to_vec())))
                },
            ))),
            "maketrans" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "maketrans".to_string(),
                move |args| {
                    if args.len() != 2 {
                        return Err(
                            "TypeError: maketrans() takes exactly 2 arguments ({} given)"
                                .to_string(),
                        );
                    }
                    let from_b = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let to_b = if let Some(b) = args[1].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[1]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    if from_b.len() != to_b.len() {
                        return Err("ValueError: the first maketrans argument must be the same length as the second".to_string());
                    }
                    let mut table: Vec<u8> = (0..=255u16).map(|i| i as u8).collect();
                    for (f, t) in from_b.iter().zip(to_b.iter()) {
                        table[*f as usize] = *t;
                    }
                    Ok(Rc::new(PyByteArray::new(table)))
                },
            ))),
            "partition" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "partition".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: partition() takes exactly one argument ({} given)"
                                .to_string(),
                        );
                    }
                    let sep = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".to_string());
                    }
                    let sep_len = sep.len();
                    let borrowed = val.borrow();
                    match borrowed.windows(sep_len).position(|w| w == sep.as_slice()) {
                        Some(pos) => {
                            let head = Rc::new(PyByteArray::new(borrowed[..pos].to_vec()))
                                as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyByteArray::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyByteArray::new(borrowed[pos + sep_len..].to_vec()))
                                as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyByteArray::new(Vec::new())) as Rc<dyn PyObject>;
                            let head =
                                Rc::new(PyByteArray::new(borrowed.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, empty.clone(), empty])))
                        }
                    }
                },
            ))),
            "pop" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "pop".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: bytearray.pop() takes at most 1 argument".to_string()
                        );
                    }
                    let mut arr = val.borrow_mut();
                    if arr.is_empty() {
                        return Err("IndexError: pop from empty bytearray".to_string());
                    }
                    let idx = if args.is_empty() {
                        arr.len() - 1
                    } else {
                        let n = args[0].as_any().downcast_ref::<PyInt>().ok_or_else(|| {
                            "TypeError: bytearray.pop() index must be int".to_string()
                        })?;
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
                },
            ))),
            "remove" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "remove".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: bytearray.remove() takes exactly one argument".to_string()
                        );
                    }
                    let n = args[0].as_any().downcast_ref::<PyInt>().ok_or_else(|| {
                        "TypeError: bytearray.remove() argument must be int".to_string()
                    })?;
                    let v = n.as_i64().unwrap_or(0) as u8;
                    let mut arr = val.borrow_mut();
                    let pos = arr.iter().position(|&x| x == v);
                    match pos {
                        Some(p) => {
                            arr.remove(p);
                            Ok(Rc::new(PyNone::new()))
                        }
                        None => {
                            Err("ValueError: bytearray.remove(x): x not in bytearray".to_string())
                        }
                    }
                },
            ))),
            "removeprefix" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "removeprefix".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: removeprefix() takes exactly one argument ({} given)"
                                .to_string(),
                        );
                    }
                    let prefix = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    if borrowed.starts_with(&prefix) {
                        Ok(Rc::new(PyByteArray::new(borrowed[prefix.len()..].to_vec())))
                    } else {
                        Ok(Rc::new(PyByteArray::new(borrowed.clone())))
                    }
                },
            ))),
            "removesuffix" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "removesuffix".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: removesuffix() takes exactly one argument ({} given)"
                                .to_string(),
                        );
                    }
                    let suffix = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    if borrowed.ends_with(&suffix) {
                        Ok(Rc::new(PyByteArray::new(
                            borrowed[..borrowed.len() - suffix.len()].to_vec(),
                        )))
                    } else {
                        Ok(Rc::new(PyByteArray::new(borrowed.clone())))
                    }
                },
            ))),
            "replace" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "replace".to_string(),
                move |args| {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(
                            "TypeError: replace() takes 2-3 arguments ({} given)".to_string()
                        );
                    }
                    let old = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let new = if let Some(b) = args[1].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[1]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let count = if args.len() > 2 {
                        args[2]
                            .as_any()
                            .downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64()
                            .unwrap_or(-1)
                    } else {
                        -1
                    };
                    let borrowed = val.borrow();
                    if old.is_empty() {
                        let mut result = Vec::new();
                        let limit = if count < 0 {
                            borrowed.len() + 1
                        } else {
                            (count as usize).min(borrowed.len() + 1)
                        };
                        for i in 0..limit {
                            if i > 0 {
                                result.extend(new.iter());
                            }
                            if i < borrowed.len() {
                                result.push(borrowed[i]);
                            }
                        }
                        if limit < borrowed.len() + 1 {
                            result.extend(borrowed[limit - 1..].iter());
                        }
                        return Ok(Rc::new(PyByteArray::new(result)));
                    }
                    let mut result = Vec::new();
                    let mut i = 0;
                    let mut replacements = 0i64;
                    while i <= borrowed.len() {
                        if (count >= 0 && replacements >= count) || i + old.len() > borrowed.len() {
                            result.extend(borrowed[i..].iter());
                            break;
                        }
                        if borrowed[i..].starts_with(&old) {
                            result.extend(new.iter());
                            i += old.len();
                            replacements += 1;
                        } else {
                            result.push(borrowed[i]);
                            i += 1;
                        }
                    }
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "reverse" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "reverse".to_string(),
                move |args| {
                    if args.len() != 0 {
                        return Err("TypeError: bytearray.reverse() takes no arguments".to_string());
                    }
                    val.borrow_mut().reverse();
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "rfind" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rfind".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err("TypeError: rfind() takes 1-3 arguments ({} given)".to_string());
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64((start + slice.len()) as i64)));
                    }
                    match slice.windows(sub.len()).rposition(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Ok(Rc::new(PyInt::from_i64(-1))),
                    }
                },
            ))),
            "rindex" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rindex".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err(
                            "TypeError: rindex() takes 1-3 arguments ({} given)".to_string()
                        );
                    }
                    let sub = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    if sub.is_empty() {
                        return Ok(Rc::new(PyInt::from_i64((start + slice.len()) as i64)));
                    }
                    match slice.windows(sub.len()).rposition(|w| w == sub.as_slice()) {
                        Some(pos) => Ok(Rc::new(PyInt::from_i64((start + pos) as i64))),
                        None => Err("ValueError: subsection not found".to_string()),
                    }
                },
            ))),
            "rjust" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rjust".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: rjust() takes 1-2 arguments ({} given)".to_string());
                    }
                    let width = args[0]
                        .as_any()
                        .downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize()
                        .unwrap_or(0);
                    let fillbyte =
                        if args.len() > 1 {
                            let fb = args[1].as_any().downcast_ref::<PyByteArray>().ok_or_else(
                                || "TypeError: a bytes-like object is required".to_string(),
                            )?;
                            fb.value.borrow().get(0).copied().unwrap_or(b' ')
                        } else {
                            b' '
                        };
                    let borrowed = val.borrow();
                    if width <= borrowed.len() {
                        Ok(Rc::new(PyByteArray::new(borrowed.clone())))
                    } else {
                        let mut result = vec![fillbyte; width - borrowed.len()];
                        result.extend(borrowed.iter());
                        Ok(Rc::new(PyByteArray::new(result)))
                    }
                },
            ))),
            "rpartition" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rpartition".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: rpartition() takes exactly one argument ({} given)"
                                .to_string(),
                        );
                    }
                    let sep = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".to_string());
                    }
                    let sep_len = sep.len();
                    let borrowed = val.borrow();
                    match borrowed.windows(sep_len).rposition(|w| w == sep.as_slice()) {
                        Some(pos) => {
                            let head = Rc::new(PyByteArray::new(borrowed[..pos].to_vec()))
                                as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyByteArray::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyByteArray::new(borrowed[pos + sep_len..].to_vec()))
                                as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyByteArray::new(Vec::new())) as Rc<dyn PyObject>;
                            let tail =
                                Rc::new(PyByteArray::new(borrowed.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(PyTuple::new(vec![empty.clone(), empty, tail])))
                        }
                    }
                },
            ))),
            "rsplit" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rsplit".to_string(),
                move |args| {
                    if args.len() > 2 {
                        return Err(
                            "TypeError: rsplit() takes at most 2 arguments ({} given)".to_string()
                        );
                    }
                    let sep =
                        if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                            None
                        } else if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                            Some(b.value.borrow().clone())
                        } else if let Some(b) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            Some(b.value.clone())
                        } else {
                            return Err("TypeError: expected a bytes-like object".to_string());
                        };
                    let maxsplit = if args.len() > 1 {
                        args[1]
                            .as_any()
                            .downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64()
                            .unwrap_or(-1)
                    } else {
                        -1
                    };
                    let borrowed = val.borrow();
                    let parts = match sep {
                        None => {
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let limit = if maxsplit < 0 {
                                usize::MAX
                            } else {
                                maxsplit as usize
                            };
                            let mut i = borrowed.len();
                            while i > 0 {
                                while i > 0 && is_whitespace_byte(borrowed[i - 1]) {
                                    i -= 1;
                                }
                                if i == 0 {
                                    break;
                                }
                                let end = i;
                                while i > 0 && !is_whitespace_byte(borrowed[i - 1]) {
                                    i -= 1;
                                }
                                result.push(borrowed[i..end].to_vec());
                                if result.len() >= limit {
                                    break;
                                }
                            }
                            result.reverse();
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
                            let mut remaining = borrowed.clone();
                            let limit = if maxsplit < 0 {
                                usize::MAX
                            } else {
                                maxsplit as usize
                            };
                            for _ in 0..limit {
                                match remaining
                                    .windows(sep_bytes.len())
                                    .rposition(|w| w == sep_bytes.as_slice())
                                {
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
                    let list_items: Vec<Rc<dyn PyObject>> = parts
                        .into_iter()
                        .map(|p| Rc::new(PyByteArray::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                },
            ))),
            "rstrip" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "rstrip".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: rstrip() takes at most 1 argument (2 given)".to_string()
                        );
                    }
                    let chars =
                        if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                            whitespace_bytes().to_vec()
                        } else if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                            b.value.borrow().clone()
                        } else if let Some(b) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            b.value.clone()
                        } else {
                            return Err("TypeError: expected a bytes-like object".to_string());
                        };
                    let borrowed = val.borrow();
                    let mut end = borrowed.len();
                    while end > 0 && chars.contains(&borrowed[end - 1]) {
                        end -= 1;
                    }
                    Ok(Rc::new(PyByteArray::new(borrowed[..end].to_vec())))
                },
            ))),
            "split" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "split".to_string(),
                move |args| {
                    if args.len() > 2 {
                        return Err(
                            "TypeError: split() takes at most 2 arguments ({} given)".to_string()
                        );
                    }
                    let sep =
                        if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                            None
                        } else if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                            Some(b.value.borrow().clone())
                        } else if let Some(b) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            Some(b.value.clone())
                        } else {
                            return Err("TypeError: expected a bytes-like object".to_string());
                        };
                    let maxsplit = if args.len() > 1 {
                        args[1]
                            .as_any()
                            .downcast_ref::<PyInt>()
                            .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                            .as_i64()
                            .unwrap_or(-1)
                    } else {
                        -1
                    };
                    let borrowed = val.borrow();
                    let parts = match sep {
                        None => {
                            let mut result: Vec<Vec<u8>> = Vec::new();
                            let limit = if maxsplit < 0 {
                                usize::MAX
                            } else {
                                maxsplit as usize
                            };
                            let mut i = 0;
                            let mut splits = 0usize;
                            while i < borrowed.len() {
                                while i < borrowed.len() && is_whitespace_byte(borrowed[i]) {
                                    i += 1;
                                }
                                if i >= borrowed.len() {
                                    break;
                                }
                                let start = i;
                                while i < borrowed.len() && !is_whitespace_byte(borrowed[i]) {
                                    i += 1;
                                }
                                result.push(borrowed[start..i].to_vec());
                                splits += 1;
                                if splits >= limit {
                                    while i < borrowed.len() && is_whitespace_byte(borrowed[i]) {
                                        i += 1;
                                    }
                                    if i < borrowed.len() {
                                        result.push(borrowed[i..].to_vec());
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
                            let mut remaining = borrowed.clone();
                            let limit = if maxsplit < 0 {
                                usize::MAX
                            } else {
                                maxsplit as usize
                            };
                            for _ in 0..limit {
                                match remaining
                                    .windows(sep_bytes.len())
                                    .position(|w| w == sep_bytes.as_slice())
                                {
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
                    let list_items: Vec<Rc<dyn PyObject>> = parts
                        .into_iter()
                        .map(|p| Rc::new(PyByteArray::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                },
            ))),
            "splitlines" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "splitlines".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: splitlines() takes at most 1 argument ({} given)"
                                .to_string(),
                        );
                    }
                    let keepends = if args.is_empty() {
                        false
                    } else {
                        args[0]
                            .as_any()
                            .downcast_ref::<PyBool>()
                            .map(|b| b.value)
                            .unwrap_or(false)
                    };
                    let borrowed = val.borrow();
                    let mut result: Vec<Vec<u8>> = Vec::new();
                    let mut i = 0;
                    while i < borrowed.len() {
                        let start = i;
                        while i < borrowed.len() && borrowed[i] != b'\n' && borrowed[i] != b'\r' {
                            i += 1;
                        }
                        if i >= borrowed.len() {
                            result.push(borrowed[start..].to_vec());
                            break;
                        }
                        if borrowed[i] == b'\r'
                            && i + 1 < borrowed.len()
                            && borrowed[i + 1] == b'\n'
                        {
                            if keepends {
                                result.push(borrowed[start..i + 2].to_vec());
                            } else {
                                result.push(borrowed[start..i].to_vec());
                            }
                            i += 2;
                        } else {
                            if keepends {
                                result.push(borrowed[start..i + 1].to_vec());
                            } else {
                                result.push(borrowed[start..i].to_vec());
                            }
                            i += 1;
                        }
                    }
                    let list_items: Vec<Rc<dyn PyObject>> = result
                        .into_iter()
                        .map(|p| Rc::new(PyByteArray::new(p)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(PyList::new(list_items)))
                },
            ))),
            "startswith" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "startswith".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 3 {
                        return Err(
                            "TypeError: startswith() takes 1-3 arguments ({} given)".to_string()
                        );
                    }
                    let prefix = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    let borrowed = val.borrow();
                    let (start, end) = get_start_end_from_borrowed(&borrowed, &args, 1);
                    let slice = &borrowed[start..end];
                    Ok(Rc::new(PyBool::new(slice.starts_with(&prefix))))
                },
            ))),
            "strip" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "strip".to_string(),
                move |args| {
                    if args.len() > 1 {
                        return Err(
                            "TypeError: strip() takes at most 1 argument (2 given)".to_string()
                        );
                    }
                    let chars =
                        if args.is_empty() || args[0].as_any().downcast_ref::<PyNone>().is_some() {
                            whitespace_bytes().to_vec()
                        } else if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                            b.value.borrow().clone()
                        } else if let Some(b) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::bytes::PyBytes>()
                        {
                            b.value.clone()
                        } else {
                            return Err("TypeError: expected a bytes-like object".to_string());
                        };
                    let borrowed = val.borrow();
                    let mut start = 0;
                    while start < borrowed.len() && chars.contains(&borrowed[start]) {
                        start += 1;
                    }
                    let mut end = borrowed.len();
                    while end > start && chars.contains(&borrowed[end - 1]) {
                        end -= 1;
                    }
                    Ok(Rc::new(PyByteArray::new(borrowed[start..end].to_vec())))
                },
            ))),
            "swapcase" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "swapcase".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err(
                            "TypeError: swapcase() takes no arguments (1 given)".to_string()
                        );
                    }
                    let result: Vec<u8> = val
                        .borrow()
                        .iter()
                        .map(|&b| {
                            if b.is_ascii_uppercase() {
                                b + 32
                            } else if b.is_ascii_lowercase() {
                                b - 32
                            } else {
                                b
                            }
                        })
                        .collect();
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "title" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "title".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: title() takes no arguments (1 given)".to_string());
                    }
                    let borrowed = val.borrow();
                    let mut result = Vec::with_capacity(borrowed.len());
                    let mut at_start = true;
                    for &b in borrowed.iter() {
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
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "translate" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "translate".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: translate() takes exactly one argument ({} given)"
                                .to_string(),
                        );
                    }
                    if args[0].as_any().downcast_ref::<PyNone>().is_some() {
                        return Ok(Rc::new(PyByteArray::new(val.borrow().clone())));
                    }
                    let table = if let Some(b) = args[0].as_any().downcast_ref::<PyByteArray>() {
                        b.value.borrow().clone()
                    } else if let Some(b) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::bytes::PyBytes>()
                    {
                        b.value.clone()
                    } else {
                        return Err("TypeError: expected a bytes-like object".to_string());
                    };
                    if table.len() != 256 {
                        return Err(
                            "ValueError: translation table must be 256 bytes long".to_string()
                        );
                    }
                    let borrowed = val.borrow();
                    let result: Vec<u8> = borrowed.iter().map(|&b| table[b as usize]).collect();
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "upper" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "upper".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: upper() takes no arguments (1 given)".to_string());
                    }
                    let result: Vec<u8> = val
                        .borrow()
                        .iter()
                        .map(|&b| if b.is_ascii_lowercase() { b - 32 } else { b })
                        .collect();
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            "zfill" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "zfill".to_string(),
                move |args| {
                    if args.len() != 1 {
                        return Err(
                            "TypeError: zfill() takes exactly one argument ({} given)".to_string()
                        );
                    }
                    let width = args[0]
                        .as_any()
                        .downcast_ref::<PyInt>()
                        .ok_or_else(|| "TypeError: integer argument expected".to_string())?
                        .to_usize()
                        .unwrap_or(0);
                    let borrowed = val.borrow();
                    if width <= borrowed.len() {
                        return Ok(Rc::new(PyByteArray::new(borrowed.clone())));
                    }
                    let sign_prefix =
                        if !borrowed.is_empty() && (borrowed[0] == b'+' || borrowed[0] == b'-') {
                            vec![borrowed[0]]
                        } else {
                            Vec::new()
                        };
                    let padding = width - borrowed.len();
                    let mut result = sign_prefix.clone();
                    result.extend(std::iter::repeat(b'0').take(padding));
                    if !sign_prefix.is_empty() {
                        result.extend(&borrowed[1..]);
                    } else {
                        result.extend(borrowed.iter());
                    }
                    Ok(Rc::new(PyByteArray::new(result)))
                },
            ))),
            _ => Err(format!(
                "AttributeError: 'bytearray' object has no attribute '{}'",
                attr
            )),
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
