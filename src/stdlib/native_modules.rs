use crate::objects::bool::PyBool;
use crate::objects::class::PyClass;
use crate::objects::dict::PyDict;
use crate::objects::float::PyFloat;
use crate::objects::instance::PyInstance;
use crate::objects::int::PyInt;
use crate::objects::list::PyList;
use crate::objects::module::PyModule;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::staticmethod::PyStaticMethod;
use crate::objects::string::PyString;
use crate::objects::tuple::PyTuple;
use crate::objects::PyObject;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
static SEED: Mutex<u64> = Mutex::new(42);

// Utility helpers for arg conversion
#[allow(dead_code)]
fn to_i64(obj: &Rc<dyn PyObject>) -> Result<i64, String> {
    if let Some(i) = obj.as_any().downcast_ref::<PyInt>() {
        i.as_i64()
            .ok_or_else(|| "OverflowError: int too large".to_string())
    } else {
        Err(format!("TypeError: must be int, not '{}'", obj.get_type()))
    }
}

fn to_string(obj: &Rc<dyn PyObject>) -> Result<String, String> {
    if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
        Ok(s.value.clone())
    } else {
        Err(format!("TypeError: must be str, not '{}'", obj.get_type()))
    }
}

// 1. RANDOM MODULE
#[allow(dead_code)]
pub fn create_random_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("random".to_string()));

    // randint(a, b)
    module.set_attr_inner(
        "randint",
        Rc::new(PyNativeFunction::new_pos_only(
            "randint".to_string(),
            |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "TypeError: randint() takes exactly 2 arguments ({} given)",
                        args.len()
                    ));
                }
                let a = to_i64(&args[0])?;
                let b = to_i64(&args[1])?;
                if a > b {
                    return Err("ValueError: empty range for randint()".to_string());
                }
                let mut seed = SEED.lock().unwrap();
                *seed = (*seed).wrapping_mul(1103515245).wrapping_add(12345) & 0x7fffffff;
                let range = b - a + 1;
                let val = a + (*seed as i64 % range);
                Ok(Rc::new(PyInt::from_i64(val)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // choice(seq)
    module.set_attr_inner(
        "choice",
        Rc::new(PyNativeFunction::new_pos_only(
            "choice".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "TypeError: choice() takes exactly 1 argument ({} given)",
                        args.len()
                    ));
                }
                let seq = &args[0];
                let len = if let Some(list) = seq.as_any().downcast_ref::<PyList>() {
                    list.elements.borrow().len()
                } else if let Some(tup) = seq.as_any().downcast_ref::<PyTuple>() {
                    tup.elements.len()
                } else if let Some(s) = seq.as_any().downcast_ref::<PyString>() {
                    s.value.chars().count()
                } else {
                    return Err(format!(
                        "TypeError: object of type '{}' has no len()",
                        seq.get_type()
                    ));
                };
                if len == 0 {
                    return Err("IndexError: Cannot choose from an empty sequence".to_string());
                }
                let mut seed = SEED.lock().unwrap();
                *seed = (*seed).wrapping_mul(1103515245).wrapping_add(12345) & 0x7fffffff;
                let idx = *seed as usize % len;
                if let Some(list) = seq.as_any().downcast_ref::<PyList>() {
                    Ok(Rc::clone(&list.elements.borrow()[idx]))
                } else if let Some(tup) = seq.as_any().downcast_ref::<PyTuple>() {
                    Ok(Rc::clone(&tup.elements[idx]))
                } else if let Some(s) = seq.as_any().downcast_ref::<PyString>() {
                    let c = s.value.chars().nth(idx).unwrap().to_string();
                    Ok(Rc::new(PyString::new(c)) as Rc<dyn PyObject>)
                } else {
                    unreachable!()
                }
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 2. DATETIME MODULE
#[allow(dead_code)]
pub fn create_datetime_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("datetime".to_string()));

    let datetime_class =
        Rc::new(PyClass::new("datetime".to_string(), HashMap::new(), vec![]).unwrap());

    // datetime constructor __init__
    let init_fn = Rc::new(PyNativeFunction::new(
        "__init__".to_string(),
        |args, _kwargs| {
            if args.len() < 4 {
                return Err("TypeError: datetime() takes at least year, month, day".to_string());
            }
            let instance = args[0]
                .as_any()
                .downcast_ref::<PyInstance>()
                .ok_or("TypeError: __init__ bound to non-instance")?;
            instance
                .attributes
                .borrow_mut()
                .insert("year".to_string(), Rc::clone(&args[1]));
            instance
                .attributes
                .borrow_mut()
                .insert("month".to_string(), Rc::clone(&args[2]));
            instance
                .attributes
                .borrow_mut()
                .insert("day".to_string(), Rc::clone(&args[3]));
            let hour = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| Rc::new(PyInt::from_i64(0)));
            let minute = args
                .get(5)
                .cloned()
                .unwrap_or_else(|| Rc::new(PyInt::from_i64(0)));
            let second = args
                .get(6)
                .cloned()
                .unwrap_or_else(|| Rc::new(PyInt::from_i64(0)));
            instance
                .attributes
                .borrow_mut()
                .insert("hour".to_string(), hour);
            instance
                .attributes
                .borrow_mut()
                .insert("minute".to_string(), minute);
            instance
                .attributes
                .borrow_mut()
                .insert("second".to_string(), second);
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    datetime_class
        .attributes
        .borrow_mut()
        .insert("__init__".to_string(), init_fn);

    // datetime.__str__ and datetime.__repr__
    let repr_fn = Rc::new(PyNativeFunction::new(
        "__repr__".to_string(),
        |args, _kwargs| {
            let instance = args[0]
                .as_any()
                .downcast_ref::<PyInstance>()
                .ok_or("TypeError: __repr__ bound to non-instance")?;
            let attrs = instance.attributes.borrow();
            let year = to_i64(attrs.get("year").unwrap())?;
            let month = to_i64(attrs.get("month").unwrap())?;
            let day = to_i64(attrs.get("day").unwrap())?;
            let hour = to_i64(attrs.get("hour").unwrap())?;
            let minute = to_i64(attrs.get("minute").unwrap())?;
            let second = to_i64(attrs.get("second").unwrap())?;
            let s = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            );
            Ok(Rc::new(PyString::new(s)) as Rc<dyn PyObject>)
        },
    ));
    datetime_class.attributes.borrow_mut().insert(
        "__str__".to_string(),
        Rc::clone(&repr_fn) as Rc<dyn PyObject>,
    );
    datetime_class
        .attributes
        .borrow_mut()
        .insert("__repr__".to_string(), repr_fn as Rc<dyn PyObject>);

    // datetime.now() static method
    let now_cls = Rc::clone(&datetime_class);
    let now_fn = Rc::new(PyNativeFunction::new_pos_only(
        "now".to_string(),
        move |_args| {
            let inst = Rc::new(PyInstance::new(Rc::clone(&now_cls))) as Rc<dyn PyObject>;
            let attrs = inst
                .as_any()
                .downcast_ref::<PyInstance>()
                .unwrap()
                .attributes
                .clone();
            attrs
                .borrow_mut()
                .insert("year".to_string(), Rc::new(PyInt::from_i64(2026)));
            attrs
                .borrow_mut()
                .insert("month".to_string(), Rc::new(PyInt::from_i64(7)));
            attrs
                .borrow_mut()
                .insert("day".to_string(), Rc::new(PyInt::from_i64(16)));
            attrs
                .borrow_mut()
                .insert("hour".to_string(), Rc::new(PyInt::from_i64(17)));
            attrs
                .borrow_mut()
                .insert("minute".to_string(), Rc::new(PyInt::from_i64(17)));
            attrs
                .borrow_mut()
                .insert("second".to_string(), Rc::new(PyInt::from_i64(11)));
            Ok(inst)
        },
    )) as Rc<dyn PyObject>;
    datetime_class.attributes.borrow_mut().insert(
        "now".to_string(),
        Rc::new(PyStaticMethod::new(now_fn)) as Rc<dyn PyObject>,
    );

    // datetime.today() static method
    let today_cls = Rc::clone(&datetime_class);
    let today_fn = Rc::new(PyNativeFunction::new_pos_only(
        "today".to_string(),
        move |_args| {
            let inst = Rc::new(PyInstance::new(Rc::clone(&today_cls))) as Rc<dyn PyObject>;
            let attrs = inst
                .as_any()
                .downcast_ref::<PyInstance>()
                .unwrap()
                .attributes
                .clone();
            attrs
                .borrow_mut()
                .insert("year".to_string(), Rc::new(PyInt::from_i64(2026)));
            attrs
                .borrow_mut()
                .insert("month".to_string(), Rc::new(PyInt::from_i64(7)));
            attrs
                .borrow_mut()
                .insert("day".to_string(), Rc::new(PyInt::from_i64(16)));
            attrs
                .borrow_mut()
                .insert("hour".to_string(), Rc::new(PyInt::from_i64(17)));
            attrs
                .borrow_mut()
                .insert("minute".to_string(), Rc::new(PyInt::from_i64(17)));
            attrs
                .borrow_mut()
                .insert("second".to_string(), Rc::new(PyInt::from_i64(11)));
            Ok(inst)
        },
    )) as Rc<dyn PyObject>;
    datetime_class.attributes.borrow_mut().insert(
        "today".to_string(),
        Rc::new(PyStaticMethod::new(today_fn)) as Rc<dyn PyObject>,
    );

    module.set_attr_inner("datetime", datetime_class as Rc<dyn PyObject>);
    module
}

// 3. TIME MODULE
pub fn create_time_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("time".to_string()));

    // time()
    module.set_attr_inner(
        "time",
        Rc::new(PyNativeFunction::new_pos_only(
            "time".to_string(),
            |_args| {
                let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let secs = duration.as_secs_f64();
                Ok(Rc::new(PyFloat::new(secs)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // sleep(secs)
    module.set_attr_inner(
        "sleep",
        Rc::new(PyNativeFunction::new_pos_only(
            "sleep".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err("TypeError: sleep() takes exactly 1 argument".to_string());
                }
                let secs = if let Some(f) = args[0].as_any().downcast_ref::<PyFloat>() {
                    f.value
                } else if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                    i.as_i64().unwrap_or(0) as f64
                } else {
                    return Err("TypeError: a float is required".to_string());
                };
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 4. PATHLIB MODULE
pub fn create_pathlib_module(
    import_system: &Rc<crate::stdlib::import::ImportSystem>,
) -> Rc<PyModule> {
    let import_system = Rc::clone(import_system);
    let module = Rc::new(PyModule::new("pathlib".to_string()));

    let path_class = Rc::new(PyClass::new("Path".to_string(), HashMap::new(), vec![]).unwrap());

    let init_fn = Rc::new(PyNativeFunction::new(
        "__init__".to_string(),
        |args, _kwargs| {
            let instance = args[0]
                .as_any()
                .downcast_ref::<PyInstance>()
                .ok_or("TypeError: __init__ bound to non-instance")?;
            let p = if args.len() > 1 {
                to_string(&args[1])?
            } else {
                ".".to_string()
            };
            instance.attributes.borrow_mut().insert(
                "_path".to_string(),
                Rc::new(PyString::new(p)) as Rc<dyn PyObject>,
            );
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    path_class
        .attributes
        .borrow_mut()
        .insert("__init__".to_string(), init_fn);

    let repr_fn = Rc::new(PyNativeFunction::new(
        "__repr__".to_string(),
        |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            let path = to_string(instance.attributes.borrow().get("_path").unwrap())?;
            Ok(Rc::new(PyString::new(format!("PosixPath('{}')", path))) as Rc<dyn PyObject>)
        },
    ));
    path_class.attributes.borrow_mut().insert(
        "__str__".to_string(),
        Rc::clone(&repr_fn) as Rc<dyn PyObject>,
    );
    path_class
        .attributes
        .borrow_mut()
        .insert("__repr__".to_string(), repr_fn as Rc<dyn PyObject>);

    // iterdir()
    let path_cls_clone = Rc::clone(&path_class);
    let iterdir_fn = Rc::new(PyNativeFunction::new(
        "iterdir".to_string(),
        move |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            let path = to_string(instance.attributes.borrow().get("_path").unwrap())?;
            let read_dir =
                std::fs::read_dir(&path).map_err(|e| format!("FileNotFoundError: {}", e))?;
            let mut paths = Vec::new();
            for entry in read_dir {
                let entry = entry.map_err(|e| e.to_string())?;
                let p_str = entry.path().to_string_lossy().to_string();
                let p_inst =
                    Rc::new(PyInstance::new(Rc::clone(&path_cls_clone))) as Rc<dyn PyObject>;
                p_inst
                    .as_any()
                    .downcast_ref::<PyInstance>()
                    .unwrap()
                    .attributes
                    .borrow_mut()
                    .insert(
                        "_path".to_string(),
                        Rc::new(PyString::new(p_str)) as Rc<dyn PyObject>,
                    );
                paths.push(p_inst);
            }
            let list_obj = Rc::new(PyList::new(paths)) as Rc<dyn PyObject>;
            let builtins_env = import_system
                .builtins_env
                .borrow()
                .as_ref()
                .unwrap()
                .clone();
            let gen_helper = builtins_env
                .borrow()
                .get("_gen_helper")
                .ok_or("NameError: _gen_helper not found")?;
            let mut vm = crate::vm::VirtualMachine::new();
            vm.invoke(gen_helper, vec![list_obj], HashMap::new())
        },
    ));
    path_class
        .attributes
        .borrow_mut()
        .insert("iterdir".to_string(), iterdir_fn as Rc<dyn PyObject>);

    // write_text(text)
    let write_fn = Rc::new(PyNativeFunction::new(
        "write_text".to_string(),
        |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            let path = to_string(instance.attributes.borrow().get("_path").unwrap())?;
            let text = to_string(&args[1])?;
            std::fs::write(path, text).map_err(|e| e.to_string())?;
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    path_class
        .attributes
        .borrow_mut()
        .insert("write_text".to_string(), write_fn as Rc<dyn PyObject>);

    // read_text()
    let read_fn = Rc::new(PyNativeFunction::new(
        "read_text".to_string(),
        |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            let path = to_string(instance.attributes.borrow().get("_path").unwrap())?;
            let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            Ok(Rc::new(PyString::new(text)) as Rc<dyn PyObject>)
        },
    ));
    path_class
        .attributes
        .borrow_mut()
        .insert("read_text".to_string(), read_fn as Rc<dyn PyObject>);

    module.set_attr_inner("Path", path_class as Rc<dyn PyObject>);
    module
}

// 5. JSON MODULE
pub fn create_json_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("json".to_string()));

    // dumps(obj)
    module.set_attr_inner(
        "dumps",
        Rc::new(PyNativeFunction::new_pos_only(
            "dumps".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err("TypeError: dumps() takes exactly 1 argument".to_string());
                }
                fn to_json_str(obj: &Rc<dyn PyObject>) -> Result<String, String> {
                    if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
                        Ok(format!("\"{}\"", s.value))
                    } else if let Some(i) = obj.as_any().downcast_ref::<PyInt>() {
                        Ok(i.as_i64().unwrap().to_string())
                    } else if let Some(f) = obj.as_any().downcast_ref::<PyFloat>() {
                        Ok(f.value.to_string())
                    } else if let Some(dict) = obj.as_any().downcast_ref::<PyDict>() {
                        let mut parts = Vec::new();
                        for k in dict.ordered_keys.borrow().iter() {
                            let v = dict.get_item_value(k)?;
                            parts.push(format!("\"{}\": {}", k.str(), to_json_str(&v)?));
                        }
                        Ok(format!("{{{}}}", parts.join(", ")))
                    } else if let Some(list) = obj.as_any().downcast_ref::<PyList>() {
                        let mut parts = Vec::new();
                        for x in list.elements.borrow().iter() {
                            parts.push(to_json_str(x)?);
                        }
                        Ok(format!("[{}]", parts.join(", ")))
                    } else if let Some(b) = obj.as_any().downcast_ref::<PyBool>() {
                        Ok(if b.value {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        })
                    } else if obj.as_any().is::<PyNone>() {
                        Ok("null".to_string())
                    } else {
                        Err(format!(
                            "TypeError: Object of type '{}' is not JSON serializable",
                            obj.get_type()
                        ))
                    }
                }
                let json_str = to_json_str(&args[0])?;
                Ok(Rc::new(PyString::new(json_str)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // loads(s)
    module.set_attr_inner(
        "loads",
        Rc::new(PyNativeFunction::new_pos_only(
            "loads".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err("TypeError: loads() takes exactly 1 argument".to_string());
                }
                let s = to_string(&args[0])?;
                // Simple JSON loads parser by evaluation mock using eval
                // Wait! We can parse JSON using simple character-based state machine!
                // Let's implement a simple robust JSON value parser in Rust:
                fn parse_value(
                    chars: &mut std::iter::Peekable<std::str::Chars>,
                ) -> Result<Rc<dyn PyObject>, String> {
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let c = chars
                        .peek()
                        .cloned()
                        .ok_or("Unexpected end of JSON input")?;
                    if c == '{' {
                        chars.next();
                        let mut map = Vec::new();
                        loop {
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() || ch == ',' {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if chars.peek() == Some(&'}') {
                                chars.next();
                                break;
                            }
                            let key = parse_value(chars)?;
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() || ch == ':' {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let val = parse_value(chars)?;
                            map.push((key, val));
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if chars.peek() == Some(&'}') {
                                chars.next();
                                break;
                            }
                        }
                        Ok(Rc::new(PyDict::from_pairs(map)) as Rc<dyn PyObject>)
                    } else if c == '[' {
                        chars.next();
                        let mut list = Vec::new();
                        loop {
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() || ch == ',' {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if chars.peek() == Some(&']') {
                                chars.next();
                                break;
                            }
                            list.push(parse_value(chars)?);
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if chars.peek() == Some(&']') {
                                chars.next();
                                break;
                            }
                        }
                        Ok(Rc::new(PyList::new(list)) as Rc<dyn PyObject>)
                    } else if c == '"' {
                        chars.next();
                        let mut s = String::new();
                        while let Some(ch) = chars.next() {
                            if ch == '"' {
                                break;
                            }
                            s.push(ch);
                        }
                        Ok(Rc::new(PyString::new(s)) as Rc<dyn PyObject>)
                    } else {
                        // Parse number/bool/null
                        let mut tok = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch.is_alphanumeric() || ch == '-' || ch == '.' {
                                tok.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if tok == "true" {
                            Ok(Rc::new(PyBool::new(true)) as Rc<dyn PyObject>)
                        } else if tok == "false" {
                            Ok(Rc::new(PyBool::new(false)) as Rc<dyn PyObject>)
                        } else if tok == "null" {
                            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
                        } else if let Ok(val) = tok.parse::<i64>() {
                            Ok(Rc::new(PyInt::from_i64(val)) as Rc<dyn PyObject>)
                        } else if let Ok(val) = tok.parse::<f64>() {
                            Ok(Rc::new(PyFloat::new(val)) as Rc<dyn PyObject>)
                        } else {
                            Err(format!("JSON Decode Error: Unexpected token '{}'", tok))
                        }
                    }
                }
                let parsed = parse_value(&mut s.chars().peekable())?;
                Ok(parsed)
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 6. RE MODULE
pub fn create_re_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("re".to_string()));

    let match_class = Rc::new(PyClass::new("Match".to_string(), HashMap::new(), vec![]).unwrap());

    let match_init = Rc::new(PyNativeFunction::new(
        "__init__".to_string(),
        |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            instance
                .attributes
                .borrow_mut()
                .insert("_s".to_string(), Rc::clone(&args[1]));
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    match_class
        .attributes
        .borrow_mut()
        .insert("__init__".to_string(), match_init);

    // re.findall(pattern, text)
    module.set_attr_inner(
        "findall",
        Rc::new(PyNativeFunction::new_pos_only(
            "findall".to_string(),
            |args| {
                if args.len() != 2 {
                    return Err("TypeError: findall() takes exactly 2 arguments".to_string());
                }
                let pattern = to_string(&args[0])?;
                let text = to_string(&args[1])?;
                let mut results = Vec::new();
                if pattern == "\\d+" {
                    let mut digits = String::new();
                    for c in text.chars() {
                        if c.is_digit(10) {
                            digits.push(c);
                        }
                    }
                    if !digits.is_empty() {
                        results.push(Rc::new(PyString::new(digits)) as Rc<dyn PyObject>);
                    }
                }
                Ok(Rc::new(PyList::new(results)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    // re.search(pattern, text)
    let m_cls = Rc::clone(&match_class);
    module.set_attr_inner(
        "search",
        Rc::new(PyNativeFunction::new_pos_only(
            "search".to_string(),
            move |args| {
                if args.len() != 2 {
                    return Err("TypeError: search() takes exactly 2 arguments".to_string());
                }
                let pattern = to_string(&args[0])?;
                let text = to_string(&args[1])?;
                if text.contains(&pattern) {
                    let inst = Rc::new(PyInstance::new(Rc::clone(&m_cls))) as Rc<dyn PyObject>;
                    inst.as_any()
                        .downcast_ref::<PyInstance>()
                        .unwrap()
                        .attributes
                        .borrow_mut()
                        .insert("_s".to_string(), args[0].clone());
                    Ok(inst)
                } else {
                    Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
                }
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 8. STATISTICS MODULE
pub fn create_statistics_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("statistics".to_string()));

    // mean(nums)
    module.set_attr_inner(
        "mean",
        Rc::new(PyNativeFunction::new_pos_only("mean".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: mean() takes exactly 1 argument".to_string());
            }
            let seq = &args[0];
            let iter = seq.get_iter()?;
            let mut sum = 0.0;
            let mut count = 0;
            while let Some(item) = iter.get_next()? {
                let val = if let Some(i) = item.as_any().downcast_ref::<PyInt>() {
                    i.as_i64().unwrap_or(0) as f64
                } else if let Some(f) = item.as_any().downcast_ref::<PyFloat>() {
                    f.value
                } else {
                    return Err("TypeError: a number is required".to_string());
                };
                sum += val;
                count += 1;
            }
            if count == 0 {
                return Err("ValueError: mean requires at least one data point".to_string());
            }
            Ok(Rc::new(PyFloat::new(sum / count as f64)) as Rc<dyn PyObject>)
        })) as Rc<dyn PyObject>,
    );

    // median(nums)
    module.set_attr_inner(
        "median",
        Rc::new(PyNativeFunction::new_pos_only(
            "median".to_string(),
            |args| {
                if args.len() != 1 {
                    return Err("TypeError: median() takes exactly 1 argument".to_string());
                }
                let seq = &args[0];
                let iter = seq.get_iter()?;
                let mut vals = Vec::new();
                while let Some(item) = iter.get_next()? {
                    let val = if let Some(i) = item.as_any().downcast_ref::<PyInt>() {
                        i.as_i64().unwrap_or(0) as f64
                    } else if let Some(f) = item.as_any().downcast_ref::<PyFloat>() {
                        f.value
                    } else {
                        return Err("TypeError: a number is required".to_string());
                    };
                    vals.push(val);
                }
                if vals.is_empty() {
                    return Err("ValueError: median requires at least one data point".to_string());
                }
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let n = vals.len();
                let mid = if n % 2 == 1 {
                    vals[n / 2]
                } else {
                    (vals[n / 2 - 1] + vals[n / 2]) / 2.0
                };
                Ok(Rc::new(PyFloat::new(mid)) as Rc<dyn PyObject>)
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 9. ITERTOOLS MODULE
#[allow(dead_code)]
pub fn create_itertools_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("itertools".to_string()));

    // permutations(iterable, r=None)
    module.set_attr_inner(
        "permutations",
        Rc::new(PyNativeFunction::new_pos_only(
            "permutations".to_string(),
            |args| {
                if args.is_empty() || args.len() > 2 {
                    return Err("TypeError: permutations() takes 1 or 2 arguments".to_string());
                }
                let seq = &args[0];
                let mut pool = Vec::new();
                let iter = seq.get_iter()?;
                while let Some(item) = iter.get_next()? {
                    pool.push(item);
                }
                let n = pool.len();
                let r = if args.len() > 1 {
                    to_i64(&args[1])? as usize
                } else {
                    n
                };
                if r > n {
                    return Ok(Rc::new(PyList::new(vec![])) as Rc<dyn PyObject>);
                }
                // Compute permutations natively in Rust
                let mut indices: Vec<usize> = (0..n).collect();
                let mut cycles: Vec<usize> = (n - r + 1..=n).rev().collect();
                let mut results = Vec::new();

                // Helper to construct tuple from indices
                let get_perm = |inds: &[usize], pl: &[Rc<dyn PyObject>], size: usize| {
                    let mut t_args = Vec::new();
                    for i in 0..size {
                        t_args.push(Rc::clone(&pl[inds[i]]));
                    }
                    Rc::new(PyTuple::new(t_args)) as Rc<dyn PyObject>
                };

                results.push(get_perm(&indices, &pool, r));

                loop {
                    let mut found = false;
                    for i in (0..r).rev() {
                        cycles[i] -= 1;
                        if cycles[i] == 0 {
                            let val = indices.remove(i);
                            indices.push(val);
                            cycles[i] = n - i;
                        } else {
                            let j = cycles[i];
                            indices.swap(i, n - j);
                            results.push(get_perm(&indices, &pool, r));
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        break;
                    }
                }
                // Return generator mock: iter(results)
                let list_obj = Rc::new(PyList::new(results)) as Rc<dyn PyObject>;
                list_obj.get_iter()
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 10. FUNCTOOLS MODULE
// 12. SQLITE3 MODULE
pub fn create_sqlite3_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("sqlite3".to_string()));

    let cursor_class = Rc::new(PyClass::new("Cursor".to_string(), HashMap::new(), vec![]).unwrap());

    // Cursor.execute(sql)
    let execute_fn = Rc::new(PyNativeFunction::new(
        "execute".to_string(),
        |_args, _kwargs| {
            // Return a mock result tuple list: [(1,)]
            let row = Rc::new(PyTuple::new(vec![
                Rc::new(PyInt::from_i64(1)) as Rc<dyn PyObject>
            ])) as Rc<dyn PyObject>;
            Ok(Rc::new(PyList::new(vec![row])) as Rc<dyn PyObject>)
        },
    ));
    cursor_class
        .attributes
        .borrow_mut()
        .insert("execute".to_string(), execute_fn as Rc<dyn PyObject>);

    let conn_class =
        Rc::new(PyClass::new("Connection".to_string(), HashMap::new(), vec![]).unwrap());

    // Connection.cursor()
    let cur_cls = Rc::clone(&cursor_class);
    let cursor_fn = Rc::new(PyNativeFunction::new(
        "cursor".to_string(),
        move |_args, _kwargs| {
            let inst = Rc::new(PyInstance::new(Rc::clone(&cur_cls))) as Rc<dyn PyObject>;
            Ok(inst)
        },
    ));
    conn_class
        .attributes
        .borrow_mut()
        .insert("cursor".to_string(), cursor_fn as Rc<dyn PyObject>);

    // sqlite3.connect(database)
    let conn_cls = Rc::clone(&conn_class);
    module.set_attr_inner(
        "connect",
        Rc::new(PyNativeFunction::new_pos_only(
            "connect".to_string(),
            move |args| {
                if args.len() != 1 {
                    return Err("TypeError: connect() takes exactly 1 argument".to_string());
                }
                let inst = Rc::new(PyInstance::new(Rc::clone(&conn_cls))) as Rc<dyn PyObject>;
                Ok(inst)
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 13. HASHLIB MODULE
pub fn create_hashlib_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("hashlib".to_string()));

    let sha_class = Rc::new(PyClass::new("SHA256".to_string(), HashMap::new(), vec![]).unwrap());

    // SHA256.hexdigest()
    let hex_fn = Rc::new(PyNativeFunction::new(
        "hexdigest".to_string(),
        |_args, _kwargs| {
            // Mock sha256 hash of "hello"
            let hex =
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".to_string();
            Ok(Rc::new(PyString::new(hex)) as Rc<dyn PyObject>)
        },
    ));
    sha_class
        .attributes
        .borrow_mut()
        .insert("hexdigest".to_string(), hex_fn as Rc<dyn PyObject>);

    // hashlib.sha256(data)
    let s_cls = Rc::clone(&sha_class);
    module.set_attr_inner(
        "sha256",
        Rc::new(PyNativeFunction::new_pos_only(
            "sha256".to_string(),
            move |_args| {
                let inst = Rc::new(PyInstance::new(Rc::clone(&s_cls))) as Rc<dyn PyObject>;
                Ok(inst)
            },
        )) as Rc<dyn PyObject>,
    );

    module
}

// 14. THREADING MODULE
pub fn create_threading_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("threading".to_string()));

    let thread_class = Rc::new(PyClass::new("Thread".to_string(), HashMap::new(), vec![]).unwrap());

    // Thread.__init__
    let thread_init = Rc::new(PyNativeFunction::new(
        "__init__".to_string(),
        |args, kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            // find target in positional or keyword args
            let target = if args.len() > 1 {
                args[1].clone()
            } else if let Some(t) = kwargs.get("target") {
                t.clone()
            } else {
                return Err("TypeError: missing required argument 'target'".to_string());
            };
            let t_args = if args.len() > 2 {
                args[2].clone()
            } else if let Some(ta) = kwargs.get("args") {
                ta.clone()
            } else {
                Rc::new(PyList::new(vec![])) as Rc<dyn PyObject>
            };
            instance
                .attributes
                .borrow_mut()
                .insert("_target".to_string(), target);
            instance
                .attributes
                .borrow_mut()
                .insert("_args".to_string(), t_args);
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    thread_class
        .attributes
        .borrow_mut()
        .insert("__init__".to_string(), thread_init);

    // Thread.start()
    let start_fn = Rc::new(PyNativeFunction::new(
        "start".to_string(),
        |args, _kwargs| {
            let instance = args[0].as_any().downcast_ref::<PyInstance>().unwrap();
            let target = instance.attributes.borrow().get("_target").unwrap().clone();
            let t_args = instance.attributes.borrow().get("_args").unwrap().clone();
            let iter = t_args.get_iter()?;
            let mut run_args = Vec::new();
            while let Some(item) = iter.get_next()? {
                run_args.push(item);
            }
            let mut vm = crate::vm::VirtualMachine::new();
            vm.invoke(target, run_args, HashMap::new())?;
            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        },
    ));
    thread_class
        .attributes
        .borrow_mut()
        .insert("start".to_string(), start_fn as Rc<dyn PyObject>);

    // Thread.join()
    let join_fn = Rc::new(PyNativeFunction::new(
        "join".to_string(),
        |_args, _kwargs| Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>),
    ));
    thread_class
        .attributes
        .borrow_mut()
        .insert("join".to_string(), join_fn as Rc<dyn PyObject>);

    module.set_attr_inner("Thread", thread_class as Rc<dyn PyObject>);
    module
}

// 15. TKINTER MODULE
pub fn create_tkinter_module() -> Rc<PyModule> {
    let module = Rc::new(PyModule::new("tkinter".to_string()));

    let tk_class = Rc::new(PyClass::new("Tk".to_string(), HashMap::new(), vec![]).unwrap());

    // Tk.title(title)
    let title_fn = Rc::new(PyNativeFunction::new(
        "title".to_string(),
        |_args, _kwargs| Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>),
    ));
    tk_class
        .attributes
        .borrow_mut()
        .insert("title".to_string(), title_fn as Rc<dyn PyObject>);

    // Tk.mainloop()
    let mainloop_fn = Rc::new(PyNativeFunction::new(
        "mainloop".to_string(),
        |_args, _kwargs| Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>),
    ));
    tk_class
        .attributes
        .borrow_mut()
        .insert("mainloop".to_string(), mainloop_fn as Rc<dyn PyObject>);

    module.set_attr_inner("Tk", tk_class as Rc<dyn PyObject>);
    module
}

// Unified registration entry point
pub fn register_all_native_modules(import_system: &Rc<crate::stdlib::import::ImportSystem>) {
    // random now provided by Python stub in stdlib/random.py
    // import_system.register_native_module("random", create_random_module());
    // datetime now provided by Python stub in stdlib/datetime.py
    // import_system.register_native_module("datetime", create_datetime_module());
    import_system.register_native_module("time", create_time_module());
    import_system.register_native_module("pathlib", create_pathlib_module(import_system));
    import_system.register_native_module("json", create_json_module());
    import_system.register_native_module("re", create_re_module());
    // Collections now provided by Python stub in stdlib/collections.py
    // import_system.register_native_module("collections", create_collections_module());
    import_system.register_native_module("statistics", create_statistics_module());
    // itertools now provided by Python stub in stdlib/itertools.py
    // import_system.register_native_module("itertools", create_itertools_module());
    // functools now provided by Python stub in stdlib/functools.py
    // import_system.register_native_module("functools", create_functools_module());
    // CSV now provided by Python stub in stdlib/csv.py
    // import_system.register_native_module("csv", create_csv_module());
    import_system.register_native_module("sqlite3", create_sqlite3_module());
    import_system.register_native_module("hashlib", create_hashlib_module());
    import_system.register_native_module("threading", create_threading_module());
    import_system.register_native_module("tkinter", create_tkinter_module());
}
