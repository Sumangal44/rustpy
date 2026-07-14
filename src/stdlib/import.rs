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
    pub builtins_env: RefCell<Option<Rc<RefCell<crate::runtime::Environment>>>>,
}

impl ImportSystem {
    pub fn new() -> Self {
        Self {
            sys_modules: Rc::new(PyDict::new()),
            native_modules: Rc::new(RefCell::new(HashMap::new())),
            builtins_env: RefCell::new(None),
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

        // Check sys.path for filesystem imports
        let sys_key = Rc::new(PyString::new("sys".to_string())) as Rc<dyn PyObject>;
        if let Ok(sys_mod) = self.sys_modules.get_item_value(&sys_key) {
            if let Ok(path_obj) = sys_mod.get_attr("path") {
                if let Some(path_list) = path_obj.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    for p in path_list.elements.borrow().iter() {
                        if let Some(p_str) = p.as_any().downcast_ref::<PyString>() {
                            let dir_path = std::path::Path::new(&p_str.value);
                            
                            let mut file_path = dir_path.join(top_name);
                            file_path.set_extension("py");
                            
                            let mut init_path = dir_path.join(top_name);
                            init_path.push("__init__.py");
                            
                            let target_path = if file_path.exists() {
                                Some(file_path)
                            } else if init_path.exists() {
                                Some(init_path)
                            } else {
                                None
                            };

                            if let Some(path) = target_path {
                                let source = match std::fs::read_to_string(&path) {
                                    Ok(s) => s,
                                    Err(_) => continue,
                                };
                                
                                let builtins_env = self.builtins_env.borrow().as_ref().unwrap().clone();
                                let module_env = crate::runtime::Environment::new_enclosed(builtins_env);
                                
                                let module = Rc::new(PyModule::new(top_name.to_string()));
                                let module_obj = Rc::clone(&module) as Rc<dyn PyObject>;
                                self.sys_modules.set_item(Rc::clone(&key), Rc::clone(&module_obj)).map_err(|e| format!("ImportError: {}", e))?;

                                let lexer = crate::lexer::Lexer::new(&source);
                                match crate::parser::Parser::new(lexer) {
                                    Ok(mut parser) => {
                                        match parser.parse_module() {
                                            Ok(ast_module) => {
                                                let mut compiler = crate::compiler::Compiler::new(path.to_string_lossy().to_string());
                                                match compiler.compile(&ast_module) {
                                                    Ok(code) => {
                                                        let mut frame = crate::vm::frame::Frame::new(code, Rc::clone(&module_env));
                                                        let mut vm = crate::vm::VirtualMachine::new();
                                                        
                                                        match vm.run(&mut frame) {
                                                            Ok(_) => {
                                                                for (k, v) in module_env.borrow().get_all_locals() {
                                                                    module.set_attr_inner(&k, v);
                                                                }
                                                                return Ok(module_obj);
                                                            }
                                                            Err(e) => return Err(format!("RuntimeError in module {}: {}", top_name, e)),
                                                        }
                                                    }
                                                    Err(e) => return Err(format!("CompileError in module {}: {}", top_name, e)),
                                                }
                                            }
                                            Err(e) => return Err(format!("ParseError in module {}: {:?}", top_name, e)),
                                        }
                                    }
                                    Err(e) => return Err(format!("ParseError in module {}: {:?}", top_name, e)),
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(format!(
            "ModuleNotFoundError: No module named '{}'",
            name
        ))
    }
}
