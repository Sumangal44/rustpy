use crate::objects::PyObject;
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
            if let Some(s) = obj.as_any().downcast_ref::<PyString>() {
                // String length (character count)
                Ok(Rc::new(PyInt::new(s.value.chars().count() as i64)))
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
            Ok(Rc::new(crate::objects::int::PyInt::new(ptr)))
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
                Ok(h) => Ok(Rc::new(crate::objects::int::PyInt::new(h))),
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
                return Ok(Rc::new(crate::objects::int::PyInt::new(0)));
            }
            if args.len() != 1 {
                return Err("TypeError: int() takes at most 1 argument".to_string());
            }
            let obj = &args[0];
            if let Some(i) = obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                return Ok(Rc::new(crate::objects::int::PyInt::new(i.value)));
            }
            if let Some(s) = obj.as_any().downcast_ref::<crate::objects::string::PyString>() {
                if let Ok(val) = s.value.parse::<i64>() {
                    return Ok(Rc::new(crate::objects::int::PyInt::new(val)));
                } else {
                    return Err(format!("ValueError: invalid literal for int() with base 10: '{}'", s.value));
                }
            }
            if let Some(b) = obj.as_any().downcast_ref::<crate::objects::bool::PyBool>() {
                return Ok(Rc::new(crate::objects::int::PyInt::new(if b.value { 1 } else { 0 })));
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

    // dict()
    env_mut.set(
        "dict".to_string(),
        Rc::new(PyNativeFunction::new("dict".to_string(), |args| {
            if args.is_empty() {
                return Ok(Rc::new(crate::objects::dict::PyDict::new(
                    std::collections::HashMap::new(),
                )));
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
    // classmethod(function)
    env_mut.set(
        "classmethod".to_string(),
        Rc::new(PyNativeFunction::new("classmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: classmethod() takes exactly one argument".to_string());
            }
            Ok(args[0].clone()) // Stub for now
        })),
    );

    // staticmethod(function)
    env_mut.set(
        "staticmethod".to_string(),
        Rc::new(PyNativeFunction::new("staticmethod".to_string(), |args| {
            if args.len() != 1 {
                return Err("TypeError: staticmethod() takes exactly one argument".to_string());
            }
            Ok(args[0].clone()) // Stub for now
        })),
    );

    // property(fget)
    env_mut.set(
        "property".to_string(),
        Rc::new(PyNativeFunction::new("property".to_string(), |args| {
            if args.is_empty() || args.len() > 4 {
                return Err("TypeError: property() takes 1-4 arguments".to_string());
            }
            Ok(args[0].clone()) // Stub for now
        })),
    );

    // super()
    env_mut.set(
        "super".to_string(),
        Rc::new(PyNativeFunction::new("super".to_string(), |args| {
            // Stub for super(), returning None since inheritance isn't fully wired yet
            Ok(Rc::new(crate::objects::none::PyNone::new()))
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
                Ok(Rc::new(crate::objects::int::PyInt::new(i.value.abs())))
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
                .map(|i| i.value)
                .unwrap_or(0);
            for arg in args.iter().skip(1) {
                if let Some(i) = arg.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    if i.value > max_val {
                        max_val = i.value;
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
                .map(|i| i.value)
                .unwrap_or(0);
            for arg in args.iter().skip(1) {
                if let Some(i) = arg.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    if i.value < min_val {
                        min_val = i.value;
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
                    total += i.value;
                } else {
                    return Err("TypeError: unsupported operand type(s) for + in sum()".to_string());
                }
            }
            Ok(Rc::new(crate::objects::int::PyInt::new(total)))
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
                    i.value
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
                    i.value
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
            let mut dict = std::collections::HashMap::new();
            for (k, v) in env_clone.borrow().get_all_locals() {
                dict.insert(k, v);
            }
            Ok(Rc::new(crate::objects::dict::PyDict::new(dict)))
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
            let mut dict = std::collections::HashMap::new();
            for (k, v) in env_clone2.borrow().get_all_locals() {
                dict.insert(k, v);
            }
            Ok(Rc::new(crate::objects::dict::PyDict::new(dict)))
        })),
    );
    // eval(expression, globals=None, locals=None)
    env_mut.set(
        "eval".to_string(),
        Rc::new(PyNativeFunction::new("eval".to_string(), |args| {
            Err("NotImplementedError: eval() is not fully wired to the AST yet".to_string())
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
}
