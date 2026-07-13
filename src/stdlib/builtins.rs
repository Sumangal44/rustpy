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
}
