use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyDictKeyIterator {
    pub ordered_keys: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
    pub index: RefCell<usize>,
}

impl std::fmt::Debug for PyDictKeyIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyDictKeyIterator {
    fn get_type(&self) -> &'static str {
        "dict_keyiterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<dict_keyiterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        let ord = self.ordered_keys.borrow();
        if *idx < ord.len() {
            let key = Rc::clone(&ord[*idx]);
            *idx += 1;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        match attr {
            "__next__" => {
                let it = self.clone();
                Ok(Rc::new(
                    crate::objects::native_function::PyNativeFunction::new_pos_only(
                        "__next__".to_string(),
                        move |args| {
                            if args.len() != 0 {
                                return Err("TypeError: __next__() takes no arguments".to_string());
                            }
                            match it.get_next()? {
                                Some(val) => Ok(val),
                                None => Err("StopIteration".to_string()),
                            }
                        },
                    ),
                ))
            }
            _ => Err(format!(
                "AttributeError: '{}' object has no attribute '{}'",
                self.get_type(),
                attr
            )),
        }
    }
}

#[derive(Clone)]
pub struct PyDict {
    pub entries: Rc<RefCell<HashMap<u64, Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)>>>>,
    pub ordered_keys: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
}

pub fn get_hash(key: &Rc<dyn PyObject>) -> Result<u64, String> {
    key.hash().map(|h| h as u64)
}

pub fn find_in_bucket(
    bucket: &[(Rc<dyn PyObject>, Rc<dyn PyObject>)],
    key: &Rc<dyn PyObject>,
) -> Option<usize> {
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
            ordered_keys: Rc::new(RefCell::new(Vec::new())),
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
            bucket[idx] = (Rc::clone(&key), value);
        } else {
            bucket.push((Rc::clone(&key), value));
            self.ordered_keys.borrow_mut().push(key);
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

    #[allow(dead_code)]
    pub fn del_item(&self, key: &Rc<dyn PyObject>) -> Result<(), String> {
        let h = get_hash(key)?;
        let mut entries = self.entries.borrow_mut();
        if let Some(bucket) = entries.get_mut(&h) {
            if let Some(idx) = find_in_bucket(bucket, key) {
                bucket.remove(idx);
                if bucket.is_empty() {
                    entries.remove(&h);
                }
                let mut ord = self.ordered_keys.borrow_mut();
                let key_for_ord = Rc::clone(key);
                if let Some(pos) = ord.iter().position(|k| {
                    if let Some(eq) = k.eq(Rc::clone(&key_for_ord)) {
                        eq.is_truthy()
                    } else {
                        false
                    }
                }) {
                    ord.remove(pos);
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
        let ord = self.ordered_keys.borrow();
        let entries = self.entries.borrow();
        let mut out = String::new();
        out.push('{');
        let mut first = true;
        for k in ord.iter() {
            if !first {
                out.push_str(", ");
            }
            first = false;
            out.push_str(&k.repr());
            out.push_str(": ");
            if let Ok(h) = get_hash(k) {
                if let Some(bucket) = entries.get(&h) {
                    if let Some(idx) = find_in_bucket(bucket, k) {
                        out.push_str(&bucket[idx].1.repr());
                    }
                }
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

    fn del_item(&self, key: Rc<dyn PyObject>) -> Result<(), String> {
        self.del_item(&key)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyDictKeyIterator {
            ordered_keys: Rc::clone(&self.ordered_keys),
            index: RefCell::new(0),
        }))
    }

    fn bitor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(other_dict) = other.as_any().downcast_ref::<PyDict>() {
            let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
            let ord = self.ordered_keys.borrow();
            let entries = self.entries.borrow();
            for k in ord.iter() {
                if let Ok(h) = get_hash(k) {
                    if let Some(bucket) = entries.get(&h) {
                        if let Some(idx) = find_in_bucket(bucket, k) {
                            pairs.push((Rc::clone(k), Rc::clone(&bucket[idx].1)));
                        }
                    }
                }
            }
            drop(ord);
            drop(entries);
            let other_ord = other_dict.ordered_keys.borrow();
            let other_entries = other_dict.entries.borrow();
            for k in other_ord.iter() {
                if let Ok(h) = get_hash(k) {
                    if let Some(bucket) = other_entries.get(&h) {
                        if let Some(idx) = find_in_bucket(bucket, k) {
                            pairs.push((Rc::clone(k), Rc::clone(&bucket[idx].1)));
                        }
                    }
                }
            }
            Some(Rc::new(PyDict::from_pairs(pairs)))
        } else {
            None
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let entries = Rc::clone(&self.entries);
        let ordered_keys = Rc::clone(&self.ordered_keys);
        match attr {
            "keys" => {
                let ok = Rc::clone(&ordered_keys);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "keys".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: dict.keys() takes no arguments".to_string());
                        }
                        let ord = ok.borrow();
                        let mut keys: Vec<Rc<dyn PyObject>> = Vec::new();
                        for k in ord.iter() {
                            keys.push(Rc::clone(k));
                        }
                        Ok(Rc::new(crate::objects::list::PyList::new(keys)))
                    },
                )))
            }
            "values" => {
                let ok = Rc::clone(&ordered_keys);
                let ent = Rc::clone(&entries);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "values".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: dict.values() takes no arguments".to_string());
                        }
                        let ord = ok.borrow();
                        let map = ent.borrow();
                        let mut vals: Vec<Rc<dyn PyObject>> = Vec::new();
                        for k in ord.iter() {
                            if let Ok(h) = get_hash(k) {
                                if let Some(bucket) = map.get(&h) {
                                    if let Some(idx) = find_in_bucket(bucket, k) {
                                        vals.push(Rc::clone(&bucket[idx].1));
                                    }
                                }
                            }
                        }
                        Ok(Rc::new(crate::objects::list::PyList::new(vals)))
                    },
                )))
            }
            "items" => {
                let ok = Rc::clone(&ordered_keys);
                let ent = Rc::clone(&entries);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "items".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: dict.items() takes no arguments".to_string());
                        }
                        let ord = ok.borrow();
                        let map = ent.borrow();
                        let mut items: Vec<Rc<dyn PyObject>> = Vec::new();
                        for k in ord.iter() {
                            if let Ok(h) = get_hash(k) {
                                if let Some(bucket) = map.get(&h) {
                                    if let Some(idx) = find_in_bucket(bucket, k) {
                                        let pair = vec![Rc::clone(k), Rc::clone(&bucket[idx].1)];
                                        items.push(Rc::new(crate::objects::tuple::PyTuple::new(
                                            pair,
                                        ))
                                            as Rc<dyn PyObject>);
                                    }
                                }
                            }
                        }
                        Ok(Rc::new(crate::objects::list::PyList::new(items)))
                    },
                )))
            }
            "get" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "get".to_string(),
                move |args| {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("TypeError: get() takes 1-2 arguments".to_string());
                    }
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
                },
            ))),
            "pop" => {
                let ok = Rc::clone(&ordered_keys);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "pop".to_string(),
                    move |args| {
                        if args.len() < 1 || args.len() > 2 {
                            return Err("TypeError: pop() takes 1-2 arguments".to_string());
                        }
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
                                let mut ord = ok.borrow_mut();
                                let key_for_ord = Rc::clone(key);
                                if let Some(pos) = ord.iter().position(|k| {
                                    if let Some(eq) = k.eq(Rc::clone(&key_for_ord)) {
                                        eq.is_truthy()
                                    } else {
                                        false
                                    }
                                }) {
                                    ord.remove(pos);
                                }
                                return Ok(val);
                            }
                        }
                        if args.len() >= 2 {
                            Ok(Rc::clone(&args[1]))
                        } else {
                            Err(format!("KeyError: {}", args[0].repr()))
                        }
                    },
                )))
            }
            "popitem" => {
                let ok = Rc::clone(&ordered_keys);
                let ent = Rc::clone(&entries);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "popitem".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: popitem() takes no arguments".to_string());
                        }
                        let mut ord = ok.borrow_mut();
                        if ord.is_empty() {
                            return Err("KeyError: 'popitem(): dictionary is empty'".to_string());
                        }
                        let last_idx = ord.len() - 1;
                        let k = ord.remove(last_idx);
                        let h = get_hash(&k).unwrap();
                        let mut map = ent.borrow_mut();
                        if let Some(bucket) = map.get_mut(&h) {
                            if let Some(idx) = find_in_bucket(bucket, &k) {
                                let v = Rc::clone(&bucket[idx].1);
                                bucket.remove(idx);
                                if bucket.is_empty() {
                                    map.remove(&h);
                                }
                                let pair = vec![k, v];
                                return Ok(Rc::new(crate::objects::tuple::PyTuple::new(pair))
                                    as Rc<dyn PyObject>);
                            }
                        }
                        Err("KeyError: 'popitem(): dictionary is empty'".to_string())
                    },
                )))
            }
            "update" => {
                let ok = Rc::clone(&ordered_keys);
                let ent = Rc::clone(&entries);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "update".to_string(),
                    move |args| {
                        if args.len() != 1 {
                            return Err(
                                "TypeError: update() takes exactly one argument".to_string()
                            );
                        }
                        if let Some(other) = args[0]
                            .as_any()
                            .downcast_ref::<crate::objects::dict::PyDict>()
                        {
                            let other_ord = other.ordered_keys.borrow();
                            let other_entries = other.entries.borrow();
                            let mut this_ord = ok.borrow_mut();
                            let mut this_map = ent.borrow_mut();
                            for k in other_ord.iter() {
                                if let Ok(h) = get_hash(k) {
                                    if let Some(bucket) = other_entries.get(&h) {
                                        if let Some(idx) = find_in_bucket(bucket, k) {
                                            let val = Rc::clone(&bucket[idx].1);
                                            let this_bucket =
                                                this_map.entry(h).or_insert_with(Vec::new);
                                            if let Some(existing_idx) =
                                                find_in_bucket(this_bucket, k)
                                            {
                                                this_bucket[existing_idx] = (Rc::clone(k), val);
                                            } else {
                                                this_bucket.push((Rc::clone(k), val));
                                                this_ord.push(Rc::clone(k));
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            return Err("TypeError: update() argument must be a dict".to_string());
                        }
                        Ok(Rc::new(crate::objects::none::PyNone::new()))
                    },
                )))
            }
            "clear" => {
                let ok = Rc::clone(&ordered_keys);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "clear".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: dict.clear() takes no arguments".to_string());
                        }
                        entries.borrow_mut().clear();
                        ok.borrow_mut().clear();
                        Ok(Rc::new(crate::objects::none::PyNone::new()))
                    },
                )))
            }
            "copy" => {
                let ok = Rc::clone(&ordered_keys);
                let ent = Rc::clone(&entries);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "copy".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: dict.copy() takes no arguments".to_string());
                        }
                        let ord = ok.borrow();
                        let map = ent.borrow();
                        let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
                        for k in ord.iter() {
                            if let Ok(h) = get_hash(k) {
                                if let Some(bucket) = map.get(&h) {
                                    if let Some(idx) = find_in_bucket(bucket, k) {
                                        pairs.push((Rc::clone(k), Rc::clone(&bucket[idx].1)));
                                    }
                                }
                            }
                        }
                        Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
                    },
                )))
            }
            "fromkeys" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "fromkeys".to_string(),
                move |args| {
                    if args.is_empty() {
                        return Err(
                            "TypeError: fromkeys() takes at least 1 argument (0 given)".to_string()
                        );
                    }
                    let iterable = Rc::clone(&args[0]);
                    let value = if args.len() > 1 {
                        Rc::clone(&args[1])
                    } else {
                        Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>
                    };
                    let it = iterable.get_iter()?;
                    let mut pairs = Vec::new();
                    while let Some(key) = it.get_next()? {
                        pairs.push((key, Rc::clone(&value)));
                    }
                    Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
                },
            ))),
            "setdefault" => {
                let ok = Rc::clone(&ordered_keys);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "setdefault".to_string(),
                    move |args| {
                        if args.len() < 1 || args.len() > 2 {
                            return Err("TypeError: setdefault() takes 1-2 arguments".to_string());
                        }
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
                            let default = if args.len() >= 2 {
                                Rc::clone(&args[1])
                            } else {
                                Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>
                            };
                            bucket.push((Rc::clone(key), Rc::clone(&default)));
                            ok.borrow_mut().push(Rc::clone(key));
                            Ok(default)
                        }
                    },
                )))
            }
            "__iter__" => {
                let ok = Rc::clone(&ordered_keys);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "__iter__".to_string(),
                    move |args| {
                        if args.len() != 0 {
                            return Err("TypeError: __iter__() takes no arguments".to_string());
                        }
                        Ok(Rc::new(PyDictKeyIterator {
                            ordered_keys: Rc::clone(&ok),
                            index: RefCell::new(0),
                        }) as Rc<dyn PyObject>)
                    },
                )))
            }
            _ => Err(format!(
                "AttributeError: 'dict' object has no attribute '{}'",
                attr
            )),
        }
    }
}
