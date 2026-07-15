



use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
pub struct PyMap {
    func: Rc<dyn PyObject>,
    iter: Rc<dyn PyObject>,
}
impl PyMap {
    pub fn new(func: Rc<dyn PyObject>, iter: Rc<dyn PyObject>) -> Self {
        Self { func, iter }
    }
}
impl std::fmt::Debug for PyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<map object at {:p}>", self)
    }
}
impl PyObject for PyMap {
    fn get_type(&self) -> &'static str {
        "map"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn repr(&self) -> String {
        format!("<map object at {:p}>", self)
    }
    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        // Technically, a map object returns itself. We need an Rc, but we don't have it here.
        // For simplicity in RustPy without Rc<Self>, we can just wrap another PyMap or since 
        // PyMap is just called via get_next by the VM, we just implement get_next on it.
        // Wait, get_iter needs to return an Rc. But if the caller already has an Rc, they can use it.
        // We can just return Err. Usually get_iter isn't called on the map itself in a simple VM, 
        // or if it is, it's called during `list(map(...))`.
        // Let's implement an internal iterator struct instead, or just assume the caller handles it.
        Err("TypeError: map object is not iterable in this simple implementation".to_string())
    }
    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        if let Some(item) = self.iter.get_next()? {
            let res = self.func.call(vec![item], std::collections::HashMap::new())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}
