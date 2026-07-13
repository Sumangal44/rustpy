use super::PyObject;
use crate::objects::native_function::PyNativeFunction;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyList {
    pub elements: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
}

impl PyList {
    pub fn new(elements: Vec<Rc<dyn PyObject>>) -> Self {
        Self {
            elements: Rc::new(RefCell::new(elements)),
        }
    }
}

impl std::fmt::Debug for PyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyList {
    fn get_type(&self) -> &'static str {
        "list"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let elements = self.elements.borrow();
        let mut out = String::new();
        out.push('[');
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&elem.repr());
        }
        out.push(']');
        out
    }

    fn is_truthy(&self) -> bool {
        !self.elements.borrow().is_empty()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx_obj) = key.as_any().downcast_ref::<crate::objects::int::PyInt>() {
            let elements = self.elements.borrow();
            let mut idx = idx_obj.value;
            if idx < 0 {
                idx += elements.len() as i64;
            }
            if idx >= 0 && (idx as usize) < elements.len() {
                Ok(Rc::clone(&elements[idx as usize]))
            } else {
                Err("IndexError: list index out of range".to_string())
            }
        } else if let Some(slice) = key.as_any().downcast_ref::<crate::objects::slice::PySlice>() {
            let elements = self.elements.borrow();
            let length = elements.len();
            let (raw_start, raw_stop, step) = slice.resolve(length);
            let mut result = Vec::new();
            if step > 0 {
                let mut i = raw_start;
                while i < raw_stop {
                    result.push(Rc::clone(&elements[i]));
                    i = (i as i64 + step) as usize;
                }
            } else if step < 0 {
                let start = if slice.start.is_some() { raw_start } else { length - 1 };
                let stop = if slice.stop.is_some() { raw_stop } else { 0 };
                let mut i = start;
                loop {
                    result.push(Rc::clone(&elements[i]));
                    if i == stop { break; }
                    let next = (i as i64 + step);
                    if next < 0 || next as usize >= length { break; }
                    i = next as usize;
                }
            }
            Ok(Rc::new(crate::objects::list::PyList::new(result)))
        } else {
            Err(format!(
                "TypeError: list indices must be integers or slices, not {}",
                key.get_type()
            ))
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyListIterator {
            list: Rc::clone(&self.elements),
            index: RefCell::new(0),
        }))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let elements = Rc::clone(&self.elements);
        match attr {
            "append" => Ok(Rc::new(PyNativeFunction::new("append".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: list.append() takes exactly one argument".to_string()); }
                elements.borrow_mut().push(Rc::clone(&args[0]));
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "pop" => Ok(Rc::new(PyNativeFunction::new("pop".to_string(), move |args| {
                if args.len() > 1 { return Err("TypeError: pop() takes at most 1 argument (2 given)".to_string()); }
                let mut arr = elements.borrow_mut();
                let idx = if args.is_empty() {
                    if arr.is_empty() { return Err("IndexError: pop from empty list".to_string()); }
                    arr.len() - 1
                } else {
                    let i = match args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        Some(n) => n.value,
                        None => return Err("TypeError: 'int' object expected".to_string()),
                    };
                    if i < 0 {
                        if (-i as usize) > arr.len() { return Err("IndexError: pop index out of range".to_string()); }
                        (arr.len() as i64 + i) as usize
                    } else {
                        if (i as usize) >= arr.len() { return Err("IndexError: pop index out of range".to_string()); }
                        i as usize
                    }
                };
                Ok(arr.remove(idx))
            }))),
            "insert" => Ok(Rc::new(PyNativeFunction::new("insert".to_string(), move |args| {
                if args.len() != 2 { return Err("TypeError: list.insert() takes exactly 2 arguments".to_string()); }
                let idx = match args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Some(n) => n.value,
                    None => return Err("TypeError: 'int' object expected".to_string()),
                };
                let mut arr = elements.borrow_mut();
                let len = arr.len() as i64;
                let pos = if idx < 0 { 0.max(len + idx) as usize } else { (idx as usize).min(len as usize) };
                arr.insert(pos, Rc::clone(&args[1]));
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "remove" => Ok(Rc::new(PyNativeFunction::new("remove".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: list.remove() takes exactly one argument".to_string()); }
                let mut arr = elements.borrow_mut();
                for i in 0..arr.len() {
                    let eq = arr[i].eq(Rc::clone(&args[0]));
                    if let Some(result) = eq {
                        if result.is_truthy() {
                            arr.remove(i);
                            return Ok(Rc::new(crate::objects::none::PyNone::new()));
                        }
                    }
                }
                Err("ValueError: list.remove(x): x not in list".to_string())
            }))),
            "index" => Ok(Rc::new(PyNativeFunction::new("index".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: list.index() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                for i in 0..arr.len() {
                    let eq = arr[i].eq(Rc::clone(&args[0]));
                    if let Some(result) = eq {
                        if result.is_truthy() {
                            return Ok(Rc::new(crate::objects::int::PyInt::new(i as i64)));
                        }
                    }
                }
                Err("ValueError: list.index(x): x not in list".to_string())
            }))),
            "count" => Ok(Rc::new(PyNativeFunction::new("count".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: list.count() takes exactly one argument".to_string()); }
                let arr = elements.borrow();
                let mut cnt = 0i64;
                for i in 0..arr.len() {
                    let eq = arr[i].eq(Rc::clone(&args[0]));
                    if let Some(result) = eq {
                        if result.is_truthy() {
                            cnt += 1;
                        }
                    }
                }
                Ok(Rc::new(crate::objects::int::PyInt::new(cnt)))
            }))),
            "sort" => Ok(Rc::new(PyNativeFunction::new("sort".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: list.sort() takes no arguments".to_string()); }
                let mut arr = elements.borrow_mut();
                let n = arr.len();
                for i in 0..n {
                    for j in 0..n-1-i {
                        let do_swap = {
                            let a = Rc::clone(&arr[j]);
                            let b = Rc::clone(&arr[j+1]);
                            match a.lt(b) {
                                Some(result) => !result.is_truthy(),
                                None => false,
                            }
                        };
                        if do_swap {
                            arr.swap(j, j+1);
                        }
                    }
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "reverse" => Ok(Rc::new(PyNativeFunction::new("reverse".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: list.reverse() takes no arguments".to_string()); }
                let mut arr = elements.borrow_mut();
                arr.reverse();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "extend" => Ok(Rc::new(PyNativeFunction::new("extend".to_string(), move |args| {
                if args.len() != 1 { return Err("TypeError: list.extend() takes exactly one argument".to_string()); }
                let iter = args[0].get_iter()?;
                let mut arr = elements.borrow_mut();
                while let Some(item) = iter.get_next()? {
                    arr.push(item);
                }
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "clear" => Ok(Rc::new(PyNativeFunction::new("clear".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: list.clear() takes no arguments".to_string()); }
                elements.borrow_mut().clear();
                Ok(Rc::new(crate::objects::none::PyNone::new()))
            }))),
            "copy" => Ok(Rc::new(PyNativeFunction::new("copy".to_string(), move |args| {
                if args.len() != 0 { return Err("TypeError: list.copy() takes no arguments".to_string()); }
                let arr = elements.borrow();
                let copied: Vec<Rc<dyn PyObject>> = arr.iter().map(|e| Rc::clone(e)).collect();
                Ok(Rc::new(crate::objects::list::PyList::new(copied)))
            }))),
            _ => Err(format!("AttributeError: 'list' object has no attribute '{}'", attr)),
        }
    }
}

#[derive(Clone)]
pub struct PyListIterator {
    pub list: Rc<RefCell<Vec<Rc<dyn PyObject>>>>,
    pub index: RefCell<usize>,
}

impl std::fmt::Debug for PyListIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyListIterator {
    fn get_type(&self) -> &'static str {
        "list_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<list_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut idx = self.index.borrow_mut();
        let list = self.list.borrow();
        if *idx < list.len() {
            let item = Rc::clone(&list[*idx]);
            *idx += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
