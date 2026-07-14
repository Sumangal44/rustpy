use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyDict {
    pub entries: Rc<RefCell<HashMap<u64, Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)>>>>,
}

fn get_hash(key: &Rc<dyn PyObject>) -> Result<u64, String> {
    key.hash().map(|h| h as u64)
}

fn find_in_bucket(bucket: &[(Rc<dyn PyObject>, Rc<dyn PyObject>)], key: &Rc<dyn PyObject>) -> Option<usize> {
    for (i, (k, _)) in bucket.iter().enumerate() {
        if let Some(eq_result) = k.eq(Rc::clone(key)) {
            if eq_result.is_truthy() {
                return Some(i);
            }
        }
    }
    None
}

impl PyDict {
    pub fn new() -> Self {
        Self {
            entries: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn from_pairs(pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)>) -> Self {
        let dict = Self::new();
        for (k, v) in pairs {
            let _ = dict.set_item(k, v);
        }
        dict
    }

    pub fn set_item(&self, key: Rc<dyn PyObject>, value: Rc<dyn PyObject>) -> Result<(), String> {
        let h = get_hash(&key)?;
        let mut entries = self.entries.borrow_mut();
        let bucket = entries.entry(h).or_insert_with(Vec::new);
        if let Some(idx) = find_in_bucket(bucket, &key) {
            bucket[idx] = (key, value);
        } else {
            bucket.push((key, value));
        }
        Ok(())
    }

    pub fn get_item_value(&self, key: &Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        let h = get_hash(key)?;
        let entries = self.entries.borrow();
        if let Some(bucket) = entries.get(&h) {
            if let Some(idx) = find_in_bucket(bucket, key) {
                return Ok(Rc::clone(&bucket[idx].1));
            }
        }
        Err(format!("KeyError: {}", key.repr()))
    }

    pub fn del_item(&self, key: &Rc<dyn PyObject>) -> Result<(), String> {
        let h = get_hash(key)?;
        let mut entries = self.entries.borrow_mut();
        if let Some(bucket) = entries.get_mut(&h) {
            if let Some(idx) = find_in_bucket(bucket, key) {
                bucket.remove(idx);
                if bucket.is_empty() {
                    entries.remove(&h);
                }
                return Ok(());
            }
        }
        Err(format!("KeyError: {}", key.repr()))
    }

    pub fn contains_key(&self, key: &Rc<dyn PyObject>) -> Result<bool, String> {
        let h = get_hash(key)?;
        let entries = self.entries.borrow();
        if let Some(bucket) = entries.get(&h) {
            Ok(find_in_bucket(bucket, key).is_some())
        } else {
            Ok(false)
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
        let mut first = true;
        for bucket in entries.values() {
            for (k, v) in bucket {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&k.repr());
                out.push_str(": ");
                out.push_str(&v.repr());
            }
        }
        out.push('}');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.entries.borrow().is_empty()
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        self.contains_key(&other)
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        self.get_item_value(&key)
    }

    fn bitor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_dict) = other.as_any().downcast_ref::<PyDict>() {
            let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
            for bucket in self.entries.borrow().values() {
                for (k, v) in bucket {
                    pairs.push((Rc::clone(k), Rc::clone(v)));
                }
            }
            for bucket in other_dict.entries.borrow().values() {
                for (k, v) in bucket {
                    pairs.push((Rc::clone(k), Rc::clone(v)));
                }
            }
            Some(Rc::new(PyDict::from_pairs(pairs)))
        } else {
            None
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let entries = Rc::clone(&self.entries);
        match attr {
            "keys" => Ok(Rc::new(PyNativeFunction::new("keys".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.keys() takes no arguments".to_string()); }
                let map = entries.borrow();
                let mut keys: Vec<Rc<dyn PyObject>> = Vec::new();
                for bucket in map.values() {
                    for (k, _) in bucket {
                        keys.push(Rc::clone(k));
                    }
                }
                Ok(Rc::new(crate::objects::list::PyList::new(keys)))
            }))),
            "values" => Ok(Rc::new(PyNativeFunction::new("values".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.values() takes no arguments".to_string()); }
                let map = entries.borrow();
                let mut vals: Vec<Rc<dyn PyObject>> = Vec::new();
                for bucket in map.values() {
                    for (_, v) in bucket {
                        vals.push(Rc::clone(v));
                    }
                }
                Ok(Rc::new(crate::objects::list::PyList::new(vals)))
            }))),
            "items" => Ok(Rc::new(PyNativeFunction::new("items".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: dict.items() takes no arguments".to_string()); }
                let map = entries.borrow();
                let mut items: Vec<Rc<dyn PyObject>> = Vec::new();
                for bucket in map.values() {
                    for (k, v) in bucket {
                        let pair = vec![
                            Rc::clone(k),
                            Rc::clone(v),
                        ];
                        items.push(Rc::new(crate::objects::list::PyList::new(pair)) as Rc<dyn PyObject>);
                    }
                }
                Ok(Rc::new(crate::objects::list::PyList::new(items)))
            }))),
            "get" => Ok(Rc::new(PyNativeFunction::new("get".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: get() takes 1-2 arguments".to_string()); }
                let key = &args[0];
                let h = match get_hash(key) {
                    Ok(h) => h,
                    Err(e) => return Err(e),
                };
                let map = entries.borrow();
                if let Some(bucket) = map.get(&h) {
                    if let Some(idx) = find_in_bucket(bucket, key) {
                        return Ok(Rc::clone(&bucket[idx].1));
                    }
                }
                if args.len() >= 2 {
                    Ok(Rc::clone(&args[1]))
                } else {
                    Ok(Rc::new(crate::objects::none::PyNone::new()))
                }
            }))),
            "pop" => Ok(Rc::new(PyNativeFunction::new("pop".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: pop() takes 1-2 arguments".to_string()); }
                let key = &args[0];
                let h = match get_hash(key) {
                    Ok(h) => h,
                    Err(e) => return Err(e),
                };
                let mut map = entries.borrow_mut();
                if let Some(bucket) = map.get_mut(&h) {
                    if let Some(idx) = find_in_bucket(bucket, key) {
                        let val = Rc::clone(&bucket[idx].1);
                        bucket.remove(idx);
                        if bucket.is_empty() {
                            map.remove(&h);
                        }
                        return Ok(val);
                    }
                }
                if args.len() >= 2 {
                    Ok(Rc::clone(&args[1]))
                } else {
                    Err(format!("KeyError: {}", args[0].repr()))
                }
            }))),
            "popitem" => Ok(Rc::new(PyNativeFunction::new("popitem".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: popitem() takes no arguments".to_string()); }
                let mut map = entries.borrow_mut();
                let h = map.keys().next().cloned();
                match h {
                    Some(hash) => {
                        let bucket = map.get_mut(&hash).unwrap();
                        let (k, v) = bucket.remove(0);
                        if bucket.is_empty() {
                            map.remove(&hash);
                        }
                        let pair = vec![k, v];
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
                    for (h, bucket) in other_map.iter() {
                        let this_bucket = this_map.entry(*h).or_insert_with(Vec::new);
                        for (k, v) in bucket {
                            if let Some(idx) = find_in_bucket(this_bucket, k) {
                                this_bucket[idx] = (Rc::clone(k), Rc::clone(v));
                            } else {
                                this_bucket.push((Rc::clone(k), Rc::clone(v)));
                            }
                        }
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
                let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
                for bucket in map.values() {
                    for (k, v) in bucket {
                        pairs.push((Rc::clone(k), Rc::clone(v)));
                    }
                }
                Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
            }))),
            "setdefault" => Ok(Rc::new(PyNativeFunction::new("setdefault".to_string(), move |args| {
                if args.len() < 1 || args.len() > 2 { return Err("TypeError: setdefault() takes 1-2 arguments".to_string()); }
                let key = &args[0];
                let h = match get_hash(key) {
                    Ok(h) => h,
                    Err(e) => return Err(e),
                };
                let mut map = entries.borrow_mut();
                let bucket = map.entry(h).or_insert_with(Vec::new);
                if let Some(idx) = find_in_bucket(bucket, key) {
                    Ok(Rc::clone(&bucket[idx].1))
                } else {
                    let default = if args.len() >= 2 { Rc::clone(&args[1]) } else { Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject> };
                    bucket.push((Rc::clone(key), Rc::clone(&default)));
                    Ok(default)
                }
            }))),
            _ => Err(format!("AttributeError: 'dict' object has no attribute '{}'", attr)),
        }
    }
}
