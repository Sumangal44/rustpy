use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyClass {
    pub name: String,
    pub attributes: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
    #[allow(dead_code)]
    pub bases: Vec<Rc<dyn PyObject>>,
    pub mro: Vec<Rc<dyn PyObject>>, // MRO excluding self
    /// If Some, this class uses __slots__ and only these attrs are allowed
    pub slots: Option<Vec<String>>,
}

fn is_same_class(a: &Rc<dyn PyObject>, b: &Rc<dyn PyObject>) -> bool {
    let a_name = if let Some(a_class) = a.as_any().downcast_ref::<PyClass>() {
        a_class.name.clone()
    } else if let Some(a_type) = a.as_any().downcast_ref::<crate::objects::typeobj::PyType>() {
        a_type.name.clone()
    } else if let Some(a_nf) = a.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
        a_nf.name.clone()
    } else {
        a.repr()
    };
    let b_name = if let Some(b_class) = b.as_any().downcast_ref::<PyClass>() {
        b_class.name.clone()
    } else if let Some(b_type) = b.as_any().downcast_ref::<crate::objects::typeobj::PyType>() {
        b_type.name.clone()
    } else if let Some(b_nf) = b.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
        b_nf.name.clone()
    } else {
        b.repr()
    };
    a_name == b_name
}

fn c3_merge(mut lists: Vec<Vec<Rc<dyn PyObject>>>) -> Result<Vec<Rc<dyn PyObject>>, String> {
    let mut result = Vec::new();
    loop {
        lists.retain(|l| !l.is_empty());
        if lists.is_empty() {
            break;
        }

        let mut next_class = None;
        for seq in &lists {
            let head = Rc::clone(&seq[0]);
            let mut good_head = true;
            for other_seq in &lists {
                if other_seq.iter().skip(1).any(|c| is_same_class(c, &head)) {
                    good_head = false;
                    break;
                }
            }
            if good_head {
                next_class = Some(head);
                break;
            }
        }

        if let Some(head) = next_class {
            result.push(Rc::clone(&head));
            for seq in &mut lists {
                if is_same_class(&seq[0], &head) {
                    seq.remove(0);
                }
            }
        } else {
            return Err("TypeError: Cannot create a consistent method resolution order (MRO)".to_string());
        }
    }
    Ok(result)
}

impl PyClass {
    pub fn new(name: String, attributes: HashMap<String, Rc<dyn PyObject>>, bases: Vec<Rc<dyn PyObject>>) -> Result<Self, String> {
        // Calculate MRO for bases
        let mut lists = Vec::new();
        for base in &bases {
            let mut mro = vec![Rc::clone(base)];
            if let Some(base_class) = base.as_any().downcast_ref::<PyClass>() {
                mro.extend(base_class.mro.iter().cloned());
            }
            lists.push(mro);
        }
        if !bases.is_empty() {
            lists.push(bases.clone());
        }

        let mro = if lists.is_empty() {
            Vec::new()
        } else {
            c3_merge(lists)?
        };

        // Extract __slots__ if defined
        let slots = attributes.get("__slots__").and_then(|slots_obj| {
            // slots can be a list, tuple, or string
            if let Some(s) = slots_obj.as_any().downcast_ref::<crate::objects::string::PyString>() {
                Some(vec![s.value.clone()])
            } else {
                let mut names = Vec::new();
                let iter = slots_obj.get_iter().ok()?;
                loop {
                    match iter.get_next() {
                        Ok(Some(item)) => names.push(item.str()),
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                if names.is_empty() { None } else { Some(names) }
            }
        });

        Ok(Self {
            name,
            attributes: Rc::new(RefCell::new(attributes)),
            bases,
            mro,
            slots,
        })
    }
}

impl std::fmt::Debug for PyClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyClass {
    fn get_type(&self) -> &'static str {
        "type"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<class '{}'>", self.name)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        if attr == "__name__" {
            return Ok(Rc::new(crate::objects::string::PyString::new(self.name.clone())));
        }

        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            if let Some(sm) = val.as_any().downcast_ref::<crate::objects::staticmethod::PyStaticMethod>() {
                return Ok(Rc::clone(&sm.func));
            }
            if let Some(_cm) = val.as_any().downcast_ref::<crate::objects::classmethod::PyClassMethod>() {
                let cm = val.as_any().downcast_ref::<crate::objects::classmethod::PyClassMethod>().unwrap();
                let cls_rc: Rc<dyn PyObject> = Rc::new(self.clone()) as Rc<dyn PyObject>;
                return Ok(Rc::new(crate::objects::bound_method::PyBoundMethod::new(cls_rc, Rc::clone(&cm.func))));
            }
            return Ok(Rc::clone(val));
        }
        
        for base in &self.mro {
            if let Some(base_class) = base.as_any().downcast_ref::<PyClass>() {
                let base_attrs = base_class.attributes.borrow();
                if let Some(val) = base_attrs.get(attr) {
                    if let Some(sm) = val.as_any().downcast_ref::<crate::objects::staticmethod::PyStaticMethod>() {
                        return Ok(Rc::clone(&sm.func));
                    }
                    if let Some(_cm) = val.as_any().downcast_ref::<crate::objects::classmethod::PyClassMethod>() {
                        let cm = val.as_any().downcast_ref::<crate::objects::classmethod::PyClassMethod>().unwrap();
                        let cls_rc: Rc<dyn PyObject> = Rc::new(base_class.clone()) as Rc<dyn PyObject>;
                        return Ok(Rc::new(crate::objects::bound_method::PyBoundMethod::new(cls_rc, Rc::clone(&cm.func))));
                    }
                    return Ok(Rc::clone(val));
                }
            } else {
                if let Ok(val) = base.get_attr(attr) {
                    return Ok(val);
                }
            }
        }

        Err(format!(
            "AttributeError: type object '{}' has no attribute '{}'",
            self.name, attr
        ))
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        self.attributes.borrow_mut().insert(attr.to_string(), value);
        Ok(())
    }

    fn del_attr(&self, name: &str) -> Result<(), String> {
        self.attributes.borrow_mut().remove(name);
        Ok(())
    }
}

pub struct PySuper {
    pub type_obj: Rc<PyClass>,
    pub obj: Rc<crate::objects::instance::PyInstance>,
}

impl PySuper {
    pub fn new(type_obj: Rc<PyClass>, obj: Rc<crate::objects::instance::PyInstance>) -> Self {
        Self { type_obj, obj }
    }
}

impl std::fmt::Debug for PySuper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PySuper {
    fn get_type(&self) -> &'static str {
        "super"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<super: <class '{}'>, <{} object>>", self.type_obj.name, self.obj.class.name)
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let class_rc: Rc<dyn PyObject> = self.obj.class.clone();
        let mut full_mro = vec![class_rc];
        full_mro.extend(self.obj.class.mro.iter().cloned());

        let mut start_idx = None;
        for (i, cls) in full_mro.iter().enumerate() {
            if is_same_class(cls, &(self.type_obj.clone() as Rc<dyn PyObject>)) {
                start_idx = Some(i + 1);
                break;
            }
        }

        if let Some(idx) = start_idx {
            for cls in full_mro.iter().skip(idx) {
                if let Some(pyclass) = cls.as_any().downcast_ref::<PyClass>() {
                    let attrs = pyclass.attributes.borrow();
                    if let Some(val) = attrs.get(attr) {
                        // Bind to the original object!
                        if val.as_any().is::<crate::objects::function::PyFunction>()
                            || val.as_any().is::<crate::objects::native_function::PyNativeFunction>()
                        {
                            let bound = crate::objects::bound_method::PyBoundMethod::new(Rc::new(self.obj.as_ref().clone()) as Rc<dyn PyObject>, Rc::clone(val));
                            return Ok(Rc::new(bound));
                        }
                        return Ok(Rc::clone(val));
                    }
                } else {
                    if let Ok(val) = cls.get_attr(attr) {
                        if val.as_any().is::<crate::objects::function::PyFunction>()
                            || val.as_any().is::<crate::objects::native_function::PyNativeFunction>()
                        {
                            let bound = crate::objects::bound_method::PyBoundMethod::new(Rc::new(self.obj.as_ref().clone()) as Rc<dyn PyObject>, Rc::clone(&val));
                            return Ok(Rc::new(bound));
                        }
                        return Ok(val);
                    }
                }
            }
        }

        Err(format!("AttributeError: 'super' object has no attribute '{}'", attr))
    }
}
