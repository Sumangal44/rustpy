use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyTuple {
    pub elements: Vec<Rc<dyn PyObject>>,
}

impl PyTuple {
    pub fn new(elements: Vec<Rc<dyn PyObject>>) -> Self {
        Self { elements }
    }
}

impl std::fmt::Debug for PyTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyTuple {
    fn get_type(&self) -> &'static str {
        "tuple"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let mut out = String::new();
        out.push('(');
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&elem.repr());
        }
        if self.elements.len() == 1 {
            out.push(',');
        }
        out.push(')');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.elements.is_empty()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx_obj) = key.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let i = idx_obj.as_i64().unwrap_or(0);
            let idx = if i < 0 {
                (self.elements.len() as i64 + i) as usize
            } else {
                i as usize
            };
            if idx < self.elements.len() {
                Ok(Rc::clone(&self.elements[idx]))
            } else {
                Err("IndexError: tuple index out of range".to_string())
            }
        } else if let Some(slice) = key.as_any().downcast_ref::<crate::objects::slice::PySlice>() {
            let length = self.elements.len();
            let (raw_start, raw_stop, step) = slice.resolve(length);
            let mut result = Vec::new();
            if step > 0 {
                let mut i = raw_start;
                while i < raw_stop {
                    result.push(Rc::clone(&self.elements[i]));
                    i = (i as i64 + step) as usize;
                }
            } else if step < 0 {
                let start = if slice.start.is_some() { raw_start } else { length - 1 };
                let stop = if slice.stop.is_some() { raw_stop } else { 0 };
                let mut i = start;
                loop {
                    result.push(Rc::clone(&self.elements[i]));
                    if i == stop { break; }
                    let next = i as i64 + step;
                    if next < 0 || next as usize >= length { break; }
                    i = next as usize;
                }
            }
            Ok(Rc::new(crate::objects::tuple::PyTuple::new(result)))
        } else {
            Err(format!(
                "TypeError: tuple indices must be integers or slices, not {}",
                key.get_type()
            ))
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyTupleIterator {
            tuple: self.elements.clone(),
            index: RefCell::new(0),
        }))
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        for elem in &self.elements {
            if let Some(eq_result) = elem.eq(Rc::clone(&other)) {
                if eq_result.is_truthy() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            if self.elements.len() != t.elements.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(true)))
        } else {
            None
        }
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            if self.elements.len() != t.elements.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(true)));
            }
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        return Some(Rc::new(crate::objects::bool::PyBool::new(true)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(false)))
        } else {
            None
        }
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        if let Some(lt_result) = a.lt(Rc::clone(b)) {
                            return Some(lt_result);
                        }
                        return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(
                self.elements.len() < t.elements.len()
            )))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        if let Some(lt_result) = a.lt(Rc::clone(b)) {
                            return Some(lt_result);
                        }
                        return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(
                self.elements.len() <= t.elements.len()
            )))
        } else {
            None
        }
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        if let Some(gt_result) = a.gt(Rc::clone(b)) {
                            return Some(gt_result);
                        }
                        return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(
                self.elements.len() > t.elements.len()
            )))
        } else {
            None
        }
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            for (a, b) in self.elements.iter().zip(t.elements.iter()) {
                if let Some(eq_result) = a.eq(Rc::clone(b)) {
                    if !eq_result.is_truthy() {
                        if let Some(gt_result) = a.gt(Rc::clone(b)) {
                            return Some(gt_result);
                        }
                        return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                } else {
                    return None;
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(
                self.elements.len() >= t.elements.len()
            )))
        } else {
            None
        }
    }

    fn hash(&self) -> Result<i64, String> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for elem in &self.elements {
            let elem_hash = elem.hash()?;
            elem_hash.hash(&mut hasher);
        }
        Ok(hasher.finish() as i64)
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(t) = other.as_any().downcast_ref::<PyTuple>() {
            let mut elements = self.elements.clone();
            elements.extend(t.elements.iter().cloned());
            Some(Rc::new(PyTuple::new(elements)))
        } else {
            None
        }
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(n) = other.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let count = n.as_i64().unwrap_or(0);
            if count <= 0 {
                return Some(Rc::new(PyTuple::new(Vec::new())));
            }
            let mut elements = Vec::new();
            for _ in 0..count {
                elements.extend(self.elements.iter().cloned());
            }
            Some(Rc::new(PyTuple::new(elements)))
        } else {
            None
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let elements = self.elements.clone();
        match attr {
            "index" => Ok(Rc::new(PyNativeFunction::new_pos_only("index".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: tuple.index() takes exactly one argument".to_string()); }
                for i in 0..elements.len() {
                    let eq = elements[i].eq(Rc::clone(&args[0]));
                    if let Some(result) = eq {
                        if result.is_truthy() {
                            return Ok(Rc::new(crate::objects::int::PyInt::from_i64(i as i64)));
                        }
                    }
                }
                Err("ValueError: tuple.index(x): x not in tuple".to_string())
            }))),
            "count" => Ok(Rc::new(PyNativeFunction::new_pos_only("count".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: tuple.count() takes exactly one argument".to_string()); }
                let mut cnt = 0i64;
                for i in 0..elements.len() {
                    let eq = elements[i].eq(Rc::clone(&args[0]));
                    if let Some(result) = eq {
                        if result.is_truthy() {
                            cnt += 1;
                        }
                    }
                }
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(cnt)))
            }))),
            _ => Err(format!("AttributeError: 'tuple' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyTupleIterator {
    pub tuple: Vec<Rc<dyn PyObject>>,
    pub index: RefCell<usize>,
}

impl std::fmt::Debug for PyTupleIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyTupleIterator {
    fn get_type(&self) -> &'static str {
        "tuple_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<tuple_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        if *idx < self.tuple.len() {
            let item = Rc::clone(&self.tuple[*idx]);
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
