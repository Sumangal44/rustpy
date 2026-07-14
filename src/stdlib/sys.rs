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

    // sys.stdin, sys.stdout, sys.stderr
    let stdin = crate::objects::file::PyFile::stdin();
    module.set_attr_inner("stdin", Rc::new(stdin) as Rc<dyn PyObject>);
    let stdout = crate::objects::file::PyFile::stdout();
    module.set_attr_inner("stdout", Rc::new(stdout) as Rc<dyn PyObject>);
    let stderr = crate::objects::file::PyFile::stderr();
    module.set_attr_inner("stderr", Rc::new(stderr) as Rc<dyn PyObject>);

    module
}
