use crate::objects::PyObject;
use crate::objects::dict::PyDict;
use crate::objects::list::PyList;
use crate::objects::module::PyModule;
use crate::objects::string::PyString;
use std::rc::Rc;

pub fn create_sys_module(
    sys_modules: Rc<PyDict>,
    argv: Vec<String>,
) -> Rc<PyModule> {
    let module = PyModule::new("sys".to_string());
    let module = Rc::new(module);

    // sys.path
    let path = PyList::new(vec![
        Rc::new(PyString::new(".".to_string())) as Rc<dyn PyObject>,
    ]);
    module.set_attr_inner("path", Rc::new(path) as Rc<dyn PyObject>);

    // sys.modules (shared global dict)
    module.set_attr_inner(
        "modules",
        Rc::clone(&sys_modules) as Rc<dyn PyObject>,
    );

    // sys.argv
    let argv_list: Vec<Rc<dyn PyObject>> = argv
        .iter()
        .map(|a| Rc::new(PyString::new(a.clone())) as Rc<dyn PyObject>)
        .collect();
    module.set_attr_inner("argv", Rc::new(PyList::new(argv_list)) as Rc<dyn PyObject>);

    // sys.stdout placeholder (a simple object that has a write method)
    let stdout_module = Rc::new(PyModule::new("stdout".to_string()));
    stdout_module.set_attr_inner(
        "write",
        Rc::new(crate::objects::native_function::PyNativeFunction::new(
            "write".to_string(),
            |args| {
                if args.is_empty() {
                    return Err("TypeError: write() takes at least 1 argument".to_string());
                }
                print!("{}", args[0].str());
                Ok(Rc::new(crate::objects::none::PyNone))
            },
        )) as Rc<dyn PyObject>,
    );
    module.set_attr_inner("stdout", stdout_module as Rc<dyn PyObject>);

    module
}
