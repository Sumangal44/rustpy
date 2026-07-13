use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyClass {
    pub name: String,
    pub attributes: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
    pub bases: Vec<Rc<dyn PyObject>>,
    pub mro: Vec<Rc<dyn PyObject>>, // MRO excluding self
}

fn is_same_class(a: &Rc<dyn PyObject>, b: &Rc<dyn PyObject>) -> bool {
    let a_class = a.as_any().downcast_ref::<PyClass>().unwrap();
    let b_class = b.as_any().downcast_ref::<PyClass>().unwrap();
    a_class.name == b_class.name
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
            let base_class = base.as_any().downcast_ref::<PyClass>().unwrap();
            let mut mro = vec![Rc::clone(base)];
            mro.extend(base_class.mro.iter().cloned());
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

        Ok(Self {
            name,
            attributes: Rc::new(RefCell::new(attributes)),
            bases,
            mro,
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
        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            return Ok(Rc::clone(val));
        }
        
        for base in &self.mro {
            let base_class = base.as_any().downcast_ref::<PyClass>().unwrap();
            let base_attrs = base_class.attributes.borrow();
            if let Some(val) = base_attrs.get(attr) {
                return Ok(Rc::clone(val));
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
        let mro = &self.obj.class.mro;
        
        // Find index of type_obj in the obj.class's MRO
        // Wait, MRO in PyClass only includes bases, not the class itself!
        // We need the full MRO including the class itself.
        // Let's create a full MRO on the fly or just check the class first.
        let mut full_mro = vec![Rc::new(self.obj.class.as_ref().clone()) as Rc<dyn PyObject>]; // This is inefficient but works for now
        // wait, obj.class is Rc<PyClass>. We can just cast it.
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
                let pyclass = cls.as_any().downcast_ref::<PyClass>().unwrap();
                let attrs = pyclass.attributes.borrow();
                if let Some(val) = attrs.get(attr) {
                    // Bind to the original object!
                    if val.as_any().is::<crate::objects::function::PyFunction>()
                        || val.as_any().is::<crate::objects::native_function::PyNativeFunction>()
                    {
                        let bound = crate::objects::bound_method::PyBoundMethod::new(self.obj.as_ref().clone(), Rc::clone(val));
                        return Ok(Rc::new(bound));
                    }
                    return Ok(Rc::clone(val));
                }
            }
        }

        Err(format!("AttributeError: 'super' object has no attribute '{}'", attr))
    }
}
