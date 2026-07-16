use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyString {
    pub value: String,
}

impl PyString {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl PyObject for PyString {
    fn get_type(&self) -> &'static str {
        "str"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let mut escaped = String::new();
        for c in self.value.chars() {
            match c {
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\'' => escaped.push_str("\\'"),
                '\\' => escaped.push_str("\\\\"),
                _ => escaped.push(c),
            }
        }
        format!("'{}'", escaped)
    }

    fn str(&self) -> String {
        self.value.clone()
    }

    fn is_truthy(&self) -> bool {
        !self.value.is_empty()
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_str) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(PyString::new(format!(
                "{}{}",
                self.value, other_str.value
            ))))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(n) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let count = n.as_i64().unwrap_or(0);
            if count <= 0 {
                return Some(Rc::new(PyString::new(String::new())));
            }
            let mut result = String::new();
            for _ in 0..count {
                result.push_str(&self.value);
            }
            Some(Rc::new(PyString::new(result)))
        } else {
            None
        }
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value == s.value)))
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

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value != s.value)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value < s.value)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value <= s.value)))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value > s.value)))
        } else {
            None
        }
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Some(Rc::new(crate::objects::bool::PyBool::new(self.value >= s.value)))
        } else {
            None
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        let chars: Vec<Rc<dyn PyObject>> = self.value.chars().map(|c| Rc::new(PyString::new(c.to_string())) as Rc<dyn PyObject>).collect();
        Ok(Rc::new(PyStringIterator::from_vec(chars)))
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        if let Some(s) = other.as_any().downcast_ref::<PyString>() {
            Ok(self.value.contains(&s.value))
        } else {
            Err("TypeError: 'in <string>' requires string as left operand, not '".to_string() + other.get_type() + "'")
        }
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx_obj) = key.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let mut idx = idx_obj.as_i64().unwrap_or(0);
            let len = self.value.len() as i64;
            if idx < 0 {
                idx += len;
            }
            if idx >= 0 && idx < len {
                let c = self.value.chars().nth(idx as usize).unwrap();
                Ok(Rc::new(PyString::new(c.to_string())))
            } else {
                Err("IndexError: string index out of range".to_string())
            }
        } else if let Some(slice) = key.as_any().downcast_ref::<crate::objects::slice::PySlice>() {
            let length = self.value.chars().count();
            let (raw_start, raw_stop, step) = slice.resolve(length);
            let chars: Vec<char> = self.value.chars().collect();
            let mut result = String::new();
            if step > 0 {
                let mut i = raw_start;
                while i < raw_stop {
                    result.push(chars[i]);
                    i = (i as i64 + step) as usize;
                }
            } else if step < 0 {
                let start = if slice.start.is_some() { raw_start as i64 } else { length as i64 - 1 };
                let stop = if slice.stop.is_some() { raw_stop as i64 } else { -1i64 };
                let mut i = start;
                while i > stop {
                    result.push(chars[i as usize]);
                    let next = i + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next;
                }
            }
            Ok(Rc::new(PyString::new(result)))
        } else {
            Err(format!(
                "TypeError: string indices must be integers or slices, not {}",
                key.get_type()
            ))
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "upper" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("upper".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: upper() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(PyString::new(val.to_uppercase())))
                })))
            }
            "lower" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("lower".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: lower() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(PyString::new(val.to_lowercase())))
                })))
            }
            "capitalize" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("capitalize".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: capitalize() takes no arguments (1 given)".to_string()); }
                    let mut c = val.chars();
                    let result = match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    };
                    Ok(Rc::new(PyString::new(result)))
                })))
            }
            "strip" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("strip".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: strip() takes at most 1 argument (2 given)".to_string()); }
                    let s = if args.is_empty() {
                        val.trim().to_string()
                    } else {
                        let chars = args[0].str();
                        val.trim_matches(|c: char| chars.contains(c)).to_string()
                    };
                    Ok(Rc::new(PyString::new(s)))
                })))
            }
            "lstrip" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("lstrip".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: lstrip() takes at most 1 argument (2 given)".to_string()); }
                    let s = if args.is_empty() {
                        val.trim_start().to_string()
                    } else {
                        let chars = args[0].str();
                        val.trim_start_matches(|c: char| chars.contains(c)).to_string()
                    };
                    Ok(Rc::new(PyString::new(s)))
                })))
            }
            "rstrip" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rstrip".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: rstrip() takes at most 1 argument (2 given)".to_string()); }
                    let s = if args.is_empty() {
                        val.trim_end().to_string()
                    } else {
                        let chars = args[0].str();
                        val.trim_end_matches(|c: char| chars.contains(c)).to_string()
                    };
                    Ok(Rc::new(PyString::new(s)))
                })))
            }
            "split" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("split".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: split() takes at most 1 argument (2 given)".to_string()); }
                    let parts: Vec<Rc<dyn PyObject>> = if args.is_empty() {
                        val.split_whitespace().map(|s| Rc::new(PyString::new(s.to_string())) as Rc<dyn PyObject>).collect()
                    } else {
                        let sep = args[0].str();
                        val.split(&sep).map(|s| Rc::new(PyString::new(s.to_string())) as Rc<dyn PyObject>).collect()
                    };
                    Ok(Rc::new(crate::objects::list::PyList::new(parts)))
                })))
            }
            "join" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("join".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: join() takes exactly one argument ({} given)".to_string()); }
                    let iterable = &args[0];
                    let iter = iterable.get_iter()?;
                    let mut result = String::new();
                    let mut first = true;
                    while let Some(item) = iter.get_next()? {
                        if !first { result.push_str(&val); }
                        first = false;
                        result.push_str(&item.str());
                    }
                    Ok(Rc::new(PyString::new(result)))
                })))
            }
            "replace" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("replace".to_string(), move |args| {
                    if args.len() != 2 { return Err("TypeError: replace() takes exactly 2 arguments ({} given)".to_string()); }
                    let old = args[0].str();
                    let new = args[1].str();
                    Ok(Rc::new(PyString::new(val.replace(&old, &new))))
                })))
            }
            "startswith" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("startswith".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: startswith() takes exactly one argument ({} given)".to_string()); }
                    let prefix = args[0].str();
                    Ok(Rc::new(crate::objects::bool::PyBool::new(val.starts_with(&prefix))))
                })))
            }
            "endswith" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("endswith".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: endswith() takes exactly one argument ({} given)".to_string()); }
                    let suffix = args[0].str();
                    Ok(Rc::new(crate::objects::bool::PyBool::new(val.ends_with(&suffix))))
                })))
            }
            "find" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("find".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: find() takes exactly one argument ({} given)".to_string()); }
                    let sub = args[0].str();
                    match val.find(&sub) {
                        Some(pos) => Ok(Rc::new(crate::objects::int::PyInt::from_i64(pos as i64))),
                        None => Ok(Rc::new(crate::objects::int::PyInt::from_i64(-1))),
                    }
                })))
            }
            "index" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("index".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: index() takes exactly one argument ({} given)".to_string()); }
                    let sub = args[0].str();
                    match val.find(&sub) {
                        Some(pos) => Ok(Rc::new(crate::objects::int::PyInt::from_i64(pos as i64))),
                        None => Err("ValueError: substring not found".to_string()),
                    }
                })))
            }
            "count" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("count".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: count() takes exactly one argument ({} given)".to_string()); }
                    let sub = args[0].str();
                    Ok(Rc::new(crate::objects::int::PyInt::from_i64(val.matches(&sub).count() as i64)))
                })))
            }
            "isdigit" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isdigit".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isdigit() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(val.chars().all(|c| c.is_ascii_digit()))))
                })))
            }
            "isalpha" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isalpha".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isalpha() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_alphabetic()))))
                })))
            }
            "isalnum" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isalnum".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isalnum() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_alphanumeric()))))
                })))
            }
            "isspace" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isspace".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isspace() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_whitespace()))))
                })))
            }
            "zfill" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("zfill".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: zfill() takes exactly one argument ({} given)".to_string()); }
                    let width = match args[0].str().parse::<i64>() {
                        Ok(w) => w as usize,
                        Err(_) => return Err("TypeError: integer argument expected".to_string()),
                    };
                    if width <= val.len() {
                        Ok(Rc::new(PyString::new(val.clone())))
                    } else {
                        Ok(Rc::new(PyString::new(format!("{}{}", "0".repeat(width - val.len()), val))))
                    }
                })))
            }
            "ljust" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("ljust".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 { return Err("TypeError: ljust() takes 1-2 arguments ({} given)".to_string()); }
                    let width = match args[0].str().parse::<i64>() {
                        Ok(w) => w as usize,
                        Err(_) => return Err("TypeError: integer argument expected".to_string()),
                    };
                    let fillchar = if args.len() > 1 { args[1].str().chars().next().unwrap_or(' ') } else { ' ' };
                    if width <= val.len() {
                        Ok(Rc::new(PyString::new(val.clone())))
                    } else {
                        Ok(Rc::new(PyString::new(format!("{}{}", val, fillchar.to_string().repeat(width - val.len())))))
                    }
                })))
            }
            "rjust" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rjust".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 { return Err("TypeError: rjust() takes 1-2 arguments ({} given)".to_string()); }
                    let width = match args[0].str().parse::<i64>() {
                        Ok(w) => w as usize,
                        Err(_) => return Err("TypeError: integer argument expected".to_string()),
                    };
                    let fillchar = if args.len() > 1 { args[1].str().chars().next().unwrap_or(' ') } else { ' ' };
                    if width <= val.len() {
                        Ok(Rc::new(PyString::new(val.clone())))
                    } else {
                        Ok(Rc::new(PyString::new(format!("{}{}", fillchar.to_string().repeat(width - val.len()), val))))
                    }
                })))
            }
            "center" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("center".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 2 { return Err("TypeError: center() takes 1-2 arguments ({} given)".to_string()); }
                    let width = match args[0].str().parse::<i64>() {
                        Ok(w) => w as usize,
                        Err(_) => return Err("TypeError: integer argument expected".to_string()),
                    };
                    let fillchar = if args.len() > 1 { args[1].str().chars().next().unwrap_or(' ') } else { ' ' };
                    if width <= val.len() {
                        Ok(Rc::new(PyString::new(val.clone())))
                    } else {
                        let padding = width - val.len();
                        let left = padding / 2;
                        let right = padding - left;
                        Ok(Rc::new(PyString::new(format!("{}{}{}", fillchar.to_string().repeat(left), val, fillchar.to_string().repeat(right)))))
                    }
                })))
            }
            "title" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("title".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: title() takes no arguments (1 given)".to_string()); }
                    let mut result = String::new();
                    let mut at_start = true;
                    for c in val.chars() {
                        if at_start {
                            result.extend(c.to_uppercase());
                        } else {
                            result.extend(c.to_lowercase());
                        }
                        at_start = !c.is_alphanumeric();
                    }
                    Ok(Rc::new(PyString::new(result)))
                })))
            }
            "swapcase" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("swapcase".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: swapcase() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(PyString::new(val.chars().map(|c| if c.is_uppercase() { c.to_lowercase().to_string() } else { c.to_uppercase().to_string() }).collect::<String>())))
                })))
            }
            "casefold" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("casefold".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: casefold() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(PyString::new(val.to_lowercase())))
                })))
            }
            "__iter__" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("__iter__".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: __iter__() takes no arguments".to_string()); }
                    let chars: Vec<Rc<dyn PyObject>> = val.chars().map(|c| Rc::new(PyString::new(c.to_string())) as Rc<dyn PyObject>).collect();
                    Ok(Rc::new(PyStringIterator::from_vec(chars)))
                })))
            }
            "encode" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("encode".to_string(), move |args| {
                    if args.len() > 2 { return Err("TypeError: encode() takes at most 2 arguments ({} given)".to_string()); }
                    let encoding = if args.is_empty() { "utf-8".to_string() } else { args[0].str() };
                    crate::encoding::encode(&val, &encoding).map(|b| Rc::new(crate::objects::bytes::PyBytes::new(b)) as Rc<dyn PyObject>)
                })))
            }
            "expandtabs" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("expandtabs".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: expandtabs() takes at most 1 argument ({} given)".to_string()); }
                    let tabsize = if args.is_empty() {
                        8usize
                    } else {
                        match args[0].str().parse::<usize>() {
                            Ok(t) => t,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        }
                    };
                    let mut result = String::new();
                    let mut col = 0usize;
                    for c in val.chars() {
                        if c == '\t' {
                            let spaces = tabsize - (col % tabsize);
                            result.push_str(&" ".repeat(spaces));
                            col += spaces;
                        } else {
                            result.push(c);
                            col += 1;
                            if c == '\n' || c == '\r' {
                                col = 0;
                            }
                        }
                    }
                    Ok(Rc::new(PyString::new(result)))
                })))
            }
            "isdecimal" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isdecimal".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isdecimal() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_ascii_digit()))))
                })))
            }
            "isidentifier" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isidentifier".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isidentifier() takes no arguments (1 given)".to_string()); }
                    if val.is_empty() { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                    let mut chars = val.chars();
                    let first = chars.next().unwrap();
                    let valid = (first == '_' || first.is_alphabetic()) && chars.all(|c| c == '_' || c.is_alphanumeric());
                    Ok(Rc::new(crate::objects::bool::PyBool::new(valid)))
                })))
            }
            "islower" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("islower".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: islower() takes no arguments (1 given)".to_string()); }
                    let mut has_lower = false;
                    for c in val.chars() {
                        if c.is_uppercase() { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                        if c.is_lowercase() { has_lower = true; }
                    }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(has_lower)))
                })))
            }
            "isnumeric" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isnumeric".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isnumeric() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_numeric()))))
                })))
            }
            "isprintable" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isprintable".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isprintable() takes no arguments (1 given)".to_string()); }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(!val.is_empty() && val.chars().all(|c| c.is_ascii_graphic() || c == ' '))))
                })))
            }
            "istitle" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("istitle".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: istitle() takes no arguments (1 given)".to_string()); }
                    if val.is_empty() { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                    let mut cased = false;
                    let mut prev_cased = false;
                    for c in val.chars() {
                        if c.is_uppercase() {
                            if prev_cased { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                            cased = true;
                            prev_cased = true;
                        } else if c.is_lowercase() {
                            if !prev_cased { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                            cased = true;
                            prev_cased = true;
                        } else {
                            prev_cased = false;
                        }
                    }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(cased)))
                })))
            }
            "isupper" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("isupper".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: isupper() takes no arguments (1 given)".to_string()); }
                    let mut has_upper = false;
                    for c in val.chars() {
                        if c.is_lowercase() { return Ok(Rc::new(crate::objects::bool::PyBool::new(false))); }
                        if c.is_uppercase() { has_upper = true; }
                    }
                    Ok(Rc::new(crate::objects::bool::PyBool::new(has_upper)))
                })))
            }
            "partition" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("partition".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: partition() takes exactly one argument ({} given)".to_string()); }
                    let sep = args[0].str();
                    if sep.is_empty() { return Err("ValueError: empty separator".to_string()); }
                    match val.find(&sep) {
                        Some(pos) => {
                            let sep_str = sep.clone();
                            let head = Rc::new(PyString::new(val[..pos].to_string())) as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyString::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyString::new(val[pos + sep_str.len()..].to_string())) as Rc<dyn PyObject>;
                            Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyString::new(String::new())) as Rc<dyn PyObject>;
                            let head = Rc::new(PyString::new(val.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![head, empty.clone(), empty])))
                        }
                    }
                })))
            }
            "removeprefix" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("removeprefix".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: removeprefix() takes exactly one argument ({} given)".to_string()); }
                    let prefix = args[0].str();
                    if val.starts_with(&prefix) {
                        Ok(Rc::new(PyString::new(val[prefix.len()..].to_string())))
                    } else {
                        Ok(Rc::new(PyString::new(val.clone())))
                    }
                })))
            }
            "removesuffix" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("removesuffix".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: removesuffix() takes exactly one argument ({} given)".to_string()); }
                    let suffix = args[0].str();
                    if val.ends_with(&suffix) {
                        Ok(Rc::new(PyString::new(val[..val.len() - suffix.len()].to_string())))
                    } else {
                        Ok(Rc::new(PyString::new(val.clone())))
                    }
                })))
            }
            "rfind" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rfind".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 { return Err("TypeError: rfind() takes 1-3 arguments ({} given)".to_string()); }
                    let sub = args[0].str();
                    let len = val.len();
                    let start = if args.len() > 1 {
                        let s = match args[1].str().parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        };
                        if s < 0 { (len as i64 + s).max(0) as usize } else { (s as usize).min(len) }
                    } else { 0 };
                    let end = if args.len() > 2 {
                        let e = match args[2].str().parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        };
                        if e < 0 { (len as i64 + e).max(0) as usize } else { (e as usize).min(len) }
                    } else { len };
                    let slice = &val[start..end];
                    if sub.is_empty() { return Ok(Rc::new(crate::objects::int::PyInt::from_i64((start + slice.len()) as i64))); }
                    match slice.rfind(&sub) {
                        Some(pos) => Ok(Rc::new(crate::objects::int::PyInt::from_i64((start + pos) as i64))),
                        None => Ok(Rc::new(crate::objects::int::PyInt::from_i64(-1))),
                    }
                })))
            }
            "rindex" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rindex".to_string(), move |args| {
                    if args.len() < 1 || args.len() > 3 { return Err("TypeError: rindex() takes 1-3 arguments ({} given)".to_string()); }
                    let sub = args[0].str();
                    let len = val.len();
                    let start = if args.len() > 1 {
                        let s = match args[1].str().parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        };
                        if s < 0 { (len as i64 + s).max(0) as usize } else { (s as usize).min(len) }
                    } else { 0 };
                    let end = if args.len() > 2 {
                        let e = match args[2].str().parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        };
                        if e < 0 { (len as i64 + e).max(0) as usize } else { (e as usize).min(len) }
                    } else { len };
                    let slice = &val[start..end];
                    if sub.is_empty() { return Ok(Rc::new(crate::objects::int::PyInt::from_i64((start + slice.len()) as i64))); }
                    match slice.rfind(&sub) {
                        Some(pos) => Ok(Rc::new(crate::objects::int::PyInt::from_i64((start + pos) as i64))),
                        None => Err("ValueError: substring not found".to_string()),
                    }
                })))
            }
            "rpartition" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rpartition".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: rpartition() takes exactly one argument ({} given)".to_string()); }
                    let sep = args[0].str();
                    if sep.is_empty() { return Err("ValueError: empty separator".to_string()); }
                    match val.rfind(&sep) {
                        Some(pos) => {
                            let sep_str = sep.clone();
                            let head = Rc::new(PyString::new(val[..pos].to_string())) as Rc<dyn PyObject>;
                            let sep_obj = Rc::new(PyString::new(sep)) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyString::new(val[pos + sep_str.len()..].to_string())) as Rc<dyn PyObject>;
                            Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![head, sep_obj, tail])))
                        }
                        None => {
                            let empty = Rc::new(PyString::new(String::new())) as Rc<dyn PyObject>;
                            let tail = Rc::new(PyString::new(val.clone())) as Rc<dyn PyObject>;
                            Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![empty.clone(), empty, tail])))
                        }
                    }
                })))
            }
            "rsplit" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("rsplit".to_string(), move |args| {
                    if args.len() > 2 { return Err("TypeError: rsplit() takes at most 2 arguments ({} given)".to_string()); }
                    let sep = if args.is_empty() { None } else { Some(args[0].str()) };
                    let maxsplit = if args.len() > 1 {
                        match args[1].str().parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => return Err("TypeError: integer argument expected".to_string()),
                        }
                    } else { -1 };
                    let parts: Vec<Rc<dyn PyObject>> = match sep {
                        None => {
                            let mut result: Vec<String> = Vec::new();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            let mut i = val.len();
                            while i > 0 {
                                while i > 0 && val.as_bytes()[i - 1].is_ascii_whitespace() { i -= 1; }
                                if i == 0 { break; }
                                let end = i;
                                while i > 0 && !val.as_bytes()[i - 1].is_ascii_whitespace() { i -= 1; }
                                result.push(val[i..end].to_string());
                                if result.len() >= limit { break; }
                            }
                            result.reverse();
                            if result.is_empty() && !val.is_empty() { }
                            if result.is_empty() && val.is_empty() { result.push(String::new()); }
                            result.into_iter().map(|s| Rc::new(PyString::new(s)) as Rc<dyn PyObject>).collect()
                        }
                        Some(ref sep_str) => {
                            if sep_str.is_empty() { return Err("ValueError: empty separator".to_string()); }
                            let mut result: Vec<String> = Vec::new();
                            let mut remaining = val.clone();
                            let limit = if maxsplit < 0 { usize::MAX } else { maxsplit as usize };
                            for _ in 0..limit {
                                match remaining.rfind(sep_str) {
                                    Some(pos) => {
                                        result.push(remaining[pos + sep_str.len()..].to_string());
                                        remaining = remaining[..pos].to_string();
                                    }
                                    None => break,
                                }
                            }
                            result.push(remaining);
                            result.reverse();
                            result.into_iter().map(|s| Rc::new(PyString::new(s)) as Rc<dyn PyObject>).collect()
                        }
                    };
                    Ok(Rc::new(crate::objects::list::PyList::new(parts)))
                })))
            }
            "splitlines" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("splitlines".to_string(), move |args| {
                    if args.len() > 1 { return Err("TypeError: splitlines() takes at most 1 argument ({} given)".to_string()); }
                    let keepends = if args.is_empty() { false } else { args[0].is_truthy() };
                    let mut result: Vec<String> = Vec::new();
                    let mut i = 0;
                    let chars: Vec<char> = val.chars().collect();
                    while i < chars.len() {
                        let start = i;
                        while i < chars.len() && chars[i] != '\n' && chars[i] != '\r' {
                            i += 1;
                        }
                        if i >= chars.len() {
                            result.push(chars[start..].iter().collect());
                            break;
                        }
                        if chars[i] == '\r' && i + 1 < chars.len() && chars[i + 1] == '\n' {
                            if keepends {
                                result.push(chars[start..i + 2].iter().collect());
                            } else {
                                result.push(chars[start..i].iter().collect());
                            }
                            i += 2;
                        } else {
                            if keepends {
                                result.push(chars[start..i + 1].iter().collect());
                            } else {
                                result.push(chars[start..i].iter().collect());
                            }
                            i += 1;
                        }
                    }
                    let list_items: Vec<Rc<dyn PyObject>> = result.into_iter()
                        .map(|s| Rc::new(PyString::new(s)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(crate::objects::list::PyList::new(list_items)))
                })))
            }
            "translate" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("translate".to_string(), move |args| {
                    if args.len() != 1 { return Err("TypeError: translate() takes exactly one argument ({} given)".to_string()); }
                    let table = &args[0];
                    let mut result = String::new();
                    if let Some(d) = table.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                        for c in val.chars() {
                            let key = Rc::new(crate::objects::int::PyInt::from_i64(c as i64)) as Rc<dyn PyObject>;
                            let found = match d.get_item(key) {
                                Ok(val_obj) => {
                                    if val_obj.as_any().downcast_ref::<crate::objects::none::PyNone>().is_some() {
                                        true
                                    } else if let Some(int_obj) = val_obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                                        if let Some(code) = int_obj.as_i64() {
                                            if let Some(ch) = char::from_u32(code as u32) {
                                                result.push(ch);
                                            }
                                        }
                                        true
                                    } else {
                                        result.push_str(&val_obj.str());
                                        true
                                    }
                                }
                                Err(_) => false,
                            };
                            if !found {
                                result.push(c);
                            }
                        }
                    } else {
                        result = val.clone();
                    }
                    Ok(Rc::new(PyString::new(result)))
                })))
            }
            "__format__" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new_pos_only("__format__".to_string(), move |args| {
                    let spec = if args.is_empty() { String::new() } else { args[0].str() };
                    if spec.is_empty() {
                        return Ok(Rc::new(PyString::new(val.clone())));
                    }
                    // Handle alignment specs like <, >, ^, fill char + alignment
                    let bytes = spec.as_bytes();
                    let (fill, align, rest) = if bytes.len() >= 2 && (bytes[1] == b'<' || bytes[1] == b'>' || bytes[1] == b'^') {
                        (bytes[0] as char, bytes[1] as char, &spec[2..])
                    } else if !bytes.is_empty() && (bytes[0] == b'<' || bytes[0] == b'>' || bytes[0] == b'^') {
                        (' ', bytes[0] as char, &spec[1..])
                    } else {
                        (' ', '<', spec.as_str())
                    };
                    if let Ok(width) = rest.parse::<usize>() {
                        let len = val.chars().count();
                        if len >= width {
                            return Ok(Rc::new(PyString::new(val.clone())));
                        }
                        let padding = width - len;
                        let result = match align {
                            '>' => format!("{}{}", fill.to_string().repeat(padding), val),
                            '^' => {
                                let left = padding / 2;
                                let right = padding - left;
                                format!("{}{}{}", fill.to_string().repeat(left), val, fill.to_string().repeat(right))
                            }
                            _ => format!("{}{}", val, fill.to_string().repeat(padding)),
                        };
                        return Ok(Rc::new(PyString::new(result)));
                    }
                    Ok(Rc::new(PyString::new(val.clone())))
                })))
            }
            "format" => {
                let val = self.value.clone();
                Ok(Rc::new(PyNativeFunction::new("format".to_string(), move |args, kwargs| {
                    // Python str.format(*args, **kwargs)
                    // Parse the format string and replace placeholders
                    let mut result = String::new();
                    let mut chars = val.chars().peekable();
                    let mut auto_index: usize = 0;

                    while let Some(c) = chars.next() {
                        if c == '{' {
                            if chars.peek() == Some(&'{') {
                                chars.next();
                                result.push('{');
                                continue;
                            }
                            // Collect field spec up to matching }
                            let mut field = String::new();
                            for fc in chars.by_ref() {
                                if fc == '}' { break; }
                                field.push(fc);
                            }
                            // Split field_name and format_spec
                            let (field_name, fmt_spec) = if let Some(pos) = field.find(':') {
                                (&field[..pos], &field[pos+1..])
                            } else {
                                (field.as_str(), "")
                            };
                            // Split field_name and conversion (!r, !s, !a)
                            let (field_key, conv) = if let Some(pos) = field_name.rfind('!') {
                                (&field_name[..pos], Some(&field_name[pos+1..]))
                            } else {
                                (field_name, None)
                            };
                            // Resolve the value
                            let value_obj: Option<Rc<dyn PyObject>> = if field_key.is_empty() {
                                // Auto-numbering {}
                                let v = args.get(auto_index).cloned();
                                auto_index += 1;
                                v
                            } else if let Ok(idx) = field_key.parse::<usize>() {
                                args.get(idx).cloned()
                            } else {
                                // Handle dotted/bracketed access: "name.attr" or "name[0]"
                                let base_key = field_key.split(|c| c == '.' || c == '[').next().unwrap_or(field_key);
                                kwargs.get(base_key).cloned()
                            };
                            let value_obj = value_obj.ok_or_else(|| format!("IndexError: positional argument out of range or unknown key '{}'", field_key))?;
                            // Apply conversion
                            let converted = match conv {
                                Some("r") => value_obj.repr(),
                                Some("s") => value_obj.str(),
                                Some("a") => value_obj.repr(), // ascii - simplified
                                _ => value_obj.str(),
                            };
                            // Apply format spec
                            if fmt_spec.is_empty() {
                                result.push_str(&converted);
                            } else {
                                // Try __format__ on the object
                                let formatted = if let Ok(fmt_fn) = value_obj.get_attr("__format__") {
                                    if let Some(native) = fmt_fn.as_any().downcast_ref::<PyNativeFunction>() {
                                        let spec_obj = Rc::new(PyString::new(fmt_spec.to_string())) as Rc<dyn PyObject>;
                                        (native.func)(vec![spec_obj], std::collections::HashMap::new())?.str()
                                    } else {
                                        converted
                                    }
                                } else {
                                    converted
                                };
                                result.push_str(&formatted);
                            }
                        } else if c == '}' {
                            if chars.peek() == Some(&'}') {
                                chars.next();
                                result.push('}');
                            }
                        } else {
                            result.push(c);
                        }
                    }
                    Ok(Rc::new(PyString::new(result)) as Rc<dyn PyObject>)
                })))
            }
            _ => Err(format!("AttributeError: 'str' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyStringIterator {
    chars: Rc<Vec<Rc<dyn PyObject>>>,
    index: RefCell<usize>,
}

impl PyStringIterator {
    pub fn from_vec(chars: Vec<Rc<dyn PyObject>>) -> Self {
        Self {
            chars: Rc::new(chars),
            index: RefCell::new(0),
        }
    }
}

impl std::fmt::Debug for PyStringIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyStringIterator {
    fn get_type(&self) -> &'static str {
        "str_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<str_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        if *idx < self.chars.len() {
            let item = Rc::clone(&self.chars[*idx]);
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__next__" => {
                let it = self.clone();
                Ok(Rc::new(crate::objects::native_function::PyNativeFunction::new_pos_only("__next__".to_string(), move |args| {
                    if args.len() != 0 { return Err("TypeError: __next__() takes no arguments".to_string()); }
                    match it.get_next()? {
                        Some(val) => Ok(val),
                        None => Err("StopIteration".to_string()),
                    }
                })))
            }
            _ => Err(format!("AttributeError: '{}' object has no attribute '{}'", self.get_type(), attr)),
        }
    }
}

pub fn format_align_width(val_str: &str, spec: &str, default_align: char) -> Result<String, String> {
    let bytes = spec.as_bytes();
    if bytes.is_empty() {
        return Ok(val_str.to_string());
    }

    let mut fill = ' ';
    let mut align = default_align;
    let mut rest = spec;

    if bytes.len() >= 2 && (bytes[1] == b'<' || bytes[1] == b'>' || bytes[1] == b'^' || bytes[1] == b'=') {
        fill = bytes[0] as char;
        align = bytes[1] as char;
        rest = &spec[2..];
    } else if !bytes.is_empty() && (bytes[0] == b'<' || bytes[0] == b'>' || bytes[0] == b'^' || bytes[0] == b'=') {
        align = bytes[0] as char;
        rest = &spec[1..];
    }

    if let Some(last_byte) = rest.as_bytes().last() {
        let c = *last_byte as char;
        if c.is_alphabetic() {
            rest = &rest[..rest.len() - 1];
        }
    }

    if let Some(dot_idx) = rest.find('.') {
        rest = &rest[..dot_idx];
    }

    let mut zero_pad = false;
    if rest.starts_with('0') {
        zero_pad = true;
        rest = &rest[1..];
    }

    let width = if rest.is_empty() {
        0
    } else if let Ok(w) = rest.parse::<usize>() {
        w
    } else {
        0
    };

    if zero_pad && align == default_align {
        fill = '0';
        align = '>';
    }

    let len = val_str.chars().count();
    if len >= width {
        Ok(val_str.to_string())
    } else {
        let padding = width - len;
        let fill_str = fill.to_string().repeat(padding);
        match align {
            '>' | '=' => Ok(format!("{}{}", fill_str, val_str)),
            '^' => {
                let left = padding / 2;
                let right = padding - left;
                Ok(format!("{}{}{}", fill.to_string().repeat(left), val_str, fill.to_string().repeat(right)))
            }
            _ => Ok(format!("{}{}", val_str, fill_str)),
        }
    }
}

