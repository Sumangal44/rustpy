use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
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
        format!("'{}'", self.value)
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
                let start = if slice.start.is_some() { raw_start } else { length - 1 };
                let stop = if slice.stop.is_some() { raw_stop } else { 0 };
                let mut i = start;
                loop {
                    result.push(chars[i]);
                    if i == stop { break; }
                    let next = i as i64 + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next as usize;
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
            _ => Err(format!("AttributeError: 'str' object has no attribute '{}'", attr)),
        }
    }
}
