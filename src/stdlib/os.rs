use crate::objects::PyObject;
use crate::objects::int::PyInt;
use crate::objects::list::PyList;
use crate::objects::module::PyModule;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use crate::objects::tuple::PyTuple;
use std::fs;
use std::rc::Rc;

fn to_path(obj: &Rc<dyn PyObject>) -> String {
    if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
        s.value.clone()
    } else if let Some(_) = obj.as_any().downcast_ref::<PyNone>() {
        ".".to_string()
    } else {
        obj.str()
    }
}

fn str_to_pystring(s: String) -> Rc<dyn PyObject> {
    Rc::new(PyString::new(s)) as Rc<dyn PyObject>
}

pub fn create_os_module() -> Rc<PyModule> {
    let module = PyModule::new("os".to_string());
    let module = Rc::new(module);

    // getcwd()
    module.set_attr_inner(
        "getcwd",
        Rc::new(PyNativeFunction::new_pos_only(
            "getcwd".to_string(),
            |args| {
                if !args.is_empty() {
                    return Err(format!(
                        "TypeError: getcwd() takes no arguments ({} given)",
                        args.len()
                    ));
                }
                let cwd = std::env::current_dir().map_err(|e| format!("OSError: {}", e))?;
                Ok(str_to_pystring(cwd.to_string_lossy().to_string()))
            },
        )) as Rc<dyn PyObject>,
    );

    // chdir(path)
    module.set_attr_inner(
        "chdir",
        Rc::new(PyNativeFunction::new_pos_only(
            "chdir".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: chdir() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                std::env::set_current_dir(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // listdir(path='.')
    module.set_attr_inner(
        "listdir",
        Rc::new(PyNativeFunction::new_pos_only(
            "listdir".to_string(),
            |args| {
                if args.len() > 1 {
                    return Err(format!(
                        "TypeError: listdir() takes at most 1 argument ({} given)",
                        args.len()
                    ));
                }
                let path = if args.is_empty() {
                    "."
                } else {
                    &to_path(&args[0])
                };
                let rd = fs::read_dir(path).map_err(|e| format!("OSError: {}", e))?;
                let mut entries = Vec::new();
                for entry in rd {
                    let entry = entry.map_err(|e| format!("OSError: {}", e))?;
                    entries.push(str_to_pystring(
                        entry.file_name().to_string_lossy().to_string(),
                    ));
                }
                Ok(Rc::new(PyList::new(entries)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // mkdir(path, mode=0o777)
    module.set_attr_inner(
        "mkdir",
        Rc::new(PyNativeFunction::new_pos_only(
            "mkdir".to_string(),
            |args| {
                if args.is_empty() || args.len() > 2 {
                    return Err(format!(
                        "TypeError: mkdir() takes 1-2 arguments ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                fs::create_dir(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // makedirs(path, mode=0o777)
    module.set_attr_inner(
        "makedirs",
        Rc::new(PyNativeFunction::new_pos_only(
            "makedirs".to_string(),
            |args| {
                if args.is_empty() || args.len() > 2 {
                    return Err(format!(
                        "TypeError: makedirs() takes 1-2 arguments ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                fs::create_dir_all(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // remove(path)
    module.set_attr_inner(
        "remove",
        Rc::new(PyNativeFunction::new_pos_only(
            "remove".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: remove() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                fs::remove_file(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // unlink(path) - same as remove
    module.set_attr_inner(
        "unlink",
        Rc::new(PyNativeFunction::new_pos_only(
            "unlink".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: unlink() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                fs::remove_file(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // rename(src, dst)
    module.set_attr_inner(
        "rename",
        Rc::new(PyNativeFunction::new_pos_only(
            "rename".to_string(),
            |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "TypeError: rename() takes exactly 2 arguments ({} given)",
                        args.len()
                    ));
                }
                let src = to_path(&args[0]);
                let dst = to_path(&args[1]);
                fs::rename(&src, &dst).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // replace(src, dst) - same as rename
    module.set_attr_inner(
        "replace",
        Rc::new(PyNativeFunction::new_pos_only(
            "replace".to_string(),
            |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "TypeError: replace() takes exactly 2 arguments ({} given)",
                        args.len()
                    ));
                }
                let src = to_path(&args[0]);
                let dst = to_path(&args[1]);
                fs::rename(&src, &dst).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // rmdir(path)
    module.set_attr_inner(
        "rmdir",
        Rc::new(PyNativeFunction::new_pos_only(
            "rmdir".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: rmdir() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let path = to_path(&args[0]);
                fs::remove_dir(&path).map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // getenv(key, default=None)
    module.set_attr_inner(
        "getenv",
        Rc::new(PyNativeFunction::new_pos_only(
            "getenv".to_string(),
            |args| {
                if args.is_empty() || args.len() > 2 {
                    return Err(format!(
                        "TypeError: getenv() takes 1-2 arguments ({} given)",
                        args.len()
                    ));
                }
                let key = if let Some(s) = args[0].as_any().downcast_ref::<PyString>() {
                    s.value.clone()
                } else {
                    return Err("TypeError: getenv() argument 1 must be str".to_string());
                };
                match std::env::var(&key) {
                    Ok(val) => Ok(str_to_pystring(val)),
                    Err(_) => {
                        if args.len() >= 2 {
                            Ok(Rc::clone(&args[1]))
                        } else {
                            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
                        }
                    }
                }
            },
        )) as Rc<dyn PyObject>,
    );

    // putenv(key, value)
    module.set_attr_inner(
        "putenv",
        Rc::new(PyNativeFunction::new_pos_only(
            "putenv".to_string(),
            |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "TypeError: putenv() takes exactly 2 arguments ({} given)",
                        args.len()
                    ));
                }
                let key = if let Some(s) = args[0].as_any().downcast_ref::<PyString>() {
                    s.value.clone()
                } else {
                    return Err("TypeError: putenv() argument 1 must be str".to_string());
                };
                let value = if let Some(s) = args[1].as_any().downcast_ref::<PyString>() {
                    s.value.clone()
                } else {
                    return Err("TypeError: putenv() argument 2 must be str".to_string());
                };
                unsafe {
                    std::env::set_var(&key, &value);
                }
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // cpu_count()
    module.set_attr_inner(
        "cpu_count",
        Rc::new(PyNativeFunction::new_pos_only(
            "cpu_count".to_string(),
            |args| {
                if !args.is_empty() {
                    return Err(format!(
                        "TypeError: cpu_count() takes no arguments ({} given)",
                        args.len()
                    ));
                }
                let count = std::thread::available_parallelism()
                    .map(|n| n.get() as i64)
                    .unwrap_or(1);
                Ok(Rc::new(PyInt::from_i64(count)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // urandom(n)
    module.set_attr_inner(
        "urandom",
        Rc::new(PyNativeFunction::new_pos_only(
            "urandom".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: urandom() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let n = if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                    i.as_i64()
                        .ok_or_else(|| "OverflowError: n too large".to_string())?
                        as usize
                } else {
                    return Err("TypeError: urandom() argument 1 must be int".to_string());
                };
                let mut buf = vec![0u8; n];
                // Use /dev/urandom on macOS/Linux
                let mut f =
                    std::fs::File::open("/dev/urandom").map_err(|e| format!("OSError: {}", e))?;
                use std::io::Read;
                f.read_exact(&mut buf)
                    .map_err(|e| format!("OSError: {}", e))?;
                Ok(Rc::new(crate::objects::bytes::PyBytes::new(buf)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // walk(path)
    module.set_attr_inner(
        "walk",
        Rc::new(PyNativeFunction::new_pos_only("walk".to_string(), |args| {
            if args.len() > 1 {
                return Err(format!(
                    "TypeError: walk() takes at most 1 argument ({} given)",
                    args.len()
                ));
            }
            let path = if args.is_empty() {
                "."
            } else {
                &to_path(&args[0])
            };
            let path = path.trim_end_matches('/');
            let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
            walk_recursive(path, &mut result).map_err(|e| format!("OSError: {}", e))?;
            Ok(Rc::new(PyList::new(result)) as Rc<dyn PyObject>)
        })) as Rc<dyn PyObject>,
    );

    // os.name
    module.set_attr_inner(
        "name",
        Rc::new(PyString::new("posix".to_string())) as Rc<dyn PyObject>,
    );

    module
}

fn walk_recursive(dir: &str, result: &mut Vec<Rc<dyn PyObject>>) -> std::io::Result<()> {
    let mut dirnames = Vec::new();
    let mut filenames = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let name = entry.file_name().to_string_lossy().to_string();
        if ft.is_dir() {
            dirnames.push(name);
        } else {
            filenames.push(name);
        }
    }

    // Sort directories and files for deterministic output
    dirnames.sort();
    filenames.sort();

    let dirnames_objs: Vec<Rc<dyn PyObject>> = dirnames
        .iter()
        .map(|s| Rc::new(PyString::new(s.clone())) as Rc<dyn PyObject>)
        .collect();
    let filenames_objs: Vec<Rc<dyn PyObject>> = filenames
        .iter()
        .map(|s| Rc::new(PyString::new(s.clone())) as Rc<dyn PyObject>)
        .collect();

    result.push(Rc::new(PyTuple::new(vec![
        Rc::new(PyString::new(dir.to_string())) as Rc<dyn PyObject>,
        Rc::new(PyList::new(dirnames_objs)) as Rc<dyn PyObject>,
        Rc::new(PyList::new(filenames_objs)) as Rc<dyn PyObject>,
    ])) as Rc<dyn PyObject>);

    for dirname in &dirnames {
        let subdir = format!("{}/{}", dir, dirname);
        walk_recursive(&subdir, result)?;
    }

    Ok(())
}
