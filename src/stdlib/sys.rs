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
        Rc::new(PyString::new("stdlib".to_string())) as Rc<dyn PyObject>,
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

    // sys.version and sys.platform
    module.set_attr_inner("version", Rc::new(PyString::new("3.14.0".to_string())) as Rc<dyn PyObject>);
    module.set_attr_inner("platform", Rc::new(PyString::new("darwin".to_string())) as Rc<dyn PyObject>);

    // sys.stdin, sys.stdout, sys.stderr
    let stdin = crate::objects::file::PyFile::stdin();
    module.set_attr_inner("stdin", Rc::new(stdin) as Rc<dyn PyObject>);
    let stdout = crate::objects::file::PyFile::stdout();
    module.set_attr_inner("stdout", Rc::new(stdout) as Rc<dyn PyObject>);
    let stderr = crate::objects::file::PyFile::stderr();
    module.set_attr_inner("stderr", Rc::new(stderr) as Rc<dyn PyObject>);

    // sys.builtin_module_names
    let module_names = vec![
        "builtins", "sys", "math", "os", "random", "datetime", "time", "calendar",
        "pathlib", "shutil", "json", "csv", "re", "collections", "itertools", "functools",
        "statistics", "decimal", "fractions", "string", "hashlib", "secrets", "logging",
        "sqlite3", "threading", "multiprocessing", "asyncio", "socket", "urllib", "email",
        "zipfile", "gzip", "tarfile", "tkinter", "unittest", "typing", "dataclasses",
    ];
    let builtins_list = module_names
        .iter()
        .map(|name| Rc::new(PyString::new(name.to_string())) as Rc<dyn PyObject>)
        .collect();
    let builtin_module_names = crate::objects::tuple::PyTuple::new(builtins_list);
    module.set_attr_inner(
        "builtin_module_names",
        Rc::new(builtin_module_names) as Rc<dyn PyObject>,
    );

    module
}
