use super::bound_method::PyBoundMethod;
use super::class::PyClass;
use super::classmethod::PyClassMethod;
use super::function::PyFunction;
use super::int::PyInt;
use super::native_function::PyNativeFunction;
use super::property::PyProperty;
use super::staticmethod::PyStaticMethod;
use super::string::PyString;
use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static VM_PTR: RefCell<Option<*mut ()>> = const { RefCell::new(None) };
}

pub fn set_vm_ptr(vm_ptr: *mut ()) {
    VM_PTR.with(|cell| {
        // Use try_borrow_mut to avoid panicking on re-entrant calls.
        if let Ok(mut ptr) = cell.try_borrow_mut() {
            if ptr.is_none() {
                *ptr = Some(vm_ptr);
            }
        }
    });
}

fn call_vm_func(
    func_obj: Rc<dyn PyObject>,
    args: Vec<Rc<dyn PyObject>>,
    kwargs: std::collections::HashMap<String, Rc<dyn PyObject>>,
) -> Result<Rc<dyn PyObject>, String> {
    VM_PTR.with(|cell| {
        if let Ok(ptr_cell) = cell.try_borrow() {
            if let Some(vm_ptr) = *ptr_cell {
                let vm = unsafe { &mut *(vm_ptr as *mut crate::vm::VirtualMachine) };
                return vm.invoke(func_obj, args, kwargs);
            }
        }
        Err("RuntimeError: no VM function caller available".to_string())
    })
}

#[derive(Clone)]
pub struct PyInstance {
    pub class: Rc<PyClass>,
    pub attributes: Rc<RefCell<HashMap<String, Rc<dyn PyObject>>>>,
}

impl PyInstance {
    pub fn new(class: Rc<PyClass>) -> Self {
        Self {
            class,
            attributes: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn bind_function(&self, func: Rc<dyn PyObject>) -> Rc<dyn PyObject> {
        let bound = PyBoundMethod::new(Rc::new(self.clone()) as Rc<dyn PyObject>, Rc::clone(&func));
        Rc::new(bound)
    }

    fn resolve_class_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let val = self.class.get_attr(attr)?;

        if let Some(prop) = val.as_any().downcast_ref::<PyProperty>() {
            return prop.call_getter(Rc::new(self.clone()) as Rc<dyn PyObject>);
        }
        if let Some(_sm) = val.as_any().downcast_ref::<PyStaticMethod>() {
            let sm = val.as_any().downcast_ref::<PyStaticMethod>().unwrap();
            return Ok(Rc::clone(&sm.func));
        }
        if let Some(_cm) = val.as_any().downcast_ref::<PyClassMethod>() {
            let cm = val.as_any().downcast_ref::<PyClassMethod>().unwrap();
            let cls_rc: Rc<dyn PyObject> = Rc::clone(&self.class) as Rc<dyn PyObject>;
            return Ok(Rc::new(PyBoundMethod::new(cls_rc, Rc::clone(&cm.func))));
        }

        if val.as_any().is::<crate::objects::function::PyFunction>()
            || val
                .as_any()
                .is::<crate::objects::native_function::PyNativeFunction>()
        {
            return Ok(self.bind_function(val));
        }

        Ok(val)
    }

    pub fn call_dunder(
        &self,
        name: &str,
        args: Vec<Rc<dyn PyObject>>,
    ) -> Result<Rc<dyn PyObject>, String> {
        let method = self.get_attr(name)?;
        if let Some(bound) = method.as_any().downcast_ref::<PyBoundMethod>() {
            if let Some(native) = bound.func.as_any().downcast_ref::<PyNativeFunction>() {
                let mut all_args = vec![Rc::clone(&bound.instance)];
                all_args.extend(args);
                return (native.func)(all_args, std::collections::HashMap::new());
            }
            if let Some(_func) = bound.func.as_any().downcast_ref::<PyFunction>() {
                let mut all_args = vec![Rc::clone(&bound.instance)];
                all_args.extend(args);
                return call_vm_func(
                    Rc::clone(&bound.func),
                    all_args,
                    std::collections::HashMap::new(),
                );
            }
            return Err(format!(
                "NotImplementedError: calling {} on user-defined function not supported",
                name
            ));
        } else if let Some(native) = method.as_any().downcast_ref::<PyNativeFunction>() {
            return (native.func)(args, std::collections::HashMap::new());
        }
        Err(format!(
            "TypeError: '{}' object is not callable",
            method.get_type()
        ))
    }

    pub fn len(&self) -> Result<usize, String> {
        let result = self.call_dunder("__len__", vec![])?;
        if let Some(i) = result.as_any().downcast_ref::<PyInt>() {
            Ok(i.to_usize().unwrap_or(0))
        } else {
            Err("TypeError: __len__ must return an int".to_string())
        }
    }
}

impl std::fmt::Debug for PyInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyInstance {
    fn get_type(&self) -> &'static str {
        "instance"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if let Ok(result) = self.call_dunder("__repr__", vec![]) {
            return result.str();
        }
        format!("<{} object at {:p}>", self.class.name, self)
    }

    fn str(&self) -> String {
        if let Ok(result) = self.call_dunder("__str__", vec![]) {
            return result.str();
        }
        self.repr()
    }

    fn hash(&self) -> Result<i64, String> {
        if let Ok(result) = self.call_dunder("__hash__", vec![]) {
            if let Some(i) = result.as_any().downcast_ref::<PyInt>() {
                return i
                    .as_i64()
                    .ok_or_else(|| "TypeError: __hash__ returned non-integer".to_string());
            }
            return Err("TypeError: __hash__ returned non-integer".to_string());
        }
        Err("TypeError: unhashable type: 'instance'".to_string())
    }

    fn is_truthy(&self) -> bool {
        if let Ok(result) = self.call_dunder("__bool__", vec![]) {
            return result.is_truthy();
        }
        if let Ok(result) = self.call_dunder("__len__", vec![]) {
            if let Some(i) = result.as_any().downcast_ref::<PyInt>() {
                return i.as_i64().unwrap_or(0) != 0;
            }
        }
        true
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        if let Ok(result) = self.call_dunder("__contains__", vec![Rc::clone(&other)]) {
            Ok(result.is_truthy())
        } else {
            let iter = self.get_iter()?;
            while let Some(item) = iter.get_next()? {
                if let Some(eq_result) = item.eq(Rc::clone(&other)) {
                    if eq_result.is_truthy() {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        if attr == "__dict__" {
            let attrs = self.attributes.borrow();
            let mut pairs = Vec::new();
            for (k, v) in attrs.iter() {
                pairs.push((
                    Rc::new(PyString::new(k.clone())) as Rc<dyn PyObject>,
                    Rc::clone(v),
                ));
            }
            return Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)));
        }

        if attr == "__class__" {
            return Ok(Rc::clone(&self.class) as Rc<dyn PyObject>);
        }

        let attrs = self.attributes.borrow();
        if let Some(val) = attrs.get(attr) {
            return Ok(Rc::clone(val));
        }

        // Check class and its MRO with descriptor protocol
        self.resolve_class_attr(attr)
    }

    fn set_attr(&self, attr: &str, value: Rc<dyn PyObject>) -> Result<(), String> {
        // Check __slots__ on class and its MRO
        let slots_violation = {
            let class_slots = &self.class.slots;
            if let Some(slots) = class_slots {
                if !slots.contains(&attr.to_string()) {
                    Some(format!(
                        "AttributeError: '{}' object has no attribute '{}'",
                        self.class.name, attr
                    ))
                } else {
                    None
                }
            } else {
                // Check MRO for __slots__
                let mro_has_slots = self.class.mro.iter().any(|base| {
                    if let Some(cls) = base.as_any().downcast_ref::<PyClass>() {
                        cls.slots.is_some()
                    } else {
                        false
                    }
                });
                if mro_has_slots {
                    // Check if any base class has this slot
                    let in_mro_slot = self.class.mro.iter().any(|base| {
                        if let Some(cls) = base.as_any().downcast_ref::<PyClass>() {
                            if let Some(slots) = &cls.slots {
                                return slots.contains(&attr.to_string());
                            }
                        }
                        false
                    });
                    if !in_mro_slot {
                        Some(format!(
                            "AttributeError: '{}' object has no attribute '{}'",
                            self.class.name, attr
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        if let Some(err) = slots_violation {
            return Err(err);
        }
        self.attributes.borrow_mut().insert(attr.to_string(), value);
        Ok(())
    }

    fn del_attr(&self, name: &str) -> Result<(), String> {
        self.attributes.borrow_mut().remove(name);
        Ok(())
    }

    fn eq(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__eq__", vec![other]).ok()
    }

    fn ne(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__ne__", vec![other]).ok()
    }

    fn add(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__add__", vec![other]).ok()
    }

    fn sub(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__sub__", vec![other]).ok()
    }

    fn mul(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__mul__", vec![other]).ok()
    }

    fn truediv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__truediv__", vec![other]).ok()
    }

    fn floordiv(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__floordiv__", vec![other]).ok()
    }

    fn modulo(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__mod__", vec![other]).ok()
    }

    fn pow(&self, other: Rc<dyn PyObject>) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__pow__", vec![other]).ok()
    }

    fn neg(&self) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__neg__", vec![]).ok()
    }

    fn pos(&self) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__pos__", vec![]).ok()
    }

    fn invert(&self) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__invert__", vec![]).ok()
    }

    fn abs_op(&self) -> Option<Rc<dyn PyObject>> {
        self.call_dunder("__abs__", vec![]).ok()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        self.call_dunder("__getitem__", vec![key])
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        if let Ok(iter_method) = self.call_dunder("__iter__", vec![]) {
            return Ok(iter_method);
        }
        Err(format!(
            "TypeError: '{}' object is not iterable",
            self.class.name
        ))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        match self.call_dunder("__next__", vec![]) {
            Ok(result) => Ok(Some(result)),
            Err(e) if e.contains("StopIteration") => Ok(None),
            Err(e) => Err(e),
        }
    }
}
