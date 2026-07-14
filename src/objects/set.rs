use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PySet {
    pub elements: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
}

impl PySet {
    pub fn new(elements: Vec<Rc<dyn PyObject>>) -> Self {
        let mut seen = Vec::new();
        for elem in elements {
            if !Self::has_element(&seen, &elem) {
                seen.push(elem);
            }
        }
        Self {
            elements: Rc::new(RefCell::new(seen)),
        }
    }

    pub fn has_element(vec: &[Rc<dyn PyObject>], elem: &Rc<dyn PyObject>) -> bool {
        for e in vec {
            if let Some(eq) = e.eq(Rc::clone(elem)) {
                if eq.is_truthy() {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_index(vec: &[Rc<dyn PyObject>], elem: &Rc<dyn PyObject>) -> Option<usize> {
        for (i, e) in vec.iter().enumerate() {
            if let Some(eq) = e.eq(Rc::clone(elem)) {
                if eq.is_truthy() {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn set_union(a: &[Rc<dyn PyObject>], b: &[Rc<dyn PyObject>]) -> Vec<Rc<dyn PyObject>> {
        let mut result: Vec<Rc<dyn PyObject>> = a.iter().map(|e| Rc::clone(e)).collect();
        for elem in b {
            if !Self::has_element(&result, elem) {
                result.push(Rc::clone(elem));
            }
        }
        result
    }

    pub fn set_intersection(a: &[Rc<dyn PyObject>], b: &[Rc<dyn PyObject>]) -> Vec<Rc<dyn PyObject>> {
        let mut result = Vec::new();
        for elem in a {
            if Self::has_element(b, elem) {
                result.push(Rc::clone(elem));
            }
        }
        result
    }

    pub fn set_difference(a: &[Rc<dyn PyObject>], b: &[Rc<dyn PyObject>]) -> Vec<Rc<dyn PyObject>> {
        let mut result = Vec::new();
        for elem in a {
            if !Self::has_element(b, elem) {
                result.push(Rc::clone(elem));
            }
        }
        result
    }

    pub fn set_symmetric_difference(a: &[Rc<dyn PyObject>], b: &[Rc<dyn PyObject>]) -> Vec<Rc<dyn PyObject>> {
        let mut result = Vec::new();
        for elem in a {
            if !Self::has_element(b, elem) {
                result.push(Rc::clone(elem));
            }
        }
        for elem in b {
            if !Self::has_element(a, elem) {
                result.push(Rc::clone(elem));
            }
        }
        result
    }
}

impl std::fmt::Debug for PySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PySet {
    fn get_type(&self) -> &'static str {
        "set"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let elements = self.elements.borrow();
        let mut out = String::new();
        out.push('{');
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&elem.repr());
        }
        out.push('}');
        out
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        !self.elements.borrow().is_empty()
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        let elements = self.elements.borrow();
        Ok(Self::has_element(&elements, &other))
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PySetIterator {
            elements: Rc::clone(&self.elements),
            index: RefCell::new(0),
        }))
    }

    fn bitor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_union(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_union(&a, &b))))
        } else {
            None
        }
    }

    fn bitand(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_intersection(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_intersection(&a, &b))))
        } else {
            None
        }
    }

    fn bitxor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_symmetric_difference(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_symmetric_difference(&a, &b))))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_difference(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PySet::new(Self::set_difference(&a, &b))))
        } else {
            None
        }
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            if a.len() != b.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            for elem in a.iter() {
                if !Self::has_element(&b, elem) {
                    return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(true)))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            if a.len() != b.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            for elem in a.iter() {
                if !Self::has_element(&b, elem) {
                    return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(true)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let b: Vec<Rc<dyn PyObject>> = if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            s.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            fs.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else {
            return None;
        };
        let a = self.elements.borrow();
        for elem in a.iter() {
            if !Self::has_element(&b, elem) {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
        }
        Some(Rc::new(crate::objects::bool::PyBool::new(true)))
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let b: Vec<Rc<dyn PyObject>> = if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            s.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            fs.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else {
            return None;
        };
        let a = self.elements.borrow();
        if a.len() >= b.len() {
            return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
        }
        for elem in a.iter() {
            if !Self::has_element(&b, elem) {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
        }
        Some(Rc::new(crate::objects::bool::PyBool::new(true)))
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        other.le(Rc::new(self.clone()))
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        other.lt(Rc::new(self.clone()))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let elements = Rc::clone(&self.elements);
        match attr {
            "add" => Ok(Rc::new(PyNativeFunction::new_pos_only("add".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.add() takes exactly one argument".to_string()); }
                let mut arr = elements.borrow_mut();
                if !PySet::has_element(&arr, &args[0]) {
                    arr.push(Rc::clone(&args[0]));
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "remove" => Ok(Rc::new(PyNativeFunction::new_pos_only("remove".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.remove() takes exactly one argument".to_string()); }
                let mut arr = elements.borrow_mut();
                if let Some(idx) = PySet::find_index(&arr, &args[0]) {
                    arr.remove(idx);
                    Ok(Rc::new(crate::objects::none::PyNone::new()))
                } else {
                    Err("KeyError: element not found".to_string())
                }
            }))),
            "discard" => Ok(Rc::new(PyNativeFunction::new_pos_only("discard".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.discard() takes exactly one argument".to_string()); }
                let mut arr = elements.borrow_mut();
                if let Some(idx) = PySet::find_index(&arr, &args[0]) {
                    arr.remove(idx);
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "pop" => Ok(Rc::new(PyNativeFunction::new_pos_only("pop".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: set.pop() takes no arguments".to_string()); }
                let mut arr = elements.borrow_mut();
                if arr.is_empty() {
                    Err("KeyError: pop from an empty set".to_string())
                } else {
                    Ok(arr.remove(0))
                }
            }))),
            "clear" => Ok(Rc::new(PyNativeFunction::new_pos_only("clear".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: set.clear() takes no arguments".to_string()); }
                elements.borrow_mut().clear();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "copy" => Ok(Rc::new(PyNativeFunction::new_pos_only("copy".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: set.copy() takes no arguments".to_string()); }
                let arr = elements.borrow();
                let copied: Vec<Rc<dyn PyObject>> = arr.iter().map(|e| Rc::clone(e)).collect();
                Ok(Rc::new(PySet::new(copied)))
            }))),
            "union" => Ok(Rc::new(PyNativeFunction::new_pos_only("union".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.union() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_union(&arr, &other_elements);
                Ok(Rc::new(PySet::new(result)))
            }))),
            "intersection" => Ok(Rc::new(PyNativeFunction::new_pos_only("intersection".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.intersection() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_intersection(&arr, &other_elements);
                Ok(Rc::new(PySet::new(result)))
            }))),
            "difference" => Ok(Rc::new(PyNativeFunction::new_pos_only("difference".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.difference() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_difference(&arr, &other_elements);
                Ok(Rc::new(PySet::new(result)))
            }))),
            "symmetric_difference" => Ok(Rc::new(PyNativeFunction::new_pos_only("symmetric_difference".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.symmetric_difference() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_symmetric_difference(&arr, &other_elements);
                Ok(Rc::new(PySet::new(result)))
            }))),
            "update" => Ok(Rc::new(PyNativeFunction::new_pos_only("update".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.update() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let mut arr = elements.borrow_mut();
                for elem in other_elements {
                    if !PySet::has_element(&arr, &elem) {
                        arr.push(elem);
                    }
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "intersection_update" => Ok(Rc::new(PyNativeFunction::new_pos_only("intersection_update".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.intersection_update() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let arr_ref = elements.borrow();
                let result = PySet::set_intersection(&arr_ref, &other_elements);
                drop(arr_ref);
                *elements.borrow_mut() = result;
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "difference_update" => Ok(Rc::new(PyNativeFunction::new_pos_only("difference_update".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.difference_update() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let arr_ref = elements.borrow();
                let result = PySet::set_difference(&arr_ref, &other_elements);
                drop(arr_ref);
                *elements.borrow_mut() = result;
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "symmetric_difference_update" => Ok(Rc::new(PyNativeFunction::new_pos_only("symmetric_difference_update".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.symmetric_difference_update() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let arr_ref = elements.borrow();
                let result = PySet::set_symmetric_difference(&arr_ref, &other_elements);
                drop(arr_ref);
                *elements.borrow_mut() = result;
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "isdisjoint" => Ok(Rc::new(PyNativeFunction::new_pos_only("isdisjoint".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.isdisjoint() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                for elem in arr.iter() {
                    if PySet::has_element(&other_elements, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            "issubset" => Ok(Rc::new(PyNativeFunction::new_pos_only("issubset".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.issubset() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                for elem in arr.iter() {
                    if !PySet::has_element(&other_elements, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            "issuperset" => Ok(Rc::new(PyNativeFunction::new_pos_only("issuperset".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: set.issuperset() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let arr = elements.borrow();
                for elem in other_elements.iter() {
                    if !PySet::has_element(&arr, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            _ => Err(format!("AttributeError: 'set' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyFrozenSet {
    pub elements: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
}

impl PyFrozenSet {
    pub fn new(elements: Vec<Rc<dyn PyObject>>) -> Self {
        let mut seen = Vec::new();
        for elem in elements {
            if !PySet::has_element(&seen, &elem) {
                seen.push(elem);
            }
        }
        Self {
            elements: Rc::new(RefCell::new(seen)),
        }
    }
}

impl std::fmt::Debug for PyFrozenSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyFrozenSet {
    fn get_type(&self) -> &'static str {
        "frozenset"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let elements = self.elements.borrow();
        let mut out = String::new();
        out.push_str("frozenset({");
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&elem.repr());
        }
        out.push_str("})");
        out
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn is_truthy(&self) -> bool {
        !self.elements.borrow().is_empty()
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        let elements = self.elements.borrow();
        Ok(PySet::has_element(&elements, &other))
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PySetIterator {
            elements: Rc::clone(&self.elements),
            index: RefCell::new(0),
        }))
    }

    fn hash(&self) -> Result<i64, String> {
        let mut h: i64 = 0;
        let elements = self.elements.borrow();
        let mut hashes: Vec<i64> = elements.iter().map(|e| e.hash()).collect::<Result<Vec<i64>, String>>()?;
        hashes.sort_unstable();
        for hash_val in hashes {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64 as i64).wrapping_add(hash_val);
        }
        Ok(h)
    }

    fn bitor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(PySet::set_union(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PyFrozenSet::new(PySet::set_union(&a, &b))))
        } else {
            None
        }
    }

    fn bitand(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(PySet::set_intersection(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PyFrozenSet::new(PySet::set_intersection(&a, &b))))
        } else {
            None
        }
    }

    fn bitxor(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PySet::new(PySet::set_symmetric_difference(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PyFrozenSet::new(PySet::set_symmetric_difference(&a, &b))))
        } else {
            None
        }
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            Some(Rc::new(PyFrozenSet::new(PySet::set_difference(&a, &b))))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            Some(Rc::new(PyFrozenSet::new(PySet::set_difference(&a, &b))))
        } else {
            None
        }
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            let a = self.elements.borrow();
            let b = s.elements.borrow();
            if a.len() != b.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            for elem in a.iter() {
                if !PySet::has_element(&b, elem) {
                    return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(true)))
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            let a = self.elements.borrow();
            let b = fs.elements.borrow();
            if a.len() != b.len() {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            for elem in a.iter() {
                if !PySet::has_element(&b, elem) {
                    return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Some(Rc::new(crate::objects::bool::PyBool::new(true)))
        } else {
            None
        }
    }

    fn le(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let b: Vec<Rc<dyn PyObject>> = if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            s.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            fs.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else {
            return None;
        };
        let a = self.elements.borrow();
        for elem in a.iter() {
            if !PySet::has_element(&b, elem) {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
        }
        Some(Rc::new(crate::objects::bool::PyBool::new(true)))
    }

    fn lt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        let b: Vec<Rc<dyn PyObject>> = if let Some(s) = other.as_any().downcast_ref::<PySet>() {
            s.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else if let Some(fs) = other.as_any().downcast_ref::<PyFrozenSet>() {
            fs.elements.borrow().iter().map(|e| Rc::clone(e)).collect()
        } else {
            return None;
        };
        let a = self.elements.borrow();
        if a.len() >= b.len() {
            return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
        }
        for elem in a.iter() {
            if !PySet::has_element(&b, elem) {
                return Some(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
        }
        Some(Rc::new(crate::objects::bool::PyBool::new(true)))
    }

    fn ge(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        other.le(Rc::new(self.clone()))
    }

    fn gt(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        other.lt(Rc::new(self.clone()))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let elements = Rc::clone(&self.elements);
        match attr {
            "copy" => Ok(Rc::new(PyNativeFunction::new_pos_only("copy".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: frozenset.copy() takes no arguments".to_string()); }
                let arr = elements.borrow();
                let copied: Vec<Rc<dyn PyObject>> = arr.iter().map(|e| Rc::clone(e)).collect();
                Ok(Rc::new(PyFrozenSet::new(copied)))
            }))),
            "union" => Ok(Rc::new(PyNativeFunction::new_pos_only("union".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.union() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_union(&arr, &other_elements);
                Ok(Rc::new(PyFrozenSet::new(result)))
            }))),
            "intersection" => Ok(Rc::new(PyNativeFunction::new_pos_only("intersection".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.intersection() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_intersection(&arr, &other_elements);
                Ok(Rc::new(PyFrozenSet::new(result)))
            }))),
            "difference" => Ok(Rc::new(PyNativeFunction::new_pos_only("difference".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.difference() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_difference(&arr, &other_elements);
                Ok(Rc::new(PyFrozenSet::new(result)))
            }))),
            "symmetric_difference" => Ok(Rc::new(PyNativeFunction::new_pos_only("symmetric_difference".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.symmetric_difference() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                let result = PySet::set_symmetric_difference(&arr, &other_elements);
                Ok(Rc::new(PyFrozenSet::new(result)))
            }))),
            "isdisjoint" => Ok(Rc::new(PyNativeFunction::new_pos_only("isdisjoint".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.isdisjoint() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                for elem in arr.iter() {
                    if PySet::has_element(&other_elements, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            "issubset" => Ok(Rc::new(PyNativeFunction::new_pos_only("issubset".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.issubset() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let other_elements = set_elements_from_arg(&args[0])?;
                for elem in arr.iter() {
                    if !PySet::has_element(&other_elements, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            "issuperset" => Ok(Rc::new(PyNativeFunction::new_pos_only("issuperset".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: frozenset.issuperset() takes exactly one argument".to_string()); }
                let other_elements = set_elements_from_arg(&args[0])?;
                let arr = elements.borrow();
                for elem in other_elements.iter() {
                    if !PySet::has_element(&arr, elem) {
                        return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                    }
                }
                Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
            }))),
            _ => Err(format!("AttributeError: 'frozenset' object has no attribute '{}'", attr)),
        }
    }
}

fn set_elements_from_arg(arg: &Rc<dyn PyObject>) -> Result<Vec<Rc<dyn PyObject>>, String> {
    if let Some(s) = arg.as_any().downcast_ref::<PySet>() {
        Ok(s.elements.borrow().iter().map(|e| Rc::clone(e)).collect())
    } else if let Some(fs) = arg.as_any().downcast_ref::<PyFrozenSet>() {
        Ok(fs.elements.borrow().iter().map(|e| Rc::clone(e)).collect())
    } else {
        let iter = arg.get_iter()?;
        let mut result = Vec::new();
        while let Some(item) = iter.get_next()? {
            result.push(item);
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub struct PySetIterator {
    pub elements: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
    pub index: RefCell<usize>,
}

impl std::fmt::Debug for PySetIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PySetIterator {
    fn get_type(&self) -> &'static str {
        "set_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<set_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        let list = self.elements.borrow();
        if *idx < list.len() {
            let item = Rc::clone(&list[*idx]);
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
