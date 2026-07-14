use crate::objects::PyObject;
use crate::objects::bytes::PyBytes;
use crate::objects::int::PyInt;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use crate::runtime::Environment;
use std::cell::RefCell;
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
            Rc::new(PyNativeFunction::new(exc_name.clone(), move |args| {
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

    // print(*args)
    env_mut.set(
        "print".to_string(),
        Rc::new(PyNativeFunction::new("print".to_string(), |args| {
            let mut out = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                out.push_str(&arg.str());
            }
            println!("{}", out);
            Ok(Rc::new(PyNone::new()))
        })),
    );

    // len(obj)
    env_mut.set(
        "len".to_string(),
        Rc::new(PyNativeFunction::new("len".to_string(), |args| {
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

    // str(obj)
    env_mut.set(
        "str".to_string(),
        Rc::new(PyNativeFunction::new("str".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("type".to_string(), |args| {
            if args.len() != 1 {
                return Err(format!(
                    "TypeError: type() takes exactly one argument ({} given)",
                    args.len()
                ));
            }
            let obj = &args[0];
            Ok(Rc::new(PyString::new(format!(
                "<class '{}'>",
                obj.get_type()
            ))))
        })),
    );

    // getattr(object, name[, default])
    env_mut.set(
        "getattr".to_string(),
        Rc::new(PyNativeFunction::new("getattr".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("setattr".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("hasattr".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("isinstance".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("id".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("hash".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("bool".to_string(), |args| {
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

    // int(object)
    env_mut.set(
        "int".to_string(),
        Rc::new(PyNativeFunction::new("int".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(0)));
            }
            if args.len() != 1 {
                return Err("TypeError: int() takes at most 1 argument".to_string());
            }
            let obj = &args[0];
            if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(i.as_i64().unwrap_or(0))));
            }
            if let Some(s) = obj.as_any().downcast_ref::<crate::objects::string::PyString>() {
                if let Ok(val) = s.value.parse::<i64>() {
                    return Ok(Rc::new(crate::objects::int::PyInt::from_i64(val)));
                } else {
                    return Err(format!("ValueError: invalid literal for int() with base 10: '{}'", s.value));
                }
            }
            if let Some(b) = obj.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
                return Ok(Rc::new(crate::objects::int::PyInt::from_i64(if b.value { 1 } else { 0 })));
            }
            Err(format!("TypeError: int() argument must be a string, a bytes-like object or a real number, not '{}'", obj.get_type()))
        })),
    );

    // list(iterable)
    env_mut.set(
        "list".to_string(),
        Rc::new(PyNativeFunction::new("list".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("tuple".to_string(), |args| {
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

    // dict()
    env_mut.set(
        "dict".to_string(),
        Rc::new(PyNativeFunction::new("dict".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::dict::PyDict::new()));
            }
            Err("TypeError: dict() kwargs/iterables not fully implemented yet".to_string())
        })),
    );
    // callable(object)
    env_mut.set(
        "callable".to_string(),
        Rc::new(PyNativeFunction::new("callable".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: callable() takes exactly one argument".to_string());
            }
            let obj = &args[0];
            let is_call = obj.get_type() == "function"
                || obj.get_type() == "native_function"
                || obj.get_type() == "class"
                || obj.get_type() == "bound_method";
            Ok(Rc::new(crate::objects::bool::PyBool::new(is_call)))
        })),
    );

    // object()
    env_mut.set(
        "object".to_string(),
        Rc::new(PyNativeFunction::new("object".to_string(), |args| {
            if !args.is_empty() {
                return Err("TypeError: object() takes no arguments".to_string());
            }
            Ok(Rc::new(crate::objects::none::PyNone::new()))
        })),
    );
    // complex(real, imag)
    env_mut.set(
        "complex".to_string(),
        Rc::new(PyNativeFunction::new("complex".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("classmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: classmethod() takes exactly one argument".to_string());
            }
            Ok(Rc::new(crate::objects::classmethod::PyClassMethod::new(Rc::clone(&args[0]))))
        })),
    );

    // staticmethod(function)
    env_mut.set(
        "staticmethod".to_string(),
        Rc::new(PyNativeFunction::new("staticmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: staticmethod() takes exactly one argument".to_string());
            }
            Ok(Rc::new(crate::objects::staticmethod::PyStaticMethod::new(Rc::clone(&args[0]))))
        })),
    );

    // property(fget, fset=None, fdel=None, doc=None)
    env_mut.set(
        "property".to_string(),
        Rc::new(PyNativeFunction::new("property".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("super".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("abs".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: abs() takes exactly one argument".to_string());
            }
            if let Some(i) = args[0]
                .as_any()
                .downcast_ref::<crate::objects::int::PyInt>()
            {
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(i.as_i64().unwrap_or(0).abs())))
            } else {
                Err("TypeError: bad operand type for abs()".to_string())
            }
        })),
    );

    // max(*args)
    env_mut.set(
        "max".to_string(),
        Rc::new(PyNativeFunction::new("max".to_string(), |args| {
            if args.is_empty() {
                return Err("TypeError: max expected 1 argument, got 0".to_string());
            }
            let mut maximum = &args[0];
            let mut max_val = maximum
                .as_any()
                .downcast_ref::<crate::objects::int::PyInt>()
                .map(|i| i.as_i64().unwrap_or(0))
                .unwrap_or(0);
            for arg in args.iter().skip(1) {
                if let Some(i) = arg.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    if i.as_i64().unwrap_or(0) > max_val {
                        max_val = i.as_i64().unwrap_or(0);
                        maximum = arg;
                    }
                }
            }
            Ok(maximum.clone())
        })),
    );

    // min(*args)
    env_mut.set(
        "min".to_string(),
        Rc::new(PyNativeFunction::new("min".to_string(), |args| {
            if args.is_empty() {
                return Err("TypeError: min expected 1 argument, got 0".to_string());
            }
            let mut minimum = &args[0];
            let mut min_val = minimum
                .as_any()
                .downcast_ref::<crate::objects::int::PyInt>()
                .map(|i| i.as_i64().unwrap_or(0))
                .unwrap_or(0);
            for arg in args.iter().skip(1) {
                if let Some(i) = arg.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    if i.as_i64().unwrap_or(0) < min_val {
                        min_val = i.as_i64().unwrap_or(0);
                        minimum = arg;
                    }
                }
            }
            Ok(minimum.clone())
        })),
    );

    // sum(iterable)
    env_mut.set(
        "sum".to_string(),
        Rc::new(PyNativeFunction::new("sum".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: sum expected 1 argument".to_string());
            }
            let iter = args[0].get_iter()?;
            let mut total = 0;
            while let Some(item) = iter.get_next()? {
                if let Some(i) = item.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    total += i.as_i64().unwrap_or(0);
                } else {
                    return Err("TypeError: unsupported operand type(s) for + in sum()".to_string());
                }
            }
            Ok(Rc::new(crate::objects::int::PyInt::from_i64(total)))
        })),
    );

    // repr(obj)
    env_mut.set(
        "repr".to_string(),
        Rc::new(PyNativeFunction::new("repr".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("bin".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("hex".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("iter".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: iter() takes exactly one argument".to_string());
            }
            args[0].get_iter()
        })),
    );

    // next(iterator[, default])
    env_mut.set(
        "next".to_string(),
        Rc::new(PyNativeFunction::new("next".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("all".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("any".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("globals".to_string(), move |args| {
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
        Rc::new(PyNativeFunction::new("locals".to_string(), move |args| {
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
    env_mut.set(
        "eval".to_string(),
        Rc::new(PyNativeFunction::new("eval".to_string(), |args| {
            Err("NotImplementedError: eval() is not fully wired to the AST yet".to_string())
        })),
    );

    // range(stop), range(start, stop), range(start, stop, step)
    env_mut.set(
        "range".to_string(),
        Rc::new(PyNativeFunction::new("range".to_string(), |args| {
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

    // exec(object, globals=None, locals=None)
    env_mut.set(
        "exec".to_string(),
        Rc::new(PyNativeFunction::new("exec".to_string(), |args| {
            Err("NotImplementedError: exec() is not fully wired to the AST yet".to_string())
        })),
    );

    // compile(source, filename, mode, flags=0, dont_inherit=False, optimize=-1)
    env_mut.set(
        "compile".to_string(),
        Rc::new(PyNativeFunction::new("compile".to_string(), |args| {
            Err("NotImplementedError: compile() is not fully wired to the AST yet".to_string())
        })),
    );

    // __import__(name, globals=None, locals=None, fromlist=(), level=0)
    env_mut.set(
        "__import__".to_string(),
        Rc::new(PyNativeFunction::new("__import__".to_string(), |args| {
            Err("ImportError: __import__() module loading not implemented yet".to_string())
        })),
    );

    // chr(i) -> str
    env_mut.set(
        "chr".to_string(),
        Rc::new(PyNativeFunction::new("chr".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("ord".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: ord() takes exactly one argument".to_string()); }
            let s = args[0].str();
            let c = s.chars().next().ok_or_else(|| "TypeError: ord() expected a character, not empty string".to_string())?;
            Ok(Rc::new(crate::objects::int::PyInt::from_i64(c as i64)))
        })),
    );

    // pow(x, y) -> number
    env_mut.set(
        "pow".to_string(),
        Rc::new(PyNativeFunction::new("pow".to_string(), |args| {
            if args.len() < 2 || args.len() > 3 { return Err("TypeError: pow() takes 2-3 arguments".to_string()); }
            // Simple two-argument pow using multiplication
            let base = &args[0];
            let exp = &args[1];
            match base.pow(Rc::clone(exp)) {
                Some(result) => Ok(result),
                None => Err(format!("TypeError: unsupported operand type(s) for pow(): '{}' and '{}'", base.get_type(), exp.get_type())),
            }
        })),
    );

    // round(x) -> int
    env_mut.set(
        "round".to_string(),
        Rc::new(PyNativeFunction::new("round".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: round() takes exactly one argument".to_string()); }
            if let Some(f) = args[0].as_any().downcast_ref::<crate::objects::float::PyFloat>() {
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(f.value.round() as i64)))
            } else if let Some(i) = args[0].as_any().downcast_ref::<crate::objects::int::PyInt>() {
                Ok(Rc::new(crate::objects::int::PyInt::from_i64(i.as_i64().unwrap_or(0))))
            } else {
                Err(format!("TypeError: type {} doesn't define __round__", args[0].get_type()))
            }
        })),
    );

    // sorted(iterable) -> list
    env_mut.set(
        "sorted".to_string(),
        Rc::new(PyNativeFunction::new("sorted".to_string(), |args| {
            if args.len() != 1 { return Err("TypeError: sorted() takes exactly one argument".to_string()); }
            let iter = args[0].get_iter()?;
            let mut items: Vec<Rc<dyn PyObject>> = Vec::new();
            while let Some(item) = iter.get_next()? {
                items.push(item);
            }
            // Bubble sort using lt()
            let n = items.len();
            for i in 0..n {
                for j in 0..n-1-i {
                    let do_swap = {
                        let a = Rc::clone(&items[j]);
                        let b = Rc::clone(&items[j+1]);
                        match a.lt(b) {
                            Some(result) => !result.is_truthy(),
                            None => false,
                        }
                    };
                    if do_swap {
                        items.swap(j, j+1);
                    }
                }
            }
            Ok(Rc::new(crate::objects::list::PyList::new(items)))
        })),
    );

    // reversed(seq) -> iterator (eager: returns a reversed list)
    env_mut.set(
        "reversed".to_string(),
        Rc::new(PyNativeFunction::new("reversed".to_string(), |args| {
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

    // enumerate(iterable, start=0) -> list of [index, value] pairs
    env_mut.set(
        "enumerate".to_string(),
        Rc::new(PyNativeFunction::new("enumerate".to_string(), |args| {
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
                result.push(Rc::new(crate::objects::list::PyList::new(pair)) as Rc<dyn PyObject>);
                idx += 1;
            }
            Ok(Rc::new(crate::objects::list::PyList::new(result)))
        })),
    );

    // map(func, iterable) -> list (eager; only native functions for now)
    env_mut.set(
        "map".to_string(),
        Rc::new(PyNativeFunction::new("map".to_string(), |args| {
            if args.len() != 2 { return Err("TypeError: map() takes exactly 2 arguments".to_string()); }
            if let Some(native) = args[0].as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                let iter = args[1].get_iter()?;
                let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
                while let Some(item) = iter.get_next()? {
                    result.push((native.func)(vec![item])?);
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
        Rc::new(PyNativeFunction::new("filter".to_string(), |args| {
            if args.len() != 2 { return Err("TypeError: filter() takes exactly 2 arguments".to_string()); }
            if let Some(native) = args[0].as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
                let iter = args[1].get_iter()?;
                let mut result: Vec<Rc<dyn PyObject>> = Vec::new();
                while let Some(item) = iter.get_next()? {
                    let should_keep = (native.func)(vec![Rc::clone(&item)])?;
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
        Rc::new(PyNativeFunction::new("zip".to_string(), |args| {
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
                result.push(Rc::new(crate::objects::list::PyList::new(group)) as Rc<dyn PyObject>);
            }
        })),
    );

    // bytes(source) -> bytes
    env_mut.set(
        "bytes".to_string(),
        Rc::new(PyNativeFunction::new("bytes".to_string(), |args| {
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

    // set(iterable) -> set
    env_mut.set(
        "set".to_string(),
        Rc::new(PyNativeFunction::new("set".to_string(), |args| {
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
        Rc::new(PyNativeFunction::new("frozenset".to_string(), |args| {
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
}
