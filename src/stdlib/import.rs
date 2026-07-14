use crate::objects::PyObject;
use crate::objects::dict::PyDict;
use crate::objects::module::PyModule;
use crate::objects::string::PyString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct ImportSystem {
    pub sys_modules: Rc<PyDict>,
    pub native_modules: Rc<RefCell<HashMap<String, Rc<PyModule>>>>,
}

impl ImportSystem {
    pub fn new() -> Self {
        Self {
            sys_modules: Rc::new(PyDict::new()),
            native_modules: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_native_module(&self, name: &str, module: Rc<PyModule>) {
        self.native_modules
            .borrow_mut()
            .insert(name.to_string(), module);
    }

    pub fn import_module(&self, name: &str) -> Result<Rc<dyn PyObject>, String> {
        let top_name = name.split('.').next().unwrap_or(name);

        // Check sys.modules
        let key = Rc::new(PyString::new(top_name.to_string())) as Rc<dyn PyObject>;
        if let Ok(module) = self.sys_modules.get_item_value(&key) {
            return Ok(module);
        }

        // Check native modules
        let native = self.native_modules.borrow();
        if let Some(module) = native.get(top_name) {
            let module_obj = Rc::clone(module) as Rc<dyn PyObject>;
            drop(native);
            self.sys_modules
                .set_item(key, Rc::clone(&module_obj))
                .map_err(|e| format!("ImportError: {}", e))?;
            return Ok(module_obj);
        }

        Err(format!(
            "ModuleNotFoundError: No module named '{}'",
            name
        ))
    }
}
