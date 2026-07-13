use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyDict {
    // We restrict keys to strings for simplicity in Phase 9
    pub entries: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
}

impl PyDict {
    pub fn new(entries: HashMap<String, Rc<dyn PyObject>>) -> Self {
        Self {
            entries: Rc::new(RefCell::new(entries)),
        }
    }
}

impl std::fmt::Debug for PyDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyDict {
    fn get_type(&self) -> &'static str {
        "dict"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let entries = self.entries.borrow();
        let mut out = String::new();
        out.push('{');
        for (i, (k, v)) in entries.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("'{}': {}", k, v.repr()));
        }
        out.push('}');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.entries.borrow().is_empty()
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        let key = other.str();
        let entries = self.entries.borrow();
        Ok(entries.contains_key(&key))
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(str_key) = key
            .as_any()
            .downcast_ref::<crate::objects::string::PyString>()
        {
            let entries = self.entries.borrow();
            if let Some(val) = entries.get(&str_key.value) {
                Ok(Rc::clone(val))
            } else {
                Err(format!("KeyError: '{}'", str_key.value))
            }
        } else {
            Err(format!(
                "TypeError: unhashable type: '{}' (Only strings supported as keys currently)",
                key.get_type()
            ))
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let entries = Rc::clone(&self.entries);
        match attr {
            "keys" => Ok(Rc::new(PyNativeFunction::new("keys".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.keys() takes no arguments".to_string()); }
                let map = entries.borrow();
                let keys: Vec<Rc<dyn PyObject>> = map.keys().map(|k| Rc::new(crate::objects::string::PyString::new(k.clone())) as Rc<dyn PyObject>).collect();
                Ok(Rc::new(crate::objects::list::PyList::new(keys)))
            }))),
            "values" => Ok(Rc::new(PyNativeFunction::new("values".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.values() takes no arguments".to_string()); }
                let map = entries.borrow();
                let vals: Vec<Rc<dyn PyObject>> = map.values().map(|v| Rc::clone(v)).collect();
                Ok(Rc::new(crate::objects::list::PyList::new(vals)))
            }))),
            "items" => Ok(Rc::new(PyNativeFunction::new("items".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.items() takes no arguments".to_string()); }
                let map = entries.borrow();
                let items: Vec<Rc<dyn PyObject>> = map.iter().map(|(k, v)| {
                    let pair = vec![
                        Rc::new(crate::objects::string::PyString::new(k.clone())) as Rc<dyn PyObject>,
                        Rc::clone(v),
                    ];
                    Rc::new(crate::objects::list::PyList::new(pair)) as Rc<dyn PyObject>
                }).collect();
                Ok(Rc::new(crate::objects::list::PyList::new(items)))
            }))),
            "get" => Ok(Rc::new(PyNativeFunction::new("get".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: get() takes 1-2 arguments".to_string()); }
                let key_str = args[0].str();
                let map = entries.borrow();
                if let Some(val) = map.get(&key_str) {
                    Ok(Rc::clone(val))
                } else if args.len() >= 2 {
                    Ok(Rc::clone(&args[1]))
                } else {
                    Ok(Rc::new(crate::objects::none::PyNone::new()))
                }
            }))),
            "pop" => Ok(Rc::new(PyNativeFunction::new("pop".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: pop() takes 1-2 arguments".to_string()); }
                let key_str = args[0].str();
                let mut map = entries.borrow_mut();
                if let Some(val) = map.remove(&key_str) {
                    Ok(val)
                } else if args.len() >= 2 {
                    Ok(Rc::clone(&args[1]))
                } else {
                    Err(format!("KeyError: '{}'", key_str))
                }
            }))),
            "popitem" => Ok(Rc::new(PyNativeFunction::new("popitem".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: popitem() takes no arguments".to_string()); }
                let mut map = entries.borrow_mut();
                let key = map.keys().next().cloned();
                match key {
                    Some(k) => {
                        let v = map.remove(&k).unwrap();
                        let pair = vec![
                            Rc::new(crate::objects::string::PyString::new(k)) as Rc<dyn PyObject>,
                            v,
                        ];
                        Ok(Rc::new(crate::objects::list::PyList::new(pair)) as Rc<dyn PyObject>)
                    }
                    None => Err("KeyError: 'popitem(): dictionary is empty'".to_string()),
                }
            }))),
            "update" => Ok(Rc::new(PyNativeFunction::new("update".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: update() takes exactly one argument".to_string()); }
                if let Some(other) = args[0].as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                    let other_map = other.entries.borrow();
                    let mut this_map = entries.borrow_mut();
                    for (k, v) in other_map.iter() {
                        this_map.insert(k.clone(), Rc::clone(v));
                    }
                } else {
                    return Err("TypeError: update() argument must be a dict".to_string());
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "clear" => Ok(Rc::new(PyNativeFunction::new("clear".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.clear() takes no arguments".to_string()); }
                entries.borrow_mut().clear();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "copy" => Ok(Rc::new(PyNativeFunction::new("copy".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.copy() takes no arguments".to_string()); }
                let map = entries.borrow();
                let copied: HashMap<String, Rc<dyn PyObject>> = map.iter().map(|(k, v)| (k.clone(), Rc::clone(v))).collect();
                Ok(Rc::new(crate::objects::dict::PyDict::new(copied)))
            }))),
            "setdefault" => Ok(Rc::new(PyNativeFunction::new("setdefault".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: setdefault() takes 1-2 arguments".to_string()); }
                let key_str = args[0].str();
                let mut map = entries.borrow_mut();
                if let Some(val) = map.get(&key_str) {
                    Ok(Rc::clone(val))
                } else {
                    let default = if args.len() >= 2 { Rc::clone(&args[1]) } else { Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject> };
                    map.insert(key_str, Rc::clone(&default));
                    Ok(default)
                }
            }))),
            _ => Err(format!("AttributeError: 'dict' object has no attribute '{}'", attr)),
        }
    }
}
