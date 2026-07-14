use crate::compiler::Compiler;
use crate::lexer::Lexer;
use crate::objects::PyObject;
use crate::objects::bytes::PyBytes;
use crate::objects::int::PyInt;
use crate::objects::module::PyModule;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use crate::parser::Parser;
use crate::runtime::Environment;
use crate::vm::VirtualMachine;
use crate::vm::frame::Frame;
use crate::stdlib::import::ImportSystem;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub fn inject_builtins(env: &Rc<RefCell<Environment>>) {
    let mut env_mut = env.borrow_mut();

    // Inject constants
    env_mut.set(
        "__debug__".to_string(),
        Rc::new(crate::objects::bool::PyBool::new(true)),
    );
    env_mut.set(
        "NotImplemented".to_string(),
        Rc::new(crate::objects::constants::PyNotImplemented),
    );
    env_mut.set(
        "Ellipsis".to_string(),
        Rc::new(crate::objects::constants::PyEllipsis),
    );

    // Inject exceptions
    let exceptions = vec![
        "Exception",
        "TypeError",
        "ValueError",
        "NameError",
        "IndexError",
        "KeyError",
        "ImportError",
        "ModuleNotFoundError",
        "RuntimeError",
        "AttributeError",
        "SyntaxError",
        "IndentationError",
        "ZeroDivisionError",
        "FileNotFoundError",
        "PermissionError",
        "KeyboardInterrupt",
        "StopIteration",
        "StopAsyncIteration",
        "MemoryError",
        "OverflowError",
        "RecursionError",
    ];

    for exc in exceptions {
        let exc_name = exc.to_string();
        env_mut.set(
            exc_name.clone(),
            Rc::new(PyNativeFunction::new_pos_only(exc_name.clone(), move |args| {
                let msg = if !args.is_empty() {
                    Some(args[0].str())
                } else {
                    None
                };
                Ok(Rc::new(crate::objects::exception::PyException::new(
                    exc_name.clone(),
                    msg,
                )))
            })),
        );
    }

    // print(*objects, sep=' ', end='\n', file=sys.stdout, flush=False)
    let print_func = {
        let env = env.clone();
        Rc::new(PyNativeFunction::new("print".to_string(), move |args, kwargs| {
            let sep = kwargs.get("sep").map_or(" ".to_string(), |v| v.str());
            let end = kwargs.get("end").map_or("\n".to_string(), |v| v.str());
            let flush = kwargs.get("flush").map_or(false, |v| v.is_truthy());

            let mut out = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { out.push_str(&sep); }
                out.push_str(&arg.str());
            }
            out.push_str(&end);

            let file = if let Some(v) = kwargs.get("file") {
                Rc::clone(v)
            } else {
                env.borrow().get("stdout")
                    .unwrap_or_else(|| Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>)
            };

            if let Some(file_obj) = file.as_any().downcast_ref::<crate::objects::file::PyFile>() {
                file_obj.write(out)?;
                if flush { file_obj.flush()?; }
            } else if let Some(bm) = file.as_any().downcast_ref::<crate::objects::bound_method::PyBoundMethod>() {
                let write_args = vec![Rc::new(crate::objects::string::PyString::new(out)) as Rc<dyn PyObject>];
                let mut bound_args = vec![Rc::new(bm.instance.clone()) as Rc<dyn PyObject>];
                bound_args.extend(write_args);
                if let Some(native_fn) = bm.func.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                    (native_fn.func)(bound_args, std::collections::HashMap::new())?;
                } else {
                    return Err("TypeError: file.write is not callable".to_string());
                }
                if flush {
                    let flush_func = file.get_attr("flush")?;
                    if let Some(bm2) = flush_func.as_any().downcast_ref::<crate::objects::bound_method::PyBoundMethod>() {
                        let f_args = vec![Rc::new(bm2.instance.clone()) as Rc<dyn PyObject>];
                        if let Some(nf) = bm2.func.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                            (nf.func)(f_args, std::collections::HashMap::new())?;
                        }
                    }
                }
            } else {
                print!("{}", out);
            }

            Ok(Rc::new(PyNone::new()))
        }))
    };
    env_mut.set("print".to_string(), print_func);

    // len(obj)
    env_mut.set(
        "len".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("len".to_string(), |args| {
            if args.len() != 1 {
                return Err(format!(
                    "TypeError: len() takes exactly one argument ({} given)",
                    args.len()
                ));
            }
            let obj = &args[0];
            if let Some(b) = obj.as_any().downcast_ref::<PyBytes>() {
                Ok(Rc::new(PyInt::from_i64(b.value.len() as i64)))
            } else if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
                Ok(Rc::new(PyInt::from_i64(s.value.chars().count() as i64)))
            } else if let Some(l) = obj.as_any().downcast_ref::<crate::objects::list::PyList>() {
                Ok(Rc::new(PyInt::from_i64(l.elements.borrow().len() as i64)))
            } else if let Some(t) = obj.as_any().downcast_ref::<crate::objects::tuple::PyTuple>() {
                Ok(Rc::new(PyInt::from_i64(t.elements.len() as i64)))
            } else if let Some(d) = obj.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                Ok(Rc::new(PyInt::from_i64(d.entries.borrow().len() as i64)))
            } else if let Some(s) = obj.as_any().downcast_ref::<crate::objects::set::PySet>() {
                Ok(Rc::new(PyInt::from_i64(s.elements.borrow().len() as i64)))
            } else if let Some(fs) = obj.as_any().downcast_ref::<crate::objects::set::PyFrozenSet>() {
                Ok(Rc::new(PyInt::from_i64(fs.elements.borrow().len() as i64)))
            } else if let Some(r) = obj.as_any().downcast_ref::<crate::objects::range::PyRange>() {
                Ok(Rc::new(PyInt::from_i64(r.len() as i64)))
            } else if let Some(inst) = obj.as_any().downcast_ref::<crate::objects::instance::PyInstance>() {
                Ok(Rc::new(PyInt::from_i64(inst.len()? as i64)))
            } else {
                Err(format!(
                    "TypeError: object of type '{}' has no len()",
                    obj.get_type()
                ))
            }
        })),
    );

    // Register type objects for built-in types
    for type_name in &["int", "str", "float", "bool", "list", "dict", "tuple", "set", "frozenset", "bytes", "bytearray", "complex", "range", "slice"] {
        env_mut.set(
            type_name.to_string(),
            Rc::new(crate::objects::typeobj::PyType::new(type_name)) as Rc<dyn PyObject>,
        );
    }

    // str(obj)
    env_mut.set(
        "str".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("str".to_string(), |args| {
            if args.len() != 1 {
                return Err(format!(
                    "TypeError: str() takes exactly one argument ({} given)",
                    args.len()
                ));
            }
            let obj = &args[0];
            Ok(Rc::new(PyString::new(obj.str())))
        })),
    );

    // type(obj)
    env_mut.set(
        "type".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("type".to_string(), |args| {
            if args.len() != 1 {
                return Err(format!(
                    "TypeError: type() takes exactly one argument ({} given)",
                    args.len()
                ));
            }
            let obj = &args[0];
            Ok(Rc::new(crate::objects::typeobj::PyType::new(obj.get_type())))
        })),
    );

    // getattr(object, name[, default])
    env_mut.set(
        "getattr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("getattr".to_string(), |args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(format!(
                    "TypeError: getattr expected at least 2 arguments, got {}",
                    args.len()
                ));
            }
            let obj = &args[0];
            let name = &args[1];
            if name.get_type() != "str" {
                return Err("TypeError: getattr(): attribute name must be string".to_string());
            }
            let name_str = name
                .as_any()
                .downcast_ref::<crate::objects::string::PyString>()
                .unwrap()
                .value
                .clone();

            match obj.get_attr(&name_str) {
                Ok(val) => Ok(val),
                Err(e) => {
                    if args.len() == 3 {
                        Ok(args[2].clone())
                    } else {
                        Err(e)
                    }
                }
            }
        })),
    );

    // setattr(object, name, value)
    env_mut.set(
        "setattr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("setattr".to_string(), |args| {
            if args.len() != 3 {
                return Err("TypeError: setattr expected 3 arguments".to_string());
            }
            let obj = &args[0];
            let name = &args[1];
            let value = &args[2];
            if name.get_type() != "str" {
                return Err("TypeError: setattr(): attribute name must be string".to_string());
            }
            let name_str = name
                .as_any()
                .downcast_ref::<crate::objects::string::PyString>()
                .unwrap()
                .value
                .clone();

            obj.set_attr(&name_str, value.clone())?;
            Ok(Rc::new(crate::objects::none::PyNone::new()))
        })),
    );

    // hasattr(object, name)
    env_mut.set(
        "hasattr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("hasattr".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: hasattr expected 2 arguments".to_string());
            }
            let obj = &args[0];
            let name = &args[1];
            if name.get_type() != "str" {
                return Err("TypeError: hasattr(): attribute name must be string".to_string());
            }
            let name_str = name
                .as_any()
                .downcast_ref::<crate::objects::string::PyString>()
                .unwrap()
                .value
                .clone();

            match obj.get_attr(&name_str) {
                Ok(_) => Ok(Rc::new(crate::objects::bool::PyBool::new(true))),
                Err(_) => Ok(Rc::new(crate::objects::bool::PyBool::new(false))),
            }
        })),
    );

    // isinstance(object, classinfo)
    env_mut.set(
        "isinstance".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("isinstance".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: isinstance expected 2 arguments".to_string());
            }
            let obj = &args[0];
            let classinfo = &args[1];

            let type_name = if let Some(cls) = classinfo
                .as_any()
                .downcast_ref::<crate::objects::class::PyClass>()
            {
                cls.name.clone()
            } else if let Some(tp) = classinfo
                .as_any()
                .downcast_ref::<crate::objects::typeobj::PyType>()
            {
                tp.name.clone()
            } else if classinfo.get_type() == "str" {
                classinfo
                    .as_any()
                    .downcast_ref::<crate::objects::string::PyString>()
                    .unwrap()
                    .value
                    .clone()
            } else {
                return Err("TypeError: isinstance() arg 2 must be a type".to_string());
            };

            let is_inst = obj.get_type() == type_name;
            Ok(Rc::new(crate::objects::bool::PyBool::new(is_inst)))
        })),
    );

    // id(object)
    env_mut.set(
        "id".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("id".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: id() takes exactly one argument".to_string());
            }
            let obj = &args[0];
            // Get the raw pointer address of the dyn PyObject
            let ptr = Rc::as_ptr(obj) as *const () as i64;
            Ok(Rc::new(crate::objects::int::PyInt::from_i64(ptr)))
        })),
    );
    // hash(object)
    env_mut.set(
        "hash".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("hash".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: hash() takes exactly one argument".to_string());
            }
            let obj = &args[0];
            match obj.hash() {
                Ok(h) => Ok(Rc::new(crate::objects::int::PyInt::from_i64(h))),
                Err(e) => Err(e),
            }
        })),
    );

    // bool(object)
    env_mut.set(
        "bool".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("bool".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
            }
            if args.len() != 1 {
                return Err("TypeError: bool() takes at most 1 argument".to_string());
            }
            Ok(Rc::new(crate::objects::bool::PyBool::new(
                args[0].is_truthy(),
            )))
        })),
    );

    // int(x, base=10)
    env_mut.set(
        "int".to_string(),
        Rc::new(PyNativeFunction::new("int".to_string(), |args, kwargs| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(0)));
            }
            if args.len() > 2 {
                return Err("TypeError: int() takes at most 2 arguments".to_string());
            }
            let base = if args.len() == 2 {
                if let Some(i) = args[1].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    i.as_i64().unwrap_or(10)
                } else { 10 }
            } else {
                kwargs.get("base").and_then(|v| v.as_any().downcast_ref::<crate::objects::int::PyInt>()).map(|i| i.as_i64().unwrap_or(10)).unwrap_or(10)
            };
            let obj = &args[0];
            if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(i.as_i64().unwrap_or(0))));
            }
            if let Some(s) = obj.as_any().downcast_ref::<crate::objects::string::PyString>() {
                let trimmed = s.value.trim();
                if base == 10 {
                    if let Ok(val) = trimmed.parse::<i64>() {
                        return Ok(Rc::new(crate::objects::int::PyInt::from_i64(val)));
                    } else {
                        return Err(format!("ValueError: invalid literal for int() with base 10: '{}'", s.value));
                    }
                } else if base == 0 {
                    let (radix, digits) = if trimmed.starts_with("0x") || trimmed.starts_with("0X") { (16, &trimmed[2..]) }
                        else if trimmed.starts_with("0o") || trimmed.starts_with("0O") { (8, &trimmed[2..]) }
                        else if trimmed.starts_with("0b") || trimmed.starts_with("0B") { (2, &trimmed[2..]) }
                        else { (10, trimmed) };
                    if let Ok(val) = i64::from_str_radix(digits, radix) {
                        return Ok(Rc::new(crate::objects::int::PyInt::from_i64(val)));
                    } else {
                        return Err(format!("ValueError: invalid literal for int() with base {}: '{}'", radix, s.value));
                    }
                } else {
                    let digits = if trimmed.starts_with("0x") || trimmed.starts_with("0X") { &trimmed[2..] } else if trimmed.starts_with("0o") || trimmed.starts_with("0O") { &trimmed[2..] } else if trimmed.starts_with("0b") || trimmed.starts_with("0B") { &trimmed[2..] } else { trimmed };
                    if let Ok(val) = i64::from_str_radix(digits, base as u32) {
                        return Ok(Rc::new(crate::objects::int::PyInt::from_i64(val)));
                    } else {
                        return Err(format!("ValueError: invalid literal for int() with base {}: '{}'", base, s.value));
                    }
                }
            }
            if let Some(b) = obj.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(if b.value { 1 } else { 0 })));
            }
            if let Some(f) = obj.as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(f.value as i64)));
            }
            Err(format!("TypeError: int() argument must be a string, a bytes-like object or a real number, not '{}'", obj.get_type()))
        })),
    );

    // list(iterable)
    env_mut.set(
        "list".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("list".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::list::PyList::new(vec![])));
            }
            if args.len() != 1 {
                return Err("TypeError: list() takes at most 1 argument".to_string());
            }
            let obj = &args[0];
            let iter = obj.get_iter()?;
            let mut items = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            Ok(Rc::new(crate::objects::list::PyList::new(items)))
        })),
    );

    // tuple(iterable)
    env_mut.set(
        "tuple".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("tuple".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![])));
            }
            if args.len() != 1 {
                return Err("TypeError: tuple() takes at most 1 argument".to_string());
            }
            let obj = &args[0];
            let iter = obj.get_iter()?;
            let mut items = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            Ok(Rc::new(crate::objects::tuple::PyTuple::new(items)))
        })),
    );

    // dict(iterable)
    env_mut.set(
        "dict".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("dict".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::dict::PyDict::new()));
            }
            if args.len() != 1 {
                return Err("TypeError: dict() takes at most 1 argument".to_string());
            }
            let iter = args[0].get_iter()?;
            let mut pairs = Vec::new();
            while let Some(item) = iter.get_next()? {
                if let Some(t) = item.as_any().downcast_ref::<crate::objects::tuple::PyTuple>() {
                    if t.elements.len() != 2 {
                        return Err("TypeError: dict() item must have length 2".to_string());
                    }
                    pairs.push((Rc::clone(&t.elements[0]), Rc::clone(&t.elements[1])));
                } else if let Some(l) = item.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    let elems = l.elements.borrow();
                    if elems.len() != 2 {
                        return Err("TypeError: dict() item must have length 2".to_string());
                    }
                    pairs.push((Rc::clone(&elems[0]), Rc::clone(&elems[1])));
                } else {
                    return Err("TypeError: dict() item must be a pair".to_string());
                }
            }
            Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
        })),
    );
    // callable(object)
    env_mut.set(
        "callable".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("callable".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: callable() takes exactly one argument".to_string());
            }
            let obj = &args[0];
            let type_str = obj.get_type();
            let is_call = type_str == "function"
                || type_str == "native_function"
                || type_str == "class"
                || type_str == "bound_method"
                || type_str == "type"
                || type_str == "builtin_function_or_method"
                || type_str == "method";
            Ok(Rc::new(crate::objects::bool::PyBool::new(is_call)))
        })),
    );

    // object()
    env_mut.set(
        "object".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("object".to_string(), |args| {
            if !args.is_empty() {
                return Err("TypeError: object() takes no arguments".to_string());
            }
            Ok(Rc::new(crate::objects::none::PyNone::new()))
        })),
    );
    // complex(real, imag)
    env_mut.set(
        "complex".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("complex".to_string(), |args| {
            if args.len() != 2 {
                return Err(format!("TypeError: complex() takes exactly 2 arguments ({} given)", args.len()));
            }
            let real = if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                i.as_i64().unwrap_or(0) as f64
            } else if let Some(f) = args[0].as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                f.value
            } else {
                return Err(format!("TypeError: complex() real argument must be int or float, not '{}'", args[0].get_type()));
            };
            let imag = if let Some(i) = args[1].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                i.as_i64().unwrap_or(0) as f64
            } else if let Some(f) = args[1].as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                f.value
            } else {
                return Err(format!("TypeError: complex() imag argument must be int or float, not '{}'", args[1].get_type()));
            };
            Ok(Rc::new(crate::objects::complex::PyComplex::new(real, imag)))
        })),
    );

    // classmethod(function)
    env_mut.set(
        "classmethod".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("classmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: classmethod() takes exactly one argument".to_string());
            }
            Ok(Rc::new(crate::objects::classmethod::PyClassMethod::new(Rc::clone(&args[0]))))
        })),
    );

    // staticmethod(function)
    env_mut.set(
        "staticmethod".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("staticmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: staticmethod() takes exactly one argument".to_string());
            }
            Ok(Rc::new(crate::objects::staticmethod::PyStaticMethod::new(Rc::clone(&args[0]))))
        })),
    );

    // property(fget, fset=None, fdel=None, doc=None)
    env_mut.set(
        "property".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("property".to_string(), |args| {
            if args.is_empty() || args.len() > 4 {
                return Err("TypeError: property() takes 1-4 arguments".to_string());
            }
            let fget = Some(Rc::clone(&args[0]));
            let fset = if args.len() > 1 { Some(Rc::clone(&args[1])) } else { None };
            let fdel = if args.len() > 2 { Some(Rc::clone(&args[2])) } else { None };
            Ok(Rc::new(crate::objects::property::PyProperty::new(fget, fset, fdel)))
        })),
    );

    // super()
    env_mut.set(
        "super".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("super".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: super() takes exactly 2 arguments (type, obj)".to_string());
            }

            let type_obj = args[0].as_any().downcast_ref::<crate::objects::class::PyClass>()
                .ok_or_else(|| "TypeError: super() argument 1 must be type".to_string())?;
            let obj = args[1].as_any().downcast_ref::<crate::objects::instance::PyInstance>()
                .ok_or_else(|| "TypeError: super() argument 2 must be instance".to_string())?;

            let type_rc = Rc::new(type_obj.clone()); // Wait, this clones the class. We need the original Rc.
            // Oh! args[0] is Rc<dyn PyObject>. We can downcast the Rc directly if we wrote a method, 
            // but we can just use obj.class since the type_obj is usually the class.
            // Actually, we can clone args[0] and use it. Wait, PySuper takes Rc<PyClass>.
            // Since we can't get Rc<PyClass> out of Rc<dyn PyObject> easily without specialized traits,
            // we can change PySuper to take Rc<dyn PyObject> for type_obj instead, or just clone the PyClass.
            // Cloning PyClass is cheap because attributes are in Rc<RefCell>.
            let super_proxy = crate::objects::class::PySuper::new(type_rc, Rc::new(obj.clone()));
            Ok(Rc::new(super_proxy) as Rc<dyn PyObject>)
        })),
    );
    // abs(x)
    env_mut.set(
        "abs".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("abs".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: abs() takes exactly one argument".to_string());
            }
            match args[0].abs_op() {
                Some(result) => Ok(result),
                None => Err(format!("TypeError: bad operand type for abs(): '{}'", args[0].get_type())),
            }
        })),
    );

    // max(*args, key=None)
    env_mut.set(
        "max".to_string(),
        Rc::new(PyNativeFunction::new("max".to_string(), |args, kwargs| {
            if args.is_empty() {
                return Err("TypeError: max expected 1 argument, got 0".to_string());
            }
            let items: Vec<Rc<dyn PyObject>> = if args.len() == 1 {
                let iter = args[0].get_iter()?;
                let mut v = Vec::new();
                while let Some(item) = iter.get_next()? {
                    v.push(item);
                }
                v
            } else {
                args.to_vec()
            };
            if items.is_empty() {
                return Err("TypeError: max() arg is an empty sequence".to_string());
            }

            let key_fn = kwargs.get("key").cloned();

            let mut best = Rc::clone(&items[0]);
            let mut best_key = if let Some(ref kf) = key_fn {
                if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                    (native.func)(vec![Rc::clone(&items[0])], std::collections::HashMap::new())?
                } else { Rc::clone(&items[0]) }
            } else { Rc::clone(&items[0]) };

            for item in items.iter().skip(1) {
                let item_key = if let Some(ref kf) = key_fn {
                    if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                        (native.func)(vec![Rc::clone(item)], std::collections::HashMap::new())?
                    } else { Rc::clone(item) }
                } else { Rc::clone(item) };
                match item_key.lt(best_key.clone()) {
                    Some(result) => {
                        if !result.is_truthy() {
                            best_key = item_key;
                            best = Rc::clone(item);
                        }
                    }
                    None => {}
                }
            }
            Ok(best)
        })),
    );

    // min(*args, key=None)
    env_mut.set(
        "min".to_string(),
        Rc::new(PyNativeFunction::new("min".to_string(), |args, kwargs| {
            if args.is_empty() {
                return Err("TypeError: min expected 1 argument, got 0".to_string());
            }
            let items: Vec<Rc<dyn PyObject>> = if args.len() == 1 {
                let iter = args[0].get_iter()?;
                let mut v = Vec::new();
                while let Some(item) = iter.get_next()? {
                    v.push(item);
                }
                v
            } else {
                args.to_vec()
            };
            if items.is_empty() {
                return Err("TypeError: min() arg is an empty sequence".to_string());
            }

            let key_fn = kwargs.get("key").cloned();

            let mut best = Rc::clone(&items[0]);
            let mut best_key = if let Some(ref kf) = key_fn {
                if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                    (native.func)(vec![Rc::clone(&items[0])], std::collections::HashMap::new())?
                } else { Rc::clone(&items[0]) }
            } else { Rc::clone(&items[0]) };

            for item in items.iter().skip(1) {
                let item_key = if let Some(ref kf) = key_fn {
                    if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                        (native.func)(vec![Rc::clone(item)], std::collections::HashMap::new())?
                    } else { Rc::clone(item) }
                } else { Rc::clone(item) };
                match item_key.lt(best_key.clone()) {
                    Some(result) => {
                        if result.is_truthy() {
                            best_key = item_key;
                            best = Rc::clone(item);
                        }
                    }
                    None => {}
                }
            }
            Ok(best)
        })),
    );

    // sum(iterable, start=0)
    env_mut.set(
        "sum".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("sum".to_string(), |args| {
            if args.len() < 1 || args.len() > 2 {
                return Err("TypeError: sum expected at most 2 arguments".to_string());
            }
            let start = if args.len() >= 2 { Rc::clone(&args[1]) } else { Rc::new(crate::objects::int::PyInt::from_i64(0)) as Rc<dyn PyObject> };
            let iter = args[0].get_iter()?;
            let mut total = start;
            while let Some(item) = iter.get_next()? {
                let new_total = total.add(Rc::clone(&item));
                match new_total {
                    Some(result) => total = result,
                    None => return Err("TypeError: unsupported operand type(s) for + in sum()".to_string()),
                }
            }
            Ok(total)
        })),
    );

    // repr(obj)
    env_mut.set(
        "repr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("repr".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: repr() takes exactly one argument".to_string());
            }
            Ok(Rc::new(crate::objects::string::PyString::new(
                args[0].repr(),
            )))
        })),
    );

    // bin(x)
    env_mut.set(
        "bin".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("bin".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: bin() takes exactly one argument".to_string());
            }
            if let Some(i) = args[0]
                .as_any()
                .downcast_ref::<crate::objects::int::PyInt>()
            {
                Ok(Rc::new(crate::objects::string::PyString::new(format!(
                    "0b{:b}",
                    i.as_i64().unwrap_or(0)
                ))))
            } else {
                Err("TypeError: 'str' object cannot be interpreted as an integer".to_string())
            }
        })),
    );

    // hex(x)
    env_mut.set(
        "hex".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("hex".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: hex() takes exactly one argument".to_string());
            }
            if let Some(i) = args[0]
                .as_any()
                .downcast_ref::<crate::objects::int::PyInt>()
            {
                Ok(Rc::new(crate::objects::string::PyString::new(format!(
                    "0x{:x}",
                    i.as_i64().unwrap_or(0)
                ))))
            } else {
                Err("TypeError: 'str' object cannot be interpreted as an integer".to_string())
            }
        })),
    );
    // iter(object)
    env_mut.set(
        "iter".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("iter".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: iter() takes exactly one argument".to_string());
            }
            args[0].get_iter()
        })),
    );

    // next(iterator[, default])
    env_mut.set(
        "next".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("next".to_string(), |args| {
            if args.is_empty() || args.len() > 2 {
                return Err("TypeError: next expected at most 2 arguments".to_string());
            }
            match args[0].get_next() {
                Ok(Some(val)) => Ok(val),
                Ok(None) | Err(_) => {
                    if args.len() == 2 {
                        Ok(args[1].clone())
                    } else {
                        Err("StopIteration".to_string())
                    }
                }
            }
        })),
    );

    // all(iterable)
    env_mut.set(
        "all".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("all".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: all() takes exactly one argument".to_string());
            }
            let iter = args[0].get_iter()?;
            while let Some(item) = iter.get_next()? {
                if !item.is_truthy() {
                    return Ok(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Ok(Rc::new(crate::objects::bool::PyBool::new(true)))
        })),
    );

    // any(iterable)
    env_mut.set(
        "any".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("any".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: any() takes exactly one argument".to_string());
            }
            let iter = args[0].get_iter()?;
            while let Some(item) = iter.get_next()? {
                if item.is_truthy() {
                    return Ok(Rc::new(crate::objects::bool::PyBool::new(true)));
                }
            }
            Ok(Rc::new(crate::objects::bool::PyBool::new(false)))
        })),
    );
    // globals()
    let env_clone = Rc::clone(env);
    env_mut.set(
        "globals".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("globals".to_string(), move |args| {
            if !args.is_empty() {
                return Err("TypeError: globals() takes no arguments".to_string());
            }
            let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
            for (k, v) in env_clone.borrow().get_all_locals() {
                pairs.push((Rc::new(crate::objects::string::PyString::new(k)) as Rc<dyn PyObject>, v));
            }
            Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
        })),
    );

    // locals()
    let env_clone2 = Rc::clone(env);
    env_mut.set(
        "locals".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("locals".to_string(), move |args| {
            if !args.is_empty() {
                return Err("TypeError: locals() takes no arguments".to_string());
            }
            let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
            for (k, v) in env_clone2.borrow().get_all_locals() {
                pairs.push((Rc::new(crate::objects::string::PyString::new(k)) as Rc<dyn PyObject>, v));
            }
            Ok(Rc::new(crate::objects::dict::PyDict::from_pairs(pairs)))
        })),
    );
    // eval(expression, globals=None, locals=None)
    let env_for_eval = Rc::clone(env);
    env_mut.set(
        "eval".to_string(),
        Rc::new(PyNativeFunction::new("eval".to_string(), move |args, _kwargs| {
            if args.is_empty() || args.len() > 3 {
                return Err("TypeError: eval() takes at most 3 arguments".to_string());
            }
            let source = args[0].as_any()
                .downcast_ref::<PyString>()
                .ok_or_else(|| "TypeError: eval() arg 1 must be a string".to_string())?
                .value.clone();

            let lexer = Lexer::new(&source);
            let mut parser = Parser::new(lexer)
                .map_err(|e| format!("SyntaxError: {}", e))?;
            let expr = parser.parse_expression(0)
                .map_err(|e| format!("SyntaxError: {}", e))?;
            let compiler = Compiler::new("<string>".to_string());
            let code = compiler.compile_expression(&expr)
                .map_err(|e| format!("CompileError: {}", e))?;

            let mut frame = Frame::new(code, Rc::clone(&env_for_eval));
            let mut vm = VirtualMachine::new();
            let result = vm.run(&mut frame)?;

            Ok(result.unwrap_or_else(|| Rc::new(PyNone::new()) as Rc<dyn PyObject>))
        })),
    );

    // range(stop), range(start, stop), range(start, stop, step)
    env_mut.set(
        "range".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("range".to_string(), |args| {
            if args.is_empty() || args.len() > 3 {
                return Err(format!("TypeError: range() takes 1-3 arguments ({} given)", args.len()));
            }
            let to_i64 = |obj: &Rc<dyn PyObject>| -> Result<i64, String> {
                if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Ok(i.as_i64().unwrap_or(0))
                } else {
                    Err(format!("TypeError: '{}' object cannot be interpreted as an integer", obj.get_type()))
                }
            };
            let (start, stop, step) = match args.len() {
                1 => (0, to_i64(&args[0])?, 1),
                2 => (to_i64(&args[0])?, to_i64(&args[1])?, 1),
                3 => (to_i64(&args[0])?, to_i64(&args[1])?, to_i64(&args[2])?),
                _ => unreachable!(),
            };
            if step == 0 {
                return Err("ValueError: range() arg 3 must not be zero".to_string());
            }
            Ok(Rc::new(crate::objects::range::PyRange::new(start, stop, step)))
        })),
    );

    // exec(source, globals=None, locals=None)
    let env_for_exec = Rc::clone(env);
    env_mut.set(
        "exec".to_string(),
        Rc::new(PyNativeFunction::new("exec".to_string(), move |args, _kwargs| {
            if args.is_empty() || args.len() > 3 {
                return Err("TypeError: exec() takes at most 3 arguments".to_string());
            }
            let mut source = args[0].as_any()
                .downcast_ref::<PyString>()
                .ok_or_else(|| "TypeError: exec() arg 1 must be a string".to_string())?
                .value.clone();
            source.push('\n');

            let lexer = Lexer::new(&source);
            let mut parser = Parser::new(lexer)
                .map_err(|e| format!("SyntaxError: {}", e))?;
            let module = parser.parse_module()
                .map_err(|e| format!("SyntaxError: {}", e))?;
            let compiler = Compiler::new("<string>".to_string());
            let code = compiler.compile(&module)
                .map_err(|e| format!("CompileError: {}", e))?;

            let mut frame = Frame::new(code, Rc::clone(&env_for_exec));
            let mut vm = VirtualMachine::new();
            vm.run(&mut frame)?;

            Ok(Rc::new(PyNone::new()) as Rc<dyn PyObject>)
        })),
    );

    // compile(source, filename, mode, flags=0, dont_inherit=False, optimize=-1)
    env_mut.set(
        "compile".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("compile".to_string(), |args| {
            if args.len() < 3 {
                return Err("TypeError: compile() requires at least 3 arguments".to_string());
            }
            let source = args[0].as_any()
                .downcast_ref::<PyString>()
                .ok_or_else(|| "TypeError: compile() arg 1 must be a string".to_string())?
                .value.clone();
            let filename = args[1].as_any()
                .downcast_ref::<PyString>()
                .ok_or_else(|| "TypeError: compile() arg 2 must be a string".to_string())?
                .value.clone();
            let mode = args[2].as_any()
                .downcast_ref::<PyString>()
                .ok_or_else(|| "TypeError: compile() arg 3 must be a string".to_string())?
                .value.clone();

            let lexer = Lexer::new(&source);
            let mut parser = Parser::new(lexer)
                .map_err(|e| format!("SyntaxError: {}", e))?;

            match mode.as_str() {
                "exec" => {
                    let module = parser.parse_module()
                        .map_err(|e| format!("SyntaxError: {}", e))?;
                    let compiler = Compiler::new(filename);
                    let code = compiler.compile(&module)
                        .map_err(|e| format!("CompileError: {}", e))?;
                    Ok(Rc::new(code) as Rc<dyn PyObject>)
                }
                "eval" => {
                    let expr = parser.parse_expression(0)
                        .map_err(|e| format!("SyntaxError: {}", e))?;
                    let compiler = Compiler::new(filename);
                    let code = compiler.compile_expression(&expr)
                        .map_err(|e| format!("CompileError: {}", e))?;
                    Ok(Rc::new(code) as Rc<dyn PyObject>)
                }
                "single" => {
                    let module = parser.parse_module()
                        .map_err(|e| format!("SyntaxError: {}", e))?;
                    let compiler = Compiler::new(filename);
                    let code = compiler.compile(&module)
                        .map_err(|e| format!("CompileError: {}", e))?;
                    Ok(Rc::new(code) as Rc<dyn PyObject>)
                }
                _ => Err("ValueError: compile() mode must be 'exec', 'eval', or 'single'".to_string()),
            }
        })),
    );

    // chr(i) -> str
    env_mut.set(
        "chr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("chr".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: chr() takes exactly one argument".to_string()); }
            let val = if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                i.as_i64().unwrap_or(0)
            } else {
                return Err("TypeError: 'int' object expected".to_string());
            };
            if val < 0 || val > 0x10FFFF {
                return Err("ValueError: chr() arg not in range(0x110000)".to_string());
            }
            match char::from_u32(val as u32) {
                Some(c) => Ok(Rc::new(crate::objects::string::PyString::new(c.to_string()))),
                None => Err("ValueError: chr() arg not in range(0x110000)".to_string()),
            }
        })),
    );

    // ord(c) -> int
    env_mut.set(
        "ord".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("ord".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: ord() takes exactly one argument".to_string()); }
            let s = args[0].str();
            let c = s.chars().next().ok_or_else(|| "TypeError: ord() expected a character, not empty string".to_string())?;
            Ok(Rc::new(crate::objects::int::PyInt::from_i64(c as i64)))
        })),
    );

    // pow(base, exp, mod=None) -> number
    env_mut.set(
        "pow".to_string(),
        Rc::new(PyNativeFunction::new("pow".to_string(), |args, kwargs| {
            if args.len() < 2 || args.len() > 3 {
                return Err("TypeError: pow() takes 2-3 arguments".to_string());
            }
            let base = &args[0];
            let exp = &args[1];
            let result = match base.pow(Rc::clone(exp)) {
                Some(result) => result,
                None => return Err(format!("TypeError: unsupported operand type(s) for pow(): '{}' and '{}'", base.get_type(), exp.get_type())),
            };
            if args.len() == 3 {
                let mod_val = &args[2];
                match result.modulo(Rc::clone(mod_val)) {
                    Some(mod_result) => Ok(mod_result),
                    None => Err("TypeError: pow() 3rd argument not allowed unless all arguments are integers".to_string()),
                }
            } else if let Some(mod_val) = kwargs.get("mod") {
                match result.modulo(Rc::clone(mod_val)) {
                    Some(mod_result) => Ok(mod_result),
                    None => Err("TypeError: pow() 3rd argument not allowed unless all arguments are integers".to_string()),
                }
            } else {
                Ok(result)
            }
        })),
    );

    // round(x, ndigits=None) -> int or float
    env_mut.set(
        "round".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("round".to_string(), |args| {
            if args.len() < 1 || args.len() > 2 { return Err("TypeError: round() takes 1-2 arguments".to_string()); }
            let ndigits = if args.len() >= 2 {
                if let Some(i) = args[1].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Some(i.as_i64().unwrap_or(0))
                } else {
                    return Err("TypeError: 'int' object expected".to_string());
                }
            } else { None };
            if let Some(f) = args[0].as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                match ndigits {
                    Some(n) => {
                        let factor = 10f64.powi(n as i32);
                        Ok(Rc::new(crate::objects::float::PyFloat::new((f.value * factor).round() / factor)))
                    }
                    None => Ok(Rc::new(crate::objects::int::PyInt::from_i64(f.value.round() as i64))),
                }
            } else if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(i.as_i64().unwrap_or(0))))
            } else {
                Err(format!("TypeError: type {} doesn't define __round__", args[0].get_type()))
            }
        })),
    );

    // sorted(iterable, key=None, reverse=False) -> list
    env_mut.set(
        "sorted".to_string(),
        Rc::new(PyNativeFunction::new("sorted".to_string(), |args, kwargs| {
            if args.len() != 1 { return Err("TypeError: sorted() takes exactly 1 positional argument".to_string()); }
            let iter = args[0].get_iter()?;
            let mut items: Vec<Rc<dyn PyObject>> = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }

            let reverse = kwargs.get("reverse").map_or(false, |v| v.is_truthy());
            let key_fn = kwargs.get("key").cloned();

            // Bubble sort using lt()
            let n = items.len();
            for i in 0..n {
                for j in 0..n-1-i {
                    let do_swap = {
                        let a_val = if let Some(ref kf) = key_fn {
                            if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                                (native.func)(vec![Rc::clone(&items[j])], std::collections::HashMap::new())?
                            } else { Rc::clone(&items[j]) }
                        } else { Rc::clone(&items[j]) };
                        let b_val = if let Some(ref kf) = key_fn {
                            if let Some(native) = kf.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                                (native.func)(vec![Rc::clone(&items[j+1])], std::collections::HashMap::new())?
                            } else { Rc::clone(&items[j+1]) }
                        } else { Rc::clone(&items[j+1]) };
                        match a_val.lt(b_val) {
                            Some(result) => !result.is_truthy(),
                            None => false,
                        }
                    };
                    if do_swap {
                        items.swap(j, j+1);
                    }
                }
            }
            if reverse { items.reverse(); }
            Ok(Rc::new(crate::objects::list::PyList::new(items)))
        })),
    );

    // reversed(seq) -> iterator (eager: returns a reversed list)
    env_mut.set(
        "reversed".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("reversed".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: reversed() takes exactly one argument".to_string()); }
            let iter = args[0].get_iter()?;
            let mut items: Vec<Rc<dyn PyObject>> = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            items.reverse();
            Ok(Rc::new(crate::objects::list::PyList::new(items)))
        })),
    );

    // enumerate(iterable, start=0) -> list of (index, value) tuples
    env_mut.set(
        "enumerate".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("enumerate".to_string(), |args| {
            if args.len() < 1 || args.len() > 2 { return Err("TypeError: enumerate() takes 1-2 arguments".to_string()); }
            let start = if args.len() >= 2 {
                if let Some(i) = args[1].as_any().downcast_ref::<crate::objects::int::PyInt>() { i.as_i64().unwrap_or(0) } else { 0 }
            } else { 0 };
            let iter = args[0].get_iter()?;
            let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
            let mut idx = start;
            while let Some(item) = iter.get_next()? {
                let pair = vec![
                    Rc::new(crate::objects::int::PyInt::from_i64(idx)) as Rc<dyn PyObject>,
                    item,
                ];
                result.push(Rc::new(crate::objects::tuple::PyTuple::new(pair)) as Rc<dyn PyObject>);
                idx += 1;
            }
            Ok(Rc::new(crate::objects::list::PyList::new(result)))
        })),
    );

    // map(func, iterable) -> list (eager; only native functions for now)
    env_mut.set(
        "map".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("map".to_string(), |args| {
            if args.len() != 2 { return Err("TypeError: map() takes exactly 2 arguments".to_string()); }
            if let Some(native) = args[0].as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                let iter = args[1].get_iter()?;
                let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
                while let Some(item) = iter.get_next()? {
                    result.push((native.func)(vec![item], std::collections::HashMap::new())?);
                }
                Ok(Rc::new(crate::objects::list::PyList::new(result)))
            } else {
                Err("TypeError: map() currently only supports native functions".to_string())
            }
        })),
    );

    // filter(func, iterable) -> list (eager; only native functions for now)
    env_mut.set(
        "filter".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("filter".to_string(), |args| {
            if args.len() != 2 { return Err("TypeError: filter() takes exactly 2 arguments".to_string()); }
            if let Some(native) = args[0].as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                let iter = args[1].get_iter()?;
                let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
                while let Some(item) = iter.get_next()? {
                    let should_keep = (native.func)(vec![Rc::clone(&item)], std::collections::HashMap::new())?;
                    if should_keep.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Rc::new(crate::objects::list::PyList::new(result)))
            } else {
                Err("TypeError: filter() currently only supports native functions".to_string())
            }
        })),
    );

    // zip(*iterables) -> list of lists (eager)
    env_mut.set(
        "zip".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("zip".to_string(), |args| {
            if args.is_empty() { return Ok(Rc::new(crate::objects::list::PyList::new(Vec::new()))); }
            let mut iters: Vec<Rc<dyn PyObject>> = Vec::new();
            for arg in args {
                iters.push(arg.get_iter()?);
            }
            let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
            loop {
                let mut group: Vec<Rc<dyn PyObject>> = Vec::new();
                for iter in &iters {
                    match iter.get_next()? {
                        Some(item) => group.push(item),
                        None => {
                            return Ok(Rc::new(crate::objects::list::PyList::new(result)));
                        }
                    }
                }
                result.push(Rc::new(crate::objects::tuple::PyTuple::new(group)) as Rc<dyn PyObject>);
            }
        })),
    );

    // bytes(source) -> bytes
    env_mut.set(
        "bytes".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("bytes".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(PyBytes::new(Vec::new())));
            }
            if args.len() > 2 {
                return Err(format!("TypeError: bytes() takes at most 2 arguments ({} given)", args.len()));
            }
            let obj = &args[0];
            // bytes(integer) -> zero-initialized bytes of that length
            if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                let n = i.as_i64().unwrap_or(0);
                if n < 0 {
                    return Err("ValueError: negative count".to_string());
                }
                return Ok(Rc::new(PyBytes::new(vec![0u8; n as usize])));
            }
            // bytes(iterable_of_ints)
            if let Ok(iter) = obj.get_iter() {
                let mut result = Vec::new();
                while let Some(item) = iter.get_next()? {
                    if let Some(n) = item.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        let val = n.as_i64().unwrap_or(0);
                        if val < 0 || val > 255 {
                            return Err(format!("ValueError: bytes must be in range(0, 256)"));
                        }
                        result.push(val as u8);
                    } else {
                        return Err(format!("TypeError: '{}' object cannot be interpreted as an integer", item.get_type()));
                    }
                }
                return Ok(Rc::new(PyBytes::new(result)));
            }
            Err(format!("TypeError: cannot convert '{}' object to bytes", obj.get_type()))
        })),
    );

    // bytearray(source, encoding) -> bytearray
    env_mut.set(
        "bytearray".to_string(),
        Rc::new(PyNativeFunction::new("bytearray".to_string(), |args, _kwargs| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::bytearray::PyByteArray::new(Vec::new())));
            }
            if args.len() > 2 {
                return Err(format!("TypeError: bytearray() takes at most 2 arguments ({} given)", args.len()));
            }
            let obj = &args[0];
            // bytearray(integer) -> zero-initialized of that length
            if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                let n = i.as_i64().unwrap_or(0);
                if n < 0 {
                    return Err("ValueError: negative count".to_string());
                }
                return Ok(Rc::new(crate::objects::bytearray::PyByteArray::new(vec![0u8; n as usize])));
            }
            // bytearray(string, encoding)
            if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
                if args.len() < 2 {
                    return Err("TypeError: bytearray() argument 1 must be str, not 'str' (use encoding argument)".to_string());
                }
                let encoding = if let Some(e) = args[1].as_any().downcast_ref::<PyString>() {
                    e.value.clone()
                } else {
                    return Err("TypeError: encoding must be str".to_string());
                };
                if encoding != "utf-8" && encoding != "utf8" {
                    return Err(format!("LookupError: unknown encoding: '{}'", encoding));
                }
                return Ok(Rc::new(crate::objects::bytearray::PyByteArray::new(s.value.as_bytes().to_vec())));
            }
            // bytearray(iterable_of_ints)
            if let Ok(iter) = obj.get_iter() {
                let mut result = Vec::new();
                while let Some(item) = iter.get_next()? {
                    if let Some(n) = item.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        let val = n.as_i64().unwrap_or(0);
                        if val < 0 || val > 255 {
                            return Err(format!("ValueError: byte must be in range(0, 256)"));
                        }
                        result.push(val as u8);
                    } else {
                        return Err(format!("TypeError: '{}' object cannot be interpreted as an integer", item.get_type()));
                    }
                }
                return Ok(Rc::new(crate::objects::bytearray::PyByteArray::new(result)));
            }
            Err(format!("TypeError: cannot convert '{}' object to bytearray", obj.get_type()))
        })),
    );

    // set(iterable) -> set
    env_mut.set(
        "set".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("set".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::set::PySet::new(Vec::new())));
            }
            if args.len() != 1 {
                return Err("TypeError: set() takes at most 1 argument".to_string());
            }
            let iter = args[0].get_iter()?;
            let mut items = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            Ok(Rc::new(crate::objects::set::PySet::new(items)))
        })),
    );

    // frozenset(iterable) -> frozenset
    env_mut.set(
        "frozenset".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("frozenset".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::set::PyFrozenSet::new(Vec::new())));
            }
            if args.len() != 1 {
                return Err("TypeError: frozenset() takes at most 1 argument".to_string());
            }
            let iter = args[0].get_iter()?;
            let mut items = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            Ok(Rc::new(crate::objects::set::PyFrozenSet::new(items)))
        })),
    );

    // open(file, mode='r', encoding=None)
    env_mut.set(
        "open".to_string(),
        Rc::new(PyNativeFunction::new("open".to_string(), |args, kwargs| {
            if args.is_empty() {
                return Err("TypeError: open() missing required argument 'file'".to_string());
            }
            let path = args[0].str();
            let mode = if let Some(v) = kwargs.get("mode") {
                v.str()
            } else if args.len() >= 2 {
                args[1].str()
            } else {
                "r".to_string()
            };
            // Validate mode
            if mode.is_empty() {
                return Err("ValueError: empty mode".to_string());
            }
            let valid_chars = ['r', 'w', 'a', 'x', 'b', 't', '+'];
            for c in mode.chars() {
                if !valid_chars.contains(&c) {
                    return Err(format!("ValueError: invalid mode: '{}'", mode));
                }
            }
            if mode.chars().filter(|&c| c == 'r' || c == 'w' || c == 'a' || c == 'x').count() != 1 {
                return Err(format!("ValueError: invalid mode: '{}'", mode));
            }
            // Parse mode into OpenOptions
            let main_mode = mode.chars().find(|&c| c == 'r' || c == 'w' || c == 'a' || c == 'x').unwrap();
            let plus = mode.contains('+');

            let mut opts = std::fs::OpenOptions::new();
            match main_mode {
                'r' => {
                    opts.read(true);
                    if plus { opts.write(true); }
                }
                'w' => {
                    opts.write(true);
                    opts.truncate(true);
                    opts.create(true);
                    if plus { opts.read(true); }
                }
                'a' => {
                    opts.write(true);
                    opts.append(true);
                    opts.create(true);
                    if plus { opts.read(true); }
                }
                'x' => {
                    opts.write(true);
                    opts.create_new(true);
                    if plus { opts.read(true); }
                }
                _ => unreachable!(),
            }

            match opts.open(&path) {
                Ok(file) => Ok(Rc::new(crate::objects::file::PyFile::from_file(path, mode, file)) as Rc<dyn PyObject>),
                Err(e) => {
                    let msg = format!("{}", e);
                    if msg.contains("No such file or directory") {
                        Err(format!("FileNotFoundError: [Errno 2] No such file or directory: '{}'", path))
                    } else if msg.contains("Permission denied") {
                        Err(format!("PermissionError: [Errno 13] Permission denied: '{}'", path))
                    } else if msg.contains("File exists") {
                        Err(format!("FileExistsError: [Errno 17] File exists: '{}'", path))
                    } else {
                        Err(format!("OSError: {}: '{}'", msg, path))
                    }
                }
            }
        })),
    );

    // Drop env_mut so we can borrow env again below
    drop(env_mut);

    // Initialize import system
    let import_system = Rc::new(ImportSystem::new());
    *import_system.builtins_env.borrow_mut() = Some(Rc::clone(env));

    // Create sys module
    let argv: Vec<String> = std::env::args().collect();
    let sys_module = crate::stdlib::sys::create_sys_module(
        Rc::clone(&import_system.sys_modules),
        argv,
    );
    import_system.register_native_module("sys", Rc::clone(&sys_module));

    // Register sys in sys.modules
    let sys_key = Rc::new(PyString::new("sys".to_string())) as Rc<dyn PyObject>;
    let _ = import_system
        .sys_modules
        .set_item(Rc::clone(&sys_key), Rc::clone(&sys_module) as Rc<dyn PyObject>);

    // Create math_native module
    let math_module = Rc::new(PyModule::new("math_native".to_string()));
    math_module.set_attr_inner(
        "sqrt",
        Rc::new(PyNativeFunction::new_pos_only("sqrt".to_string(), move |args| {
            if args.len() != 1 {
                return Err("TypeError: sqrt() takes exactly one argument".to_string());
            }
            let val = if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                i.as_i64().unwrap_or(0) as f64
            } else if let Some(f) = args[0]
                .as_any()
                .downcast_ref::<crate::objects::float::PyFloat>()
            {
                f.value
            } else {
                return Err("TypeError: sqrt() argument must be int or float".to_string());
            };
            Ok(Rc::new(crate::objects::float::PyFloat::new(val.sqrt())))
        })) as Rc<dyn PyObject>,
    );
    import_system.register_native_module("math_native", Rc::clone(&math_module));

    // Create asyncio module
    let asyncio_module = Rc::new(PyModule::new("asyncio".to_string()));
    asyncio_module.set_attr_inner(
        "run",
        Rc::new(PyNativeFunction::new_pos_only("run".to_string(), move |args| {
            if args.len() != 1 {
                return Err("TypeError: run() takes exactly one argument (the coroutine)".to_string());
            }
            let coro = &args[0];
            // Get the __await__ iterator and run to completion
            let await_method = coro.get_attr("__await__")?;
            let mut vm = crate::vm::VirtualMachine::new();
            let iterator = vm.invoke(await_method, vec![], std::collections::HashMap::new())?;
            loop {
                match iterator.get_next()? {
                    Some(_val) => {
                        // The coroutine yielded (intermediate await), continue
                    }
                    None => {
                        // Coroutine completed. Get the return value.
                        if let Some(coro_obj) = coro.as_any()
                            .downcast_ref::<crate::objects::coroutine::PyCoroutine>()
                        {
                            let f = coro_obj.frame.borrow();
                            let result = f.return_value.clone()
                                .unwrap_or_else(|| Rc::new(crate::objects::none::PyNone));
                            return Ok(result);
                        }
                        return Ok(Rc::new(crate::objects::none::PyNone));
                    }
                }
            }
        })) as Rc<dyn PyObject>,
    );
    import_system.register_native_module("asyncio", Rc::clone(&asyncio_module));

    // Create math module
    let math_module = crate::stdlib::math::create_math_module();
    import_system.register_native_module("math", Rc::clone(&math_module));

    // Create os module
    let os_module = crate::stdlib::os::create_os_module();
    import_system.register_native_module("os", Rc::clone(&os_module));

    // builtins module
    let builtins_module = Rc::new(PyModule::new("builtins".to_string()));
    {
        let env_b = env.borrow();
        for (k, v) in env_b.get_all_locals() {
            builtins_module.set_attr_inner(&k, v);
        }
    }
    import_system.register_native_module("builtins", Rc::clone(&builtins_module));

    // __import__ builtin
    let import_system_for_import = Rc::clone(&import_system);
    let mut env_mut2 = env.borrow_mut();
    env_mut2.set(
        "__import__".to_string(),
        Rc::new(PyNativeFunction::new_pos_only(
            "__import__".to_string(),
            move |args| {
                if args.is_empty() {
                    return Err("TypeError: __import__() missing required argument: name".to_string());
                }
                let name = args[0].as_any().downcast_ref::<PyString>()
                    .map(|s| s.value.clone())
                    .ok_or_else(|| "TypeError: __import__() argument 1 must be str".to_string())?;
                let result = import_system_for_import.import_module(&name)?;
                Ok(result)
            },
        )),
    );

    // float(x=0.0) -> float
    env_mut2.set(
        "float".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("float".to_string(), |args| {
            let val = if args.is_empty() {
                0.0
            } else if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                use num_traits::ToPrimitive;
                i.value.to_f64().unwrap_or(0.0)
            } else if let Some(f) = args[0].as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                f.value
            } else if let Some(s) = args[0].as_any().downcast_ref::<PyString>() {
                s.value.parse::<f64>().map_err(|_| format!("ValueError: could not convert string to float: '{}'", s.value))?
            } else {
                return Err(format!("TypeError: float() argument must be a string or a number, not '{}'", args[0].get_type()));
            };
            Ok(Rc::new(crate::objects::float::PyFloat::new(val)) as Rc<dyn PyObject>)
        })),
    );

    // oct(x) -> str
    env_mut2.set(
        "oct".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("oct".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: oct() takes exactly one argument".to_string());
            }
            let _val = if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                i.repr()
            } else {
                let i: i64 = args[0].str().parse().map_err(|_| format!("TypeError: oct() argument must be an int"))?;
                format!("{}", i)
            };
            // Convert to oct - use the int's oct representation
            let s = args[0].str();
            let n: i64 = s.parse().unwrap_or(0);
            Ok(Rc::new(PyString::new(format!("0o{:o}", n))) as Rc<dyn PyObject>)
        })),
    );

    // ascii(obj) -> str
    env_mut2.set(
        "ascii".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("ascii".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: ascii() takes exactly one argument".to_string());
            }
            let r = args[0].repr();
            let mut ascii_out = String::new();
            for c in r.chars() {
                if c.is_ascii() {
                    ascii_out.push(c);
                } else {
                    let code = c as u32;
                    if code < 0x10000 {
                        ascii_out.push_str(&format!("\\u{:04x}", code));
                    } else {
                        ascii_out.push_str(&format!("\\U{:08x}", code));
                    }
                }
            }
            Ok(Rc::new(PyString::new(ascii_out)) as Rc<dyn PyObject>)
        })),
    );

    // divmod(a, b) -> (a // b, a % b)
    env_mut2.set(
        "divmod".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("divmod".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: divmod() takes exactly 2 arguments".to_string());
            }
            let a = Rc::clone(&args[0]);
            let b = Rc::clone(&args[1]);
            let div = a.floordiv(b.clone()).ok_or_else(|| "TypeError: unsupported operand type(s) for divmod()".to_string())?;
            let rem = a.modulo(b).ok_or_else(|| "TypeError: unsupported operand type(s) for divmod()".to_string())?;
            Ok(Rc::new(crate::objects::tuple::PyTuple::new(vec![div, rem])) as Rc<dyn PyObject>)
        })),
    );

    // delattr(obj, name) -> None
    env_mut2.set(
        "delattr".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("delattr".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: delattr() takes exactly 2 arguments".to_string());
            }
            let name = if let Some(s) = args[1].as_any().downcast_ref::<PyString>() {
                s.value.clone()
            } else {
                return Err("TypeError: delattr() argument 2 must be str".to_string());
            };
            args[0].del_attr(&name)?;
            Ok(Rc::new(PyNone::new()))
        })),
    );

    // dir(obj) -> list
    env_mut2.set(
        "dir".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("dir".to_string(), |args| {
            let obj = if args.is_empty() {
                return Err("TypeError: dir() expected at least 1 argument, got 0".to_string());
            } else {
                Rc::clone(&args[0])
            };
            let mut names: Vec<String> = Vec::new();
            // Common attributes for all objects
            let common_attrs = vec!["__class__", "__doc__", "__init__", "__repr__", "__str__", "__dict__", "__module__", "__new__", "__delattr__", "__format__", "__getattribute__", "__hash__", "__setattr__", "__sizeof__", "__subclasshook__"];
            for attr in &common_attrs {
                if obj.get_attr(attr).is_ok() {
                    names.push(attr.to_string());
                }
            }
            // Try to list from __dict__
            if let Ok(d) = obj.get_attr("__dict__") {
                if let Some(dict) = d.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                    for bucket in dict.entries.borrow().values() {
                        for (k, _) in bucket {
                            let name = k.str();
                            if !names.contains(&name) {
                                names.push(name);
                            }
                        }
                    }
                }
            }
            // Type-specific attributes
            let type_attrs: &[&str] = match obj.get_type() {
                "list" => &["append", "clear", "copy", "count", "extend", "index", "insert", "pop", "remove", "reverse", "sort", "__add__", "__iadd__", "__imul__", "__mul__", "__reversed__"],
                "str" => &["capitalize", "casefold", "center", "count", "encode", "endswith", "expandtabs", "find", "format", "index", "isalnum", "isalpha", "isascii", "isdecimal", "isdigit", "isidentifier", "islower", "isnumeric", "isprintable", "isspace", "istitle", "isupper", "join", "ljust", "lower", "lstrip", "maketrans", "partition", "removeprefix", "removesuffix", "replace", "rfind", "rindex", "rjust", "rpartition", "rsplit", "rstrip", "split", "splitlines", "startswith", "strip", "swapcase", "title", "translate", "upper", "zfill", "__add__", "__contains__", "__eq__", "__ge__", "__getitem__", "__gt__", "__hash__", "__iter__", "__le__", "__len__", "__lt__", "__mod__", "__mul__", "__ne__", "__rmod__", "__rmul__"],
                "int" => &["as_integer_ratio", "bit_count", "bit_length", "conjugate", "denominator", "from_bytes", "imag", "numerator", "real", "to_bytes", "__abs__", "__add__", "__and__", "__bool__", "__ceil__", "__divmod__", "__eq__", "__float__", "__floor__", "__floordiv__", "__ge__", "__gt__", "__index__", "__int__", "__invert__", "__le__", "__lshift__", "__lt__", "__mod__", "__mul__", "__ne__", "__neg__", "__or__", "__pos__", "__pow__", "__radd__", "__rand__", "__rfloordiv__", "__rlshift__", "__rmod__", "__rmul__", "__ror__", "__round__", "__rpow__", "__rrshift__", "__rshift__", "__rsub__", "__rtruediv__", "__rxor__", "__sub__", "__truediv__", "__trunc__", "__xor__"],
                "float" => &["as_integer_ratio", "conjugate", "fromhex", "hex", "imag", "is_integer", "real", "__abs__", "__add__", "__bool__", "__ceil__", "__divmod__", "__eq__", "__float__", "__floor__", "__floordiv__", "__ge__", "__gt__", "__int__", "__le__", "__lt__", "__mod__", "__mul__", "__ne__", "__neg__", "__pos__", "__pow__", "__radd__", "__rdivmod__", "__rfloordiv__", "__rmod__", "__rmul__", "__round__", "__rpow__", "__rsub__", "__rtruediv__", "__sub__", "__truediv__", "__trunc__"],
                "dict" => &["clear", "copy", "fromkeys", "get", "items", "keys", "pop", "popitem", "setdefault", "update", "values", "__contains__", "__delitem__", "__eq__", "__ge__", "__getitem__", "__gt__", "__iter__", "__le__", "__len__", "__lt__", "__ne__", "__or__", "__ror__", "__reversed__", "__setitem__", "__sizeof__"],
                "tuple" => &["count", "index", "__add__", "__contains__", "__eq__", "__ge__", "__getitem__", "__gt__", "__hash__", "__iter__", "__le__", "__len__", "__lt__", "__mul__", "__ne__", "__rmul__"],
                "set" => &["add", "clear", "copy", "difference", "difference_update", "discard", "intersection", "intersection_update", "isdisjoint", "issubset", "issuperset", "pop", "remove", "symmetric_difference", "symmetric_difference_update", "union", "update", "__and__", "__contains__", "__eq__", "__ge__", "__gt__", "__iand__", "__ior__", "__isub__", "__ixor__", "__le__", "__len__", "__lt__", "__ne__", "__or__", "__rand__", "__ror__", "__rsub__", "__rxor__", "__sub__", "__xor__"],
                "frozenset" => &["copy", "difference", "intersection", "isdisjoint", "issubset", "issuperset", "symmetric_difference", "union", "__and__", "__contains__", "__eq__", "__ge__", "__gt__", "__hash__", "__le__", "__len__", "__lt__", "__ne__", "__or__", "__rand__", "__ror__", "__rsub__", "__rxor__", "__sub__", "__xor__"],
                "bytes" => &["capitalize", "center", "count", "decode", "endswith", "expandtabs", "find", "fromhex", "hex", "index", "isalnum", "isalpha", "isascii", "isdigit", "islower", "isspace", "istitle", "isupper", "join", "ljust", "lower", "lstrip", "maketrans", "partition", "removeprefix", "removesuffix", "replace", "rfind", "rindex", "rjust", "rpartition", "rsplit", "rstrip", "split", "splitlines", "startswith", "strip", "swapcase", "title", "translate", "upper", "zfill"],
                "bytearray" => &["append", "capitalize", "center", "clear", "copy", "count", "decode", "endswith", "expandtabs", "extend", "find", "fromhex", "hex", "index", "insert", "isalnum", "isalpha", "isascii", "isdigit", "islower", "isspace", "istitle", "isupper", "join", "ljust", "lower", "lstrip", "maketrans", "partition", "pop", "remove", "removeprefix", "removesuffix", "replace", "reverse", "rfind", "rindex", "rjust", "rpartition", "rsplit", "rstrip", "sort", "split", "splitlines", "startswith", "strip", "swapcase", "title", "translate", "upper", "zfill"],
                "range" => &["count", "index", "start", "stop", "step", "__contains__", "__eq__", "__ge__", "__getitem__", "__gt__", "__hash__", "__iter__", "__le__", "__len__", "__lt__", "__ne__", "__reversed__"],
                "bool" => &["__abs__", "__add__", "__and__", "__bool__", "__ceil__", "__divmod__", "__eq__", "__float__", "__floor__", "__floordiv__", "__ge__", "__gt__", "__index__", "__int__", "__invert__", "__le__", "__lshift__", "__lt__", "__mod__", "__mul__", "__ne__", "__neg__", "__or__", "__pos__", "__pow__", "__radd__", "__rand__", "__rfloordiv__", "__rlshift__", "__rmod__", "__rmul__", "__ror__", "__round__", "__rpow__", "__rrshift__", "__rshift__", "__rsub__", "__rtruediv__", "__rxor__", "__sub__", "__truediv__", "__trunc__", "__xor__", "as_integer_ratio", "bit_count", "bit_length", "conjugate", "denominator", "from_bytes", "imag", "numerator", "real", "to_bytes"],
                "complex" => &["conjugate", "imag", "real", "__abs__", "__add__", "__bool__", "__divmod__", "__eq__", "__float__", "__floordiv__", "__ge__", "__gt__", "__int__", "__le__", "__lt__", "__mod__", "__mul__", "__ne__", "__neg__", "__pos__", "__pow__", "__radd__", "__rdivmod__", "__rfloordiv__", "__rmod__", "__rmul__", "__rpow__", "__rsub__", "__rtruediv__", "__sub__", "__truediv__"],
                "slice" => &["indices", "start", "stop", "step", "__eq__", "__ge__", "__gt__", "__hash__", "__le__", "__lt__", "__ne__", "__reduce__"],
                _ => &[],
            };
            for attr in type_attrs {
                let name = attr.to_string();
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            // Sort names
            let mut sorted: Vec<Rc<dyn PyObject>> = names.iter().map(|s| Rc::new(PyString::new(s.clone())) as Rc<dyn PyObject>).collect();
            sorted.sort_by(|a, b| a.str().cmp(&b.str()));
            Ok(Rc::new(crate::objects::list::PyList::new(sorted)) as Rc<dyn PyObject>)
        })),
    );

    // vars(obj) -> dict
    env_mut2.set(
        "vars".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("vars".to_string(), |args| {
            let obj = if args.is_empty() {
                return Err("TypeError: vars() expected at least 1 argument, got 0".to_string());
            } else {
                Rc::clone(&args[0])
            };
            obj.get_attr("__dict__")
        })),
    );

    // slice(stop) or slice(start, stop, step) -> slice object
    env_mut2.set(
        "slice".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("slice".to_string(), |args| {
            let (start, stop, step): (Option<i64>, Option<i64>, Option<i64>) = match args.len() {
                1 => {
                    let stop = args[0].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    (None, Some(stop), None)
                }
                2 => {
                    let start = args[0].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    let stop = args[1].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    (Some(start), Some(stop), None)
                }
                3 => {
                    let start = args[0].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    let stop = args[1].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    let step = args[2].str().parse::<i64>().map_err(|_| "TypeError: slice indices must be integers".to_string())?;
                    (Some(start), Some(stop), Some(step))
                }
                _ => return Err("TypeError: slice() takes at most 3 arguments".to_string()),
            };
            let s = crate::objects::slice::PySlice::new(start, stop, step);
            Ok(Rc::new(s) as Rc<dyn PyObject>)
        })),
    );

    // input(prompt='') -> str
    env_mut2.set(
        "input".to_string(),
        Rc::new(PyNativeFunction::new("input".to_string(), |args, kwargs| {
            let prompt = if let Some(v) = kwargs.get("prompt") {
                v.str()
            } else if !args.is_empty() {
                args[0].str()
            } else {
                String::new()
            };
            if !prompt.is_empty() {
                print!("{}", prompt);
                std::io::stdout().flush().ok();
            }
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).map_err(|e| format!("OSError: {}", e))?;
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') { line.pop(); }
            }
            Ok(Rc::new(PyString::new(line)) as Rc<dyn PyObject>)
        })),
    );

    // issubclass(cls, classinfo) -> bool
    env_mut2.set(
        "issubclass".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("issubclass".to_string(), |args| {
            if args.len() != 2 {
                return Err("TypeError: issubclass() takes exactly 2 arguments".to_string());
            }
            let cls = Rc::clone(&args[0]);
            let classinfo = Rc::clone(&args[1]);
            if let Some(cls_class) = cls.as_any().downcast_ref::<crate::objects::class::PyClass>() {
                if let Some(info_class) = classinfo.as_any().downcast_ref::<crate::objects::class::PyClass>() {
                    let is_sub = cls_class.mro.iter().any(|c| {
                        if let Some(c_cls) = c.as_any().downcast_ref::<crate::objects::class::PyClass>() {
                            c_cls.name == info_class.name
                        } else { false }
                    });
                    return Ok(Rc::new(crate::objects::bool::PyBool::new(is_sub)) as Rc<dyn PyObject>);
                }
            }
            let cls_name = cls.get_type();
            let info_name = classinfo.get_type();
            Ok(Rc::new(crate::objects::bool::PyBool::new(cls_name == info_name)) as Rc<dyn PyObject>)
        })),
    );

    // format(value, format_spec='') -> str
    env_mut2.set(
        "format".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("format".to_string(), |args| {
            if args.is_empty() {
                return Err("TypeError: format() takes at least 1 argument".to_string());
            }
            let format_spec = if args.len() >= 2 { args[1].str() } else { String::new() };
            if format_spec.is_empty() {
                Ok(Rc::new(PyString::new(args[0].str())) as Rc<dyn PyObject>)
            } else {
                // Try __format__ method
                if let Ok(fmt_fn) = args[0].get_attr("__format__") {
                    let formatted = if let Some(native) = fmt_fn.as_any().downcast_ref::<PyNativeFunction>() {
                        (native.func)(vec![Rc::new(PyString::new(format_spec))], std::collections::HashMap::new())?
                    } else {
                        return Err("TypeError: unsupported format string".to_string());
                    };
                    Ok(formatted)
                } else {
                    Err("TypeError: unsupported format string".to_string())
                }
            }
        })),
    );

    // help(obj) -> prints help
    env_mut2.set(
        "help".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("help".to_string(), |args| {
            if args.is_empty() {
                return Err("TypeError: help() takes at least 1 argument".to_string());
            }
            let obj = &args[0];
            let type_name = obj.get_type();
            let mut doc = String::new();
            if let Ok(d) = obj.get_attr("__doc__") {
                let s = d.str();
                if !s.is_empty() && s != "None" {
                    doc = s;
                }
            }
            let doc_str = if doc.is_empty() { String::new() } else { format!("\n    {}", doc) };
            let help_text = format!("Help on {} object:\n\nclass {}\n |{}{}\n", type_name, type_name, doc_str, "\n\n");
            print!("{}", help_text);
            std::io::stdout().flush().ok();
            Ok(Rc::new(PyNone::new()))
        })),
    );

    // memoryview(obj) -> memoryview
    env_mut2.set(
        "memoryview".to_string(),
        Rc::new(PyNativeFunction::new_pos_only("memoryview".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: memoryview() takes exactly one argument".to_string());
            }
            // Basic implementation: return a proxy object
            Err("NotImplementedError: memoryview() is not yet implemented".to_string())
        })),
    );
}
