pub mod frame;

use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::function::PyFunction;
use crate::objects::instance::PyInstance;
use crate::objects::int::PyInt;
use crate::objects::string::PyString;
use crate::objects::PyObject;
use crate::runtime::Environment;
use frame::Frame;
use std::rc::Rc;

pub const RECURSION_LIMIT: usize = 100;

/// Extract the Python exception type name from an error string.
/// e.g. "AttributeError: 'Foo' object has no attribute 'z'" -> "AttributeError"
/// Falls back to "RuntimeError" for unrecognized patterns.
fn extract_exception_type(e: &str) -> String {
    // Known Python exception types
    const KNOWN: &[&str] = &[
        "AttributeError",
        "TypeError",
        "ValueError",
        "KeyError",
        "IndexError",
        "NameError",
        "ImportError",
        "ModuleNotFoundError",
        "RuntimeError",
        "StopAsyncIteration",
        "StopIteration",
        "ZeroDivisionError",
        "OverflowError",
        "RecursionError",
        "NotImplementedError",
        "AssertionError",
        "IOError",
        "OSError",
        "FileNotFoundError",
        "PermissionError",
        "GeneratorExit",
        "SystemExit",
        "KeyboardInterrupt",
        "ArithmeticError",
        "MemoryError",
        "LookupError",
        "Exception",
        "BaseException",
        "UnicodeError",
        "UnicodeDecodeError",
        "UnicodeEncodeError",
    ];
    for known in KNOWN {
        if e.starts_with(known) {
            return known.to_string();
        }
    }
    "RuntimeError".to_string()
}

pub struct VirtualMachine {
    pub last_exception: Option<Rc<dyn PyObject>>,
    #[allow(dead_code)]
    pub frames: Vec<Frame>,
    pub recursion_depth: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            last_exception: None,
            frames: Vec::new(),
            recursion_depth: 0,
        }
    }

    fn intercept_native_function(
        &mut self,
        func_obj: &Rc<dyn PyObject>,
        args: &[Rc<dyn PyObject>],
        kwargs: &std::collections::HashMap<String, Rc<dyn PyObject>>,
        frame: &mut Frame,
    ) -> Result<Option<Rc<dyn PyObject>>, String> {
        if let Some(nat) = func_obj
            .as_any()
            .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
        {
            match nat.name.as_str() {
                "super" if args.is_empty() => {
                    // 0-arg super(): look for self and __class__ in the current frame
                    let self_val = frame.env.borrow().get("self");
                    if let Some(self_obj) = self_val {
                        if let Some(inst) = self_obj
                            .as_any()
                            .downcast_ref::<crate::objects::instance::PyInstance>()
                        {
                            let super_proxy = crate::objects::class::PySuper::new(
                                Rc::clone(&inst.class),
                                Rc::new(inst.clone()),
                            );
                            return Ok(Some(Rc::new(super_proxy) as Rc<dyn PyObject>));
                        }
                        // Check if self is a class (for use in classmethods)
                        if let Some(cls) = self_obj
                            .as_any()
                            .downcast_ref::<crate::objects::class::PyClass>()
                        {
                            return Ok(Some(Rc::new(cls.clone()) as Rc<dyn PyObject>));
                        }
                    }
                    return Err("RuntimeError: super(): no arguments".to_string());
                }
                "locals" => {
                    let mut pairs = Vec::new();
                    for (k, v) in frame.env.borrow().get_all_locals().iter() {
                        pairs.push((
                            Rc::new(crate::objects::string::PyString::new(k.clone()))
                                as Rc<dyn PyObject>,
                            Rc::clone(v),
                        ));
                    }
                    return Ok(Some(Rc::new(crate::objects::dict::PyDict::from_pairs(
                        pairs,
                    ))));
                }
                "exec" | "eval" => {
                    if args.is_empty() {
                        return Err(format!("TypeError: {} expected 1 arg", nat.name));
                    }

                    let code = if let Some(c) = args[0]
                        .as_any()
                        .downcast_ref::<crate::compiler::code::CodeObject>()
                    {
                        c.clone()
                    } else if let Some(s) = args[0]
                        .as_any()
                        .downcast_ref::<crate::objects::string::PyString>()
                    {
                        let source = s.value.clone();
                        let lexer = crate::lexer::Lexer::new(&source);
                        let mut parser = match crate::parser::Parser::new(lexer) {
                            Ok(p) => p,
                            Err(e) => return Err(format!("{:?}", e)),
                        };
                        let compiler = crate::compiler::Compiler::new(format!("<{}>", nat.name));
                        if nat.name == "eval" {
                            let ast = parser.parse_expression(0).map_err(|e| format!("{:?}", e))?;
                            compiler
                                .compile_expression(&ast)
                                .map_err(|e| format!("{:?}", e))?
                        } else {
                            let ast = parser.parse_module().map_err(|e| format!("{:?}", e))?;
                            compiler.compile(&ast).map_err(|e| format!("{:?}", e))?
                        }
                    } else {
                        return Err(format!(
                            "TypeError: {} arg 1 must be string or code object",
                            nat.name
                        ));
                    };

                    let mut new_frame = crate::vm::frame::Frame::new(code, Rc::clone(&frame.env));
                    let res = self.run(&mut new_frame)?;
                    return Ok(Some(
                        res.unwrap_or_else(|| Rc::new(crate::objects::none::PyNone)),
                    ));
                }
                "map" => {
                    if args.len() != 2 {
                        return Err(format!("TypeError: {} expected 2 args", nat.name));
                    }
                    let func = &args[0];
                    let iter = args[1].get_iter()?;
                    let mut result = Vec::new();
                    while let Some(item) = iter.get_next()? {
                        let mapped = self.invoke(
                            Rc::clone(func),
                            vec![item],
                            std::collections::HashMap::new(),
                        )?;
                        result.push(mapped);
                    }
                    return Ok(Some(Rc::new(crate::objects::list::PyList::new(result))));
                }
                "filter" => {
                    if args.len() != 2 {
                        return Err(format!("TypeError: {} expected 2 args", nat.name));
                    }
                    let func = &args[0];
                    let iter = args[1].get_iter()?;
                    let mut result = Vec::new();
                    while let Some(item) = iter.get_next()? {
                        let keep = if func.as_any().is::<crate::objects::none::PyNone>() {
                            item.is_truthy()
                        } else {
                            self.invoke(
                                Rc::clone(func),
                                vec![Rc::clone(&item)],
                                std::collections::HashMap::new(),
                            )?
                            .is_truthy()
                        };
                        if keep {
                            result.push(item);
                        }
                    }
                    return Ok(Some(Rc::new(crate::objects::list::PyList::new(result))));
                }
                "max" => {
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

                    let mut max_val: Option<Rc<dyn PyObject>> = None;
                    for item in items {
                        if max_val.is_none() {
                            max_val = Some(item);
                            continue;
                        }

                        let cur_max = max_val.clone().unwrap();

                        let (cmp_val, cmp_max) = if let Some(ref kf) = key_fn {
                            let v = self.invoke(
                                Rc::clone(kf),
                                vec![Rc::clone(&item)],
                                std::collections::HashMap::new(),
                            )?;
                            let m = self.invoke(
                                Rc::clone(kf),
                                vec![Rc::clone(&cur_max)],
                                std::collections::HashMap::new(),
                            )?;
                            (v, m)
                        } else {
                            (Rc::clone(&item), Rc::clone(&cur_max))
                        };

                        let gt = cmp_val
                            .gt(cmp_max)
                            .ok_or_else(|| "TypeError: unorderable types".to_string())?;
                        if gt.is_truthy() {
                            max_val = Some(item);
                        }
                    }
                    return Ok(Some(max_val.unwrap()));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    pub fn run(&mut self, frame: &mut Frame) -> Result<Option<Rc<dyn PyObject>>, String> {
        self.recursion_depth += 1;
        if self.recursion_depth > RECURSION_LIMIT {
            self.recursion_depth -= 1;
            return Err("RecursionError: maximum recursion depth exceeded".to_string());
        }

        // Check if there's an exception already pending in the frame (e.g. from generator.throw())
        if let Some(exc) = frame.pending_exception.take() {
            let (exc_obj, exc_msg) = if let Some(py_exc) =
                exc.as_any()
                    .downcast_ref::<crate::objects::exception::PyException>()
            {
                let msg = format!(
                    "{}: {}",
                    py_exc.exc_type,
                    py_exc.message.as_deref().unwrap_or("")
                );
                (Rc::clone(&exc), msg)
            } else {
                let exc_msg = exc.str();
                let exc_type = extract_exception_type(&exc_msg);
                let exc_body = if let Some(pos) = exc_msg.find(": ") {
                    exc_msg[pos + 2..].to_string()
                } else {
                    exc_msg.clone()
                };
                let full_msg = format!("{}: {}", exc_type, exc_body);
                (
                    Rc::new(crate::objects::exception::PyException::new(
                        exc_type,
                        Some(exc_body),
                    )) as Rc<dyn PyObject>,
                    full_msg,
                )
            };
            self.last_exception = Some(exc_obj.clone());
            let mut handled = false;
            while let Some(block) = frame.block_stack.pop() {
                match block {
                    crate::vm::frame::Block::SetupExcept {
                        handler_ip,
                        stack_size,
                    } => {
                        frame.stack.truncate(stack_size);
                        frame.push(exc_obj.clone());
                        frame.ip = handler_ip;
                        handled = true;
                        break;
                    }
                    crate::vm::frame::Block::SetupFinally {
                        handler_ip,
                        stack_size,
                    } => {
                        frame.stack.truncate(stack_size);
                        frame.pending_exception = Some(exc_obj.clone());
                        frame.ip = handler_ip;
                        handled = true;
                        break;
                    }
                    _ => {}
                }
            }
            if !handled {
                self.recursion_depth -= 1;
                return Err(exc_msg);
            }
        }

        while frame.ip < frame.code.instructions.len() {
            let opcode = frame.code.instructions[frame.ip].clone();
            frame.ip += 1;

            match self.execute_opcode(&opcode, frame) {
                Ok(Some(ret)) => {
                    self.recursion_depth -= 1;
                    return Ok(Some(ret));
                }
                Ok(None) => {} // continue
                Err(e) => {
                    frame.pending_exception = None;

                    let exc_obj = self.last_exception.take().unwrap_or_else(|| {
                        // Extract exception type from error strings like "AttributeError: ..."
                        // Known exception types to check for
                        let exc_type = extract_exception_type(&e);
                        let message = if let Some(colon) = e.find(": ") {
                            // message is everything after "TypeName: "
                            e[colon + 2..].to_string()
                        } else {
                            e.clone()
                        };
                        Rc::new(crate::objects::exception::PyException::new(
                            exc_type,
                            Some(message),
                        ))
                    });
                    self.last_exception = Some(exc_obj.clone());

                    let mut handled = false;
                    while let Some(block) = frame.block_stack.pop() {
                        match block {
                            crate::vm::frame::Block::SetupExcept {
                                handler_ip,
                                stack_size,
                            } => {
                                frame.stack.truncate(stack_size);
                                frame.push(exc_obj.clone());
                                frame.ip = handler_ip;
                                handled = true;
                                break;
                            }
                            crate::vm::frame::Block::SetupFinally {
                                handler_ip,
                                stack_size,
                            } => {
                                frame.stack.truncate(stack_size);
                                frame.pending_exception = Some(exc_obj.clone());
                                frame.ip = handler_ip;
                                handled = true;
                                break;
                            }
                            crate::vm::frame::Block::SetupWith {
                                handler_ip,
                                stack_size,
                                exit_func,
                            } => {
                                frame.stack.truncate(stack_size);
                                let none =
                                    Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>;
                                let exc_type = Rc::new(crate::objects::string::PyString::new(
                                    "Exception".to_string(),
                                ))
                                    as Rc<dyn PyObject>;
                                let exc_val = exc_obj.clone();

                                let exit_res = self.invoke(
                                    exit_func,
                                    vec![exc_type, exc_val, none],
                                    std::collections::HashMap::new(),
                                );
                                if let Ok(res) = exit_res {
                                    if res.is_truthy() {
                                        self.last_exception = None;
                                        frame.ip = handler_ip;
                                        handled = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !handled {
                        self.recursion_depth -= 1;
                        return Err(e);
                    }
                }
            }
        }

        self.recursion_depth -= 1;
        Ok(None)
    }

    fn execute_opcode(
        &mut self,
        opcode: &Opcode,
        frame: &mut Frame,
    ) -> Result<Option<Rc<dyn PyObject>>, String> {
        match opcode {
            Opcode::PopTop => {
                frame.pop()?;
            }
            Opcode::DupTop => {
                let val = frame.pop()?;
                frame.push(Rc::clone(&val));
                frame.push(val);
            }
            Opcode::RotTwo => {
                let first = frame.pop()?;
                let second = frame.pop()?;
                frame.push(first);
                frame.push(second);
            }
            Opcode::RotThree => {
                let first = frame.pop()?;
                let second = frame.pop()?;
                let third = frame.pop()?;
                frame.push(first);
                frame.push(third);
                frame.push(second);
            }
            Opcode::LoadConst(idx) => {
                let obj = Rc::clone(&frame.code.constants[*idx]);
                frame.push(obj);
            }
            Opcode::LoadName(idx) => {
                let name = &frame.code.names[*idx];
                let obj_opt = frame.env.borrow().get(name);
                if let Some(obj) = obj_opt {
                    frame.push(obj);
                } else {
                    return Err(format!("NameError: name '{}' is not defined", name));
                }
            }
            Opcode::StoreName(idx) => {
                let name = frame.code.names[*idx].clone();
                let obj = frame.pop()?;
                if frame.code.nonlocal_names.contains(&name) {
                    frame.env.borrow_mut().set_nonlocal(name, obj);
                } else {
                    frame.env.borrow_mut().set(name, obj);
                }
            }
            Opcode::LoadGlobal(idx) => {
                let name = &frame.code.names[*idx];
                let obj_opt = frame.env.borrow().get_root(name);
                if let Some(obj) = obj_opt {
                    frame.push(obj);
                } else {
                    return Err(format!("NameError: global name '{}' is not defined", name));
                }
            }
            Opcode::StoreGlobal(idx) => {
                let name = frame.code.names[*idx].clone();
                let obj = frame.pop()?;
                frame.env.borrow_mut().set_root(name, obj);
            }
            Opcode::DeleteName(idx) => {
                let name = &frame.code.names[*idx];
                if frame.env.borrow_mut().remove(name) {
                    // success
                } else {
                    return Err(format!("NameError: name '{}' is not defined", name));
                }
            }
            Opcode::DeleteAttr(attr_name) => {
                let obj = frame.pop()?;
                obj.del_attr(attr_name)?;
            }
            Opcode::DeleteSubscript => {
                let key = frame.pop()?;
                let collection = frame.pop()?;
                collection.del_item(key)?;
            }
            Opcode::StoreSubscript => {
                let key = frame.pop()?;
                let collection = frame.pop()?;
                let value = frame.pop()?;
                if let Some(list) = collection
                    .as_any()
                    .downcast_ref::<crate::objects::list::PyList>()
                {
                    if let Some(int_idx) = key.as_any().downcast_ref::<crate::objects::int::PyInt>()
                    {
                        let mut elements = list.elements.borrow_mut();
                        let mut idx_val = int_idx.as_i64().unwrap_or(0);
                        if idx_val < 0 {
                            idx_val += elements.len() as i64;
                        }
                        if idx_val >= 0 && (idx_val as usize) < elements.len() {
                            elements[idx_val as usize] = value;
                        } else {
                            return Err(
                                "IndexError: list assignment index out of range".to_string()
                            );
                        }
                    } else {
                        return Err("TypeError: list indices must be integers, not ...".to_string());
                    }
                } else if let Some(d) = collection
                    .as_any()
                    .downcast_ref::<crate::objects::dict::PyDict>()
                {
                    d.set_item(key, value)?;
                } else {
                    return Err(
                        "TypeError: '{}' object does not support item assignment".to_string()
                    );
                }
            }
            Opcode::BinaryAdd => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.add(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for +: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinarySubtract => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.sub(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for -: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryMultiply => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.mul(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for *: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::JumpAbsolute(target) => {
                frame.ip = *target;
            }
            Opcode::PopJumpIfFalse(target) => {
                let cond = frame.pop()?;
                if !cond.is_truthy() {
                    frame.ip = *target;
                }
            }
            Opcode::MakeFunction => {
                let code_obj = frame.pop()?;
                let defaults_obj = frame.pop()?;
                let kwdefaults_obj = frame.pop()?;
                let mut defaults = Vec::new();
                if let Some(tup) = defaults_obj
                    .as_any()
                    .downcast_ref::<crate::objects::tuple::PyTuple>()
                {
                    for item in &tup.elements {
                        defaults.push(Rc::clone(item));
                    }
                } else {
                    return Err("Expected tuple for defaults to MakeFunction".to_string());
                }
                let mut kwonly_defaults = Vec::new();
                if let Some(tup) = kwdefaults_obj
                    .as_any()
                    .downcast_ref::<crate::objects::tuple::PyTuple>()
                {
                    for item in &tup.elements {
                        kwonly_defaults.push(Rc::clone(item));
                    }
                } else {
                    return Err("Expected tuple for kwonly defaults to MakeFunction".to_string());
                }
                if let Some(code) = code_obj.as_any().downcast_ref::<CodeObject>() {
                    let name = code.name.clone();
                    let params = code
                        .names
                        .iter()
                        .take(code.arg_count)
                        .cloned()
                        .collect::<Vec<_>>();
                    let kwonly_params = code.kwonly_params.clone();

                    let func = Rc::new(PyFunction::new(
                        name,
                        params,
                        code.clone(),
                        Rc::clone(&frame.env),
                        defaults,
                        code.posonly_count,
                        kwonly_params,
                        kwonly_defaults,
                    ));
                    frame.push(func);
                } else {
                    return Err("Expected code object to MakeFunction".to_string());
                }
            }
            Opcode::CallFunction(argc) => {
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(frame.pop()?);
                }
                args.reverse(); // Pop gets them in reverse order

                let func_obj = frame.pop()?;

                if let Some(res) = self.intercept_native_function(
                    &func_obj,
                    &args,
                    &std::collections::HashMap::new(),
                    frame,
                )? {
                    frame.push(res);
                    return Ok(None);
                }

                if frame.ip < frame.code.instructions.len()
                    && frame.code.instructions[frame.ip] == Opcode::ReturnValue
                {
                    if let Some(func) = func_obj.as_any().downcast_ref::<PyFunction>() {
                        if func.code.name == frame.code.name {
                            let new_env = Environment::new_enclosed(Rc::clone(&func.env));
                            let mut vararg_extra = Vec::new();
                            let param_count = func.params.len();

                            for (i, arg) in args.iter().enumerate() {
                                if i < param_count {
                                    new_env
                                        .borrow_mut()
                                        .set(func.params[i].clone(), Rc::clone(arg));
                                } else if func.code.vararg.is_some() {
                                    vararg_extra.push(Rc::clone(arg));
                                } else {
                                    return Err(format!(
                                        "TypeError: {}() takes {} positional arguments but {} were given",
                                        func.code.name,
                                        param_count,
                                        args.len()
                                    ));
                                }
                            }

                            if let Some(vararg) = &func.code.vararg {
                                let tup =
                                    Rc::new(crate::objects::tuple::PyTuple::new(vararg_extra));
                                new_env.borrow_mut().set(vararg.clone(), tup);
                            }

                            let default_count = func.defaults.len();
                            let pos_count = args.len().min(param_count);
                            let mandatory_count = param_count.saturating_sub(default_count);
                            if pos_count < param_count {
                                for i in pos_count..param_count {
                                    if i >= mandatory_count {
                                        let default_idx = i - mandatory_count;
                                        if default_idx < default_count {
                                            new_env.borrow_mut().set(
                                                func.params[i].clone(),
                                                Rc::clone(&func.defaults[default_idx]),
                                            );
                                        }
                                    }
                                }
                            }

                            let kw_default_offset = func
                                .kwonly_params
                                .len()
                                .saturating_sub(func.kwonly_defaults.len());
                            for (i, param) in func.kwonly_params.iter().enumerate() {
                                if new_env.borrow().get(param).is_none() {
                                    if i >= kw_default_offset {
                                        let kw_idx = i - kw_default_offset;
                                        if kw_idx < func.kwonly_defaults.len() {
                                            new_env.borrow_mut().set(
                                                param.clone(),
                                                Rc::clone(&func.kwonly_defaults[kw_idx]),
                                            );
                                            continue;
                                        }
                                    }
                                    return Err(format!(
                                        "TypeError: missing required keyword-only argument '{}'",
                                        param
                                    ));
                                }
                            }

                            if let Some(kwarg) = &func.code.kwarg {
                                let dict = Rc::new(crate::objects::dict::PyDict::new());
                                new_env.borrow_mut().set(kwarg.clone(), dict);
                            }

                            for i in 0..param_count {
                                if new_env.borrow().get(&func.params[i]).is_none() {
                                    return Err(format!(
                                        "TypeError: missing required positional argument '{}'",
                                        func.params[i]
                                    ));
                                }
                            }

                            frame.env = new_env;
                            frame.ip = 0;
                            frame.stack.clear();
                            return Ok(None);
                        }
                    }
                }

                let result = self.invoke(func_obj, args, std::collections::HashMap::new())?;
                frame.push(result);
            }
            Opcode::CallFunctionKw(argc) => {
                let _kwarg_names_obj = frame.pop()?;
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(frame.pop()?);
                }
                args.reverse();

                let func_obj = frame.pop()?;

                let kwargs = std::collections::HashMap::new();
                if let Some(res) =
                    self.intercept_native_function(&func_obj, &args, &kwargs, frame)?
                {
                    frame.push(res);
                    return Ok(None);
                }
                let result = self.invoke(func_obj, args, kwargs)?;
                frame.push(result);
            }
            Opcode::CallFunctionEx(flags) => {
                let mut kwargs = std::collections::HashMap::new();
                if *flags & 1 != 0 {
                    let kwargs_dict_obj = frame.pop()?;
                    if let Some(dict) = kwargs_dict_obj
                        .as_any()
                        .downcast_ref::<crate::objects::dict::PyDict>()
                    {
                        for bucket in dict.entries.borrow().values() {
                            for (k, v) in bucket {
                                let key_str = if let Some(s) =
                                    k.as_any()
                                        .downcast_ref::<crate::objects::string::PyString>()
                                {
                                    s.value.clone()
                                } else {
                                    k.repr()
                                };
                                kwargs.insert(key_str, Rc::clone(v));
                            }
                        }
                    } else {
                        return Err("TypeError: kwargs must be a dict".to_string());
                    }
                }
                let args_iter_obj = frame.pop()?;
                let mut args = Vec::new();
                if let Some(list) = args_iter_obj
                    .as_any()
                    .downcast_ref::<crate::objects::list::PyList>()
                {
                    for el in list.elements.borrow().iter() {
                        args.push(Rc::clone(el));
                    }
                } else {
                    return Err("TypeError: args must be an iterable".to_string());
                }

                let func_obj = frame.pop()?;

                if let Some(res) =
                    self.intercept_native_function(&func_obj, &args, &kwargs, frame)?
                {
                    frame.push(res);
                    return Ok(None);
                }

                let result = self.invoke(func_obj, args, kwargs)?;
                frame.push(result);
            }
            Opcode::BuildList(count) => {
                let mut elements = Vec::new();
                for _ in 0..*count {
                    elements.push(frame.pop()?);
                }
                elements.reverse(); // Pop reverses the order
                let list = Rc::new(crate::objects::list::PyList::new(elements));
                frame.push(list);
            }
            Opcode::BuildTuple(count) => {
                let mut elements = Vec::new();
                for _ in 0..*count {
                    elements.push(frame.pop()?);
                }
                elements.reverse();
                let t = Rc::new(crate::objects::tuple::PyTuple::new(elements));
                frame.push(t);
            }
            Opcode::UnpackSequence(count) => {
                let seq = frame.pop()?;
                if let Some(t) = seq
                    .as_any()
                    .downcast_ref::<crate::objects::tuple::PyTuple>()
                {
                    if t.elements.len() != *count {
                        return Err(format!(
                            "ValueError: too many values to unpack (expected {})",
                            *count
                        ));
                    }
                    for i in (0..*count).rev() {
                        frame.push(Rc::clone(&t.elements[i]));
                    }
                } else if let Some(l) = seq.as_any().downcast_ref::<crate::objects::list::PyList>()
                {
                    let elements = l.elements.borrow();
                    if elements.len() != *count {
                        return Err(format!(
                            "ValueError: too many values to unpack (expected {})",
                            *count
                        ));
                    }
                    for i in (0..*count).rev() {
                        frame.push(Rc::clone(&elements[i]));
                    }
                } else {
                    let iter = seq.get_iter()?;
                    let mut items = Vec::new();
                    while let Some(item) = iter.get_next()? {
                        items.push(item);
                    }
                    if items.len() != *count {
                        return Err(format!(
                            "ValueError: too many values to unpack (expected {})",
                            *count
                        ));
                    }
                    for item in items.into_iter().rev() {
                        frame.push(item);
                    }
                }
            }
            Opcode::UnpackEx(before, after) => {
                let before = *before;
                let after = *after;
                let total = before + after;
                let seq = frame.pop()?;
                let iter = seq.get_iter()?;
                let mut items = Vec::new();
                while let Some(item) = iter.get_next()? {
                    items.push(item);
                }
                if items.len() < total {
                    return Err(format!(
                        "ValueError: not enough values to unpack (expected at least {}, got {})",
                        total,
                        items.len()
                    ));
                }
                let star_count = items.len() - total;
                // Push items_after in reverse
                for item in items.iter().skip(before + star_count).rev() {
                    frame.push(Rc::clone(item));
                }
                // Push star list
                let star_items: Vec<Rc<dyn PyObject>> = items[before..before + star_count].to_vec();
                let star_list = Rc::new(crate::objects::list::PyList::new(star_items));
                frame.push(star_list);
                // Push items_before in reverse
                for item in items[..before].iter().rev() {
                    frame.push(Rc::clone(item));
                }
            }
            Opcode::BuildMap(count) => {
                let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
                for _ in 0..*count {
                    let value = frame.pop()?;
                    let key_obj = frame.pop()?;
                    key_obj.hash().map_err(|_| {
                        format!("TypeError: unhashable type: '{}'", key_obj.get_type())
                    })?;
                    pairs.push((key_obj, value));
                }
                pairs.reverse();
                let dict = Rc::new(crate::objects::dict::PyDict::from_pairs(pairs));
                frame.push(dict);
            }
            Opcode::ListExtend => {
                let iterable = frame.pop()?;
                let list_obj = frame.pop()?;

                if let Some(list) = list_obj
                    .as_any()
                    .downcast_ref::<crate::objects::list::PyList>()
                {
                    if let Some(other_list) = iterable
                        .as_any()
                        .downcast_ref::<crate::objects::list::PyList>()
                    {
                        for el in other_list.elements.borrow().iter() {
                            list.elements.borrow_mut().push(Rc::clone(el));
                        }
                    } else {
                        return Err("TypeError: object is not iterable".to_string());
                    }
                    frame.push(list_obj); // push list back
                } else {
                    return Err("TypeError: ListExtend expected a list".to_string());
                }
            }
            Opcode::DictMerge => {
                let dict2_obj = frame.pop()?;
                let dict1_obj = frame.pop()?;

                if let Some(dict1) = dict1_obj
                    .as_any()
                    .downcast_ref::<crate::objects::dict::PyDict>()
                {
                    if let Some(dict2) = dict2_obj
                        .as_any()
                        .downcast_ref::<crate::objects::dict::PyDict>()
                    {
                        let ord2 = dict2.ordered_keys.borrow();
                        let entries2 = dict2.entries.borrow();
                        for k in ord2.iter() {
                            if let Ok(h) = crate::objects::dict::get_hash(k) {
                                if let Some(bucket) = entries2.get(&h) {
                                    if let Some(idx) =
                                        crate::objects::dict::find_in_bucket(bucket, k)
                                    {
                                        let _ =
                                            dict1.set_item(Rc::clone(k), Rc::clone(&bucket[idx].1));
                                    }
                                }
                            }
                        }
                        frame.push(dict1_obj); // push dict1 back
                    } else {
                        return Err("TypeError: dict merge expected a dict".to_string());
                    }
                } else {
                    return Err("TypeError: dict merge expected a dict".to_string());
                }
            }
            Opcode::ListAppend => {
                let item = frame.pop()?;
                let list_obj = frame.pop()?;
                if let Some(list) = list_obj
                    .as_any()
                    .downcast_ref::<crate::objects::list::PyList>()
                {
                    list.elements.borrow_mut().push(item);
                    frame.push(list_obj);
                } else {
                    return Err("TypeError: ListAppend expected a list".to_string());
                }
            }
            Opcode::BuildSet(count) => {
                let mut elements = Vec::new();
                for _ in 0..*count {
                    elements.push(frame.pop()?);
                }
                elements.reverse();
                let set = Rc::new(crate::objects::set::PySet::new(elements));
                frame.push(set);
            }
            Opcode::SetAdd => {
                let item = frame.pop()?;
                let set_obj = frame.pop()?;
                if let Some(set) = set_obj
                    .as_any()
                    .downcast_ref::<crate::objects::set::PySet>()
                {
                    let duplicate = {
                        let elements = set.elements.borrow();
                        crate::objects::set::PySet::has_element(&*elements, &item)
                    };
                    if !duplicate {
                        set.elements.borrow_mut().push(item);
                    }
                    frame.push(set_obj);
                } else {
                    return Err("TypeError: SetAdd expected a set".to_string());
                }
            }
            Opcode::MapAdd => {
                let value = frame.pop()?;
                let key = frame.pop()?;
                let dict_obj = frame.pop()?;
                if let Some(dict) = dict_obj
                    .as_any()
                    .downcast_ref::<crate::objects::dict::PyDict>()
                {
                    dict.set_item(key, value)?;
                    frame.push(dict_obj);
                } else {
                    return Err("TypeError: MapAdd expected a dict".to_string());
                }
            }
            Opcode::BuildSlice => {
                let step_obj = frame.pop()?;
                let stop_obj = frame.pop()?;
                let start_obj = frame.pop()?;
                let start = if start_obj
                    .as_any()
                    .downcast_ref::<crate::objects::none::PyNone>()
                    .is_some()
                {
                    None
                } else if let Some(i) = start_obj
                    .as_any()
                    .downcast_ref::<crate::objects::int::PyInt>()
                {
                    Some(i.as_i64().unwrap_or(0))
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                let stop = if stop_obj
                    .as_any()
                    .downcast_ref::<crate::objects::none::PyNone>()
                    .is_some()
                {
                    None
                } else if let Some(i) = stop_obj
                    .as_any()
                    .downcast_ref::<crate::objects::int::PyInt>()
                {
                    Some(i.as_i64().unwrap_or(0))
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                let step = if step_obj
                    .as_any()
                    .downcast_ref::<crate::objects::none::PyNone>()
                    .is_some()
                {
                    None
                } else if let Some(i) = step_obj
                    .as_any()
                    .downcast_ref::<crate::objects::int::PyInt>()
                {
                    Some(i.as_i64().unwrap_or(0))
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                if let Some(s) = step {
                    if s == 0 {
                        return Err("ValueError: slice step cannot be zero".to_string());
                    }
                }
                let slice = Rc::new(crate::objects::slice::PySlice::new(start, stop, step));
                frame.push(slice);
            }
            Opcode::BinarySubscript => {
                let idx = frame.pop()?;
                let collection = frame.pop()?;
                let result = collection.get_item(idx)?;
                frame.push(result);
            }
            Opcode::GetIter => {
                let obj = frame.pop()?;
                let iterator = obj.get_iter()?;
                frame.push(iterator);
            }
            Opcode::ForIter(target) => {
                let iterator = frame.stack.last().unwrap().clone();
                if let Some(item) = iterator.get_next()? {
                    frame.push(item);
                } else {
                    frame.pop()?;
                    frame.ip = *target;
                }
            }
            Opcode::BuildClass {
                bases: num_bases,
                keywords: num_keywords,
            } => {
                // Pop keywords (key, value pairs)
                let mut keywords = std::collections::HashMap::new();
                for _ in 0..*num_keywords {
                    let value = frame.pop()?;
                    let key_obj = frame.pop()?;
                    if let Some(s) = key_obj
                        .as_any()
                        .downcast_ref::<crate::objects::string::PyString>()
                    {
                        keywords.insert(s.value.clone(), value);
                    }
                }

                let mut bases = Vec::new();
                for _ in 0..*num_bases {
                    bases.push(frame.pop()?);
                }
                bases.reverse();

                let code_obj = frame.pop()?;
                let name_obj = frame.pop()?;

                let name = name_obj
                    .as_any()
                    .downcast_ref::<crate::objects::string::PyString>()
                    .ok_or_else(|| "CompilerError: class name is not a string".to_string())?
                    .value
                    .clone();

                let code = code_obj
                    .as_any()
                    .downcast_ref::<crate::compiler::code::CodeObject>()
                    .ok_or_else(|| "CompilerError: BuildClass expected code object".to_string())?;

                // Execute the code block in a new environment to collect methods
                let class_env = Environment::new_enclosed(Rc::clone(&frame.env));
                let mut class_frame = Frame::new(code.clone(), class_env.clone());

                self.run(&mut class_frame)?;

                // Extract methods from class_env
                let attributes = class_env.borrow().get_all_locals();

                let class = crate::objects::class::PyClass::new(name, attributes, bases)?;

                // Handle metaclass keyword - store on class attributes
                if let Some(metaclass) = keywords.remove("metaclass") {
                    class
                        .attributes
                        .borrow_mut()
                        .insert("__metaclass__".to_string(), metaclass);
                }

                let class_obj = Rc::new(class);
                frame.push(class_obj);
            }
            Opcode::LoadAttr(attr_name) => {
                let obj = frame.pop()?;
                let attr_val = obj.get_attr(attr_name)?;
                frame.push(attr_val);
            }
            Opcode::StoreAttr(attr_name) => {
                let obj = frame.pop()?;
                let val = frame.pop()?;
                obj.set_attr(attr_name, val)?;
            }
            Opcode::ReturnValue => {
                let val = if frame.stack.is_empty() {
                    Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>
                } else {
                    frame.pop()?
                };
                frame.return_value = Some(val.clone());
                if frame.code.is_async {
                    return Ok(Some(
                        Rc::new(crate::objects::none::PyNone::new()) as Rc<dyn PyObject>
                    ));
                }
                return Ok(Some(val));
            }
            Opcode::YieldValue => {
                let ret = frame.pop()?;
                // When generator resumes, it evaluates to None (or sent value)
                frame.push(Rc::new(crate::objects::none::PyNone));
                return Ok(Some(ret)); // Returns from run loop, but frame.ip is advanced so it can be resumed
            }
            Opcode::SetupExcept(target) => {
                let stack_size = frame.stack.len();
                frame
                    .block_stack
                    .push(crate::vm::frame::Block::SetupExcept {
                        handler_ip: *target,
                        stack_size,
                    });
            }
            Opcode::PopExcept => {
                frame
                    .block_stack
                    .pop()
                    .ok_or_else(|| "CompilerError: PopExcept on empty block stack".to_string())?;
            }
            Opcode::BinaryTrueDivide => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_zero =
                    if let Some(i) = right.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        i.as_i64() == Some(0)
                    } else if let Some(f) = right
                        .as_any()
                        .downcast_ref::<crate::objects::float::PyFloat>()
                    {
                        f.value == 0.0
                    } else {
                        false
                    };
                if is_zero {
                    let exc = crate::objects::exception::PyException::new(
                        "ZeroDivisionError".to_string(),
                        Some("division by zero".to_string()),
                    );
                    self.last_exception = Some(Rc::new(exc));
                    return Err("ZeroDivisionError: division by zero".to_string());
                }
                if let Some(result) = left.truediv(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for /: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryFloorDivide => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_zero =
                    if let Some(i) = right.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        i.as_i64() == Some(0)
                    } else if let Some(f) = right
                        .as_any()
                        .downcast_ref::<crate::objects::float::PyFloat>()
                    {
                        f.value == 0.0
                    } else {
                        false
                    };
                if is_zero {
                    let exc = crate::objects::exception::PyException::new(
                        "ZeroDivisionError".to_string(),
                        Some("integer division or modulo by zero".to_string()),
                    );
                    self.last_exception = Some(Rc::new(exc));
                    return Err("ZeroDivisionError: integer division or modulo by zero".to_string());
                }
                if let Some(result) = left.floordiv(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for //: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryModulo => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_zero =
                    if let Some(i) = right.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        i.as_i64() == Some(0)
                    } else if let Some(f) = right
                        .as_any()
                        .downcast_ref::<crate::objects::float::PyFloat>()
                    {
                        f.value == 0.0
                    } else {
                        false
                    };
                if is_zero {
                    let exc = crate::objects::exception::PyException::new(
                        "ZeroDivisionError".to_string(),
                        Some("integer division or modulo by zero".to_string()),
                    );
                    self.last_exception = Some(Rc::new(exc));
                    return Err("ZeroDivisionError: integer division or modulo by zero".to_string());
                }
                if let Some(result) = left.modulo(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for %: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryPower => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.pow(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for **: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryMatMul => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                // Try __matmul__ on user-defined classes
                let matmul_result = {
                    let is_instance = left.as_any().downcast_ref::<PyInstance>().is_some();
                    if is_instance {
                        if let Some(inst) = left.as_any().downcast_ref::<PyInstance>() {
                            if let Ok(method) = inst.get_attr("__matmul__") {
                                self.invoke(
                                    method,
                                    vec![Rc::clone(&right)],
                                    std::collections::HashMap::new(),
                                )
                                .ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        left.matmul(Rc::clone(&right))
                    }
                };
                if let Some(result) = matmul_result {
                    frame.push(result);
                } else {
                    // Try __rmatmul__ on the right operand
                    let rmatmul_result =
                        if let Some(inst) = right.as_any().downcast_ref::<PyInstance>() {
                            if let Ok(method) = inst.get_attr("__rmatmul__") {
                                self.invoke(
                                    method,
                                    vec![Rc::clone(&left)],
                                    std::collections::HashMap::new(),
                                )
                                .ok()
                            } else {
                                None
                            }
                        } else {
                            right.rmatmul(Rc::clone(&left))
                        };
                    if let Some(result) = rmatmul_result {
                        frame.push(result);
                    } else {
                        return Err(format!(
                            "TypeError: unsupported operand type(s) for @: '{}' and '{}'",
                            left.get_type(),
                            right.get_type()
                        ));
                    }
                }
            }
            Opcode::BinaryBitAnd => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitand(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for &: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryBitOr => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitor(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for |: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryBitXor => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitxor(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for ^: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryLShift => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.lshift(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for <<: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::BinaryRShift => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.rshift(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: unsupported operand type(s) for >>: '{}' and '{}'",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::UnaryNegative => {
                let obj = frame.pop()?;
                if let Some(result) = obj.neg() {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: bad operand type for unary -: '{}'",
                        obj.get_type()
                    ));
                }
            }
            Opcode::UnaryPositive => {
                let obj = frame.pop()?;
                if let Some(result) = obj.pos() {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: bad operand type for unary +: '{}'",
                        obj.get_type()
                    ));
                }
            }
            Opcode::UnaryNot => {
                let obj = frame.pop()?;
                frame.push(Rc::new(crate::objects::bool::PyBool::new(!obj.is_truthy())));
            }
            Opcode::UnaryInvert => {
                let val = frame.pop()?;
                if let Some(result) = val.invert() {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: bad operand type for unary ~: '{}'",
                        val.get_type()
                    ));
                }
            }
            Opcode::CompareEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_type_nf = if left.get_type() == "type"
                    && right.get_type() == "builtin_function_or_method"
                {
                    if let Some(t) = left
                        .as_any()
                        .downcast_ref::<crate::objects::typeobj::PyType>()
                    {
                        if let Some(nf) = right
                            .as_any()
                            .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                        ) {
                            t.name == nf.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if left.get_type() == "builtin_function_or_method"
                    && right.get_type() == "type"
                {
                    if let Some(nf) = left
                        .as_any()
                        .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
                    {
                        if let Some(t) = right
                            .as_any()
                            .downcast_ref::<crate::objects::typeobj::PyType>()
                        {
                            nf.name == t.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_type_nf {
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(true)));
                } else if let Some(result) = left.eq(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Opcode::CompareNotEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_type_nf = if left.get_type() == "type"
                    && right.get_type() == "builtin_function_or_method"
                {
                    if let Some(t) = left
                        .as_any()
                        .downcast_ref::<crate::objects::typeobj::PyType>()
                    {
                        if let Some(nf) = right
                            .as_any()
                            .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                        ) {
                            t.name == nf.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if left.get_type() == "builtin_function_or_method"
                    && right.get_type() == "type"
                {
                    if let Some(nf) = left
                        .as_any()
                        .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
                    {
                        if let Some(t) = right
                            .as_any()
                            .downcast_ref::<crate::objects::typeobj::PyType>()
                        {
                            nf.name == t.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_type_nf {
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(false)));
                } else if let Some(result) = left.ne(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(true)));
                }
            }
            Opcode::CompareLt => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.lt(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with <",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::CompareLtEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.le(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with <=",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::CompareGt => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.gt(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with >",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::CompareGtEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.ge(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with >=",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::CompareIn => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                match right.contains(Rc::clone(&left)) {
                    Ok(result) => frame.push(Rc::new(crate::objects::bool::PyBool::new(result))),
                    Err(e) => return Err(e),
                }
            }
            Opcode::CompareNotIn => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                match right.contains(Rc::clone(&left)) {
                    Ok(result) => frame.push(Rc::new(crate::objects::bool::PyBool::new(!result))),
                    Err(e) => return Err(e),
                }
            }
            Opcode::CompareIs => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_eq = if left.get_type() == "NoneType" && right.get_type() == "NoneType" {
                    true
                } else if left.get_type() == "bool" && right.get_type() == "bool" {
                    left.is_truthy() == right.is_truthy()
                } else if left.get_type() == "type"
                    && right.get_type() == "builtin_function_or_method"
                {
                    if let Some(t) = left
                        .as_any()
                        .downcast_ref::<crate::objects::typeobj::PyType>()
                    {
                        if let Some(nf) = right
                            .as_any()
                            .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                        ) {
                            t.name == nf.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if left.get_type() == "builtin_function_or_method"
                    && right.get_type() == "type"
                {
                    if let Some(nf) = left
                        .as_any()
                        .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
                    {
                        if let Some(t) = right
                            .as_any()
                            .downcast_ref::<crate::objects::typeobj::PyType>()
                        {
                            nf.name == t.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    Rc::ptr_eq(&left, &right)
                };
                frame.push(Rc::new(crate::objects::bool::PyBool::new(is_eq)));
            }
            Opcode::CompareIsNot => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                let is_eq = if left.get_type() == "NoneType" && right.get_type() == "NoneType" {
                    true
                } else if left.get_type() == "bool" && right.get_type() == "bool" {
                    left.is_truthy() == right.is_truthy()
                } else if left.get_type() == "type"
                    && right.get_type() == "builtin_function_or_method"
                {
                    if let Some(t) = left
                        .as_any()
                        .downcast_ref::<crate::objects::typeobj::PyType>()
                    {
                        if let Some(nf) = right
                            .as_any()
                            .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                        ) {
                            t.name == nf.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if left.get_type() == "builtin_function_or_method"
                    && right.get_type() == "type"
                {
                    if let Some(nf) = left
                        .as_any()
                        .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
                    {
                        if let Some(t) = right
                            .as_any()
                            .downcast_ref::<crate::objects::typeobj::PyType>()
                        {
                            nf.name == t.name
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    Rc::ptr_eq(&left, &right)
                };
                frame.push(Rc::new(crate::objects::bool::PyBool::new(!is_eq)));
            }
            Opcode::JumpForward(offset) => {
                frame.ip += offset;
            }
            Opcode::PopJumpIfTrue(target) => {
                let cond = frame.pop()?;
                if cond.is_truthy() {
                    frame.ip = *target;
                }
            }
            Opcode::SetupFinally(target) => {
                let stack_size = frame.stack.len();
                frame
                    .block_stack
                    .push(crate::vm::frame::Block::SetupFinally {
                        handler_ip: *target,
                        stack_size,
                    });
            }
            Opcode::PopFinally => {
                frame.block_stack.pop();
                frame.pending_exception = None;
            }
            Opcode::EndFinally => {
                if let Some(exc) = frame.pending_exception.take() {
                    self.last_exception = Some(exc.clone());
                    let msg = if let Some(exc_obj) =
                        exc.as_any()
                            .downcast_ref::<crate::objects::exception::PyException>()
                    {
                        let s = exc_obj.str();
                        if s.is_empty() {
                            exc_obj.repr()
                        } else {
                            format!("{}: {}", exc_obj.exc_type, s)
                        }
                    } else {
                        exc.repr()
                    };
                    return Err(msg);
                }
            }
            Opcode::ExceptionMatch(target_type_name) => {
                let exc_obj = frame.pop()?;
                let is_match = if let Some(py_exc) = exc_obj
                    .as_any()
                    .downcast_ref::<crate::objects::exception::PyException>(
                ) {
                    crate::objects::exception::is_exception_subclass(
                        &py_exc.exc_type,
                        target_type_name,
                    )
                } else if let Some(inst) = exc_obj
                    .as_any()
                    .downcast_ref::<crate::objects::instance::PyInstance>()
                {
                    inst.class.name == *target_type_name
                        || inst.class.mro.iter().any(|base| {
                            if let Some(base_cls) = base
                                .as_any()
                                .downcast_ref::<crate::objects::class::PyClass>()
                            {
                                base_cls.name == *target_type_name
                            } else {
                                false
                            }
                        })
                        || target_type_name == "Exception"
                        || target_type_name == "BaseException"
                } else {
                    false
                };
                frame.push(Rc::new(crate::objects::bool::PyBool::new(is_match)));
            }
            Opcode::TryEnd => {}
            Opcode::Raise => {
                let exc = if frame.stack.len() >= 2 {
                    let exc = frame.pop()?;
                    let cause = frame.pop()?;
                    if !cause.as_any().is::<crate::objects::none::PyNone>() {
                        let _ = exc.set_attr("__cause__", cause);
                    }
                    exc
                } else if frame.stack.len() == 1 {
                    frame.pop()?
                } else if let Some(last) = self.last_exception.clone() {
                    last
                } else {
                    return Err("RuntimeError: no active exception to re-raise".to_string());
                };
                self.last_exception = Some(exc.clone());
                let msg = if let Some(exc_obj) = exc
                    .as_any()
                    .downcast_ref::<crate::objects::exception::PyException>()
                {
                    let s = exc_obj.str();
                    if s.is_empty() {
                        exc_obj.repr()
                    } else {
                        format!("{}: {}", exc_obj.exc_type, s)
                    }
                } else {
                    exc.repr()
                };
                return Err(msg);
            }
            Opcode::SetupWith(target) => {
                let context_manager = frame.pop()?;
                let enter_func = context_manager.get_attr("__enter__")?;
                let exit_func = context_manager.get_attr("__exit__")?;

                let enter_result =
                    self.invoke(enter_func, vec![], std::collections::HashMap::new())?;

                let stack_size = frame.stack.len();
                frame.block_stack.push(crate::vm::frame::Block::SetupWith {
                    handler_ip: *target,
                    stack_size,
                    exit_func,
                });

                frame.push(enter_result);
            }
            Opcode::WithCleanup => {
                let block = frame.block_stack.pop();
                if let Some(crate::vm::frame::Block::SetupWith { exit_func, .. }) = block {
                    let none = Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>;
                    self.invoke(
                        exit_func,
                        vec![none.clone(), none.clone(), none],
                        std::collections::HashMap::new(),
                    )?;
                } else {
                    return Err("CompilerError: WithCleanup expected SetupWith block".to_string());
                }
            }
            Opcode::ImportName(idx) => {
                // Stack: ... , fromlist (tuple), level (int)
                let fromlist_obj = frame.pop()?;
                let level_obj = frame.pop()?;
                let name = &frame.code.names[*idx];

                let level = level_obj
                    .as_any()
                    .downcast_ref::<PyInt>()
                    .and_then(|i| i.as_i64())
                    .unwrap_or(0);

                // Call __import__(name, globals, locals, fromlist, level)
                let import_func_name = "__import__".to_string();
                let import_func = frame
                    .env
                    .borrow()
                    .get(&import_func_name)
                    .ok_or_else(|| "NameError: name '__import__' is not defined".to_string())?;

                let name_obj = Rc::new(PyString::new(name.clone())) as Rc<dyn PyObject>;

                // globals = current env's locals (includes __file__ if set)
                let globals_dict = {
                    let env = frame.env.borrow();
                    let mut pairs = Vec::new();
                    for (k, v) in env.get_all_locals() {
                        pairs.push((Rc::new(PyString::new(k)) as Rc<dyn PyObject>, v));
                    }
                    // Also include __file__ from the code object filename
                    if !frame.code.filename.is_empty() {
                        let file_key =
                            Rc::new(PyString::new("__file__".to_string())) as Rc<dyn PyObject>;
                        let file_val =
                            Rc::new(PyString::new(frame.code.filename.clone())) as Rc<dyn PyObject>;
                        pairs.push((file_key, file_val));
                    }
                    crate::objects::dict::PyDict::from_pairs(pairs)
                };
                let none = Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>;

                let result = self.invoke(
                    import_func,
                    vec![
                        name_obj,
                        Rc::new(globals_dict) as Rc<dyn PyObject>,
                        none,
                        fromlist_obj,
                        Rc::new(crate::objects::int::PyInt::from_i64(level)) as Rc<dyn PyObject>,
                    ],
                    std::collections::HashMap::new(),
                )?;
                frame.push(result);
            }
            Opcode::ImportFrom(idx) => {
                // Stack: ... , module
                // After: ... , module, attr (module below attr)
                let module = frame.pop()?;
                let attr_name = &frame.code.names[*idx];
                let attr_val = match module.get_attr(attr_name) {
                    Ok(val) => val,
                    Err(e) => {
                        // Check if it is a submodule in the package
                        let has_submodule = if let Ok(file_obj) = module.get_attr("__file__") {
                            let file_str = file_obj.str();
                            let path = std::path::Path::new(&file_str);
                            let pkg_dir = if file_str.ends_with("__init__.py") {
                                path.parent()
                            } else if path.is_dir() {
                                Some(path)
                            } else {
                                None
                            };

                            if let Some(dir) = pkg_dir {
                                let py_file = dir.join(format!("{}.py", attr_name));
                                let subpkg = dir.join(attr_name).join("__init__.py");
                                py_file.exists() || subpkg.exists()
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if has_submodule {
                            // Call __import__(attr_name, globals, locals, None, 1)
                            let import_func =
                                frame.env.borrow().get("__import__").ok_or_else(|| {
                                    "NameError: name '__import__' is not defined".to_string()
                                })?;
                            let name_obj =
                                Rc::new(PyString::new(attr_name.clone())) as Rc<dyn PyObject>;
                            // globals
                            let globals_dict = {
                                let env = frame.env.borrow();
                                let mut pairs = Vec::new();
                                for (k, v) in env.get_all_locals() {
                                    pairs.push((Rc::new(PyString::new(k)) as Rc<dyn PyObject>, v));
                                }
                                if let Ok(file_obj) = module.get_attr("__file__") {
                                    pairs.push((
                                        Rc::new(PyString::new("__file__".to_string()))
                                            as Rc<dyn PyObject>,
                                        file_obj,
                                    ));
                                } else if !frame.code.filename.is_empty() {
                                    pairs.push((
                                        Rc::new(PyString::new("__file__".to_string()))
                                            as Rc<dyn PyObject>,
                                        Rc::new(PyString::new(frame.code.filename.clone()))
                                            as Rc<dyn PyObject>,
                                    ));
                                }
                                crate::objects::dict::PyDict::from_pairs(pairs)
                            };
                            let none = Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>;
                            let level_obj = Rc::new(crate::objects::int::PyInt::from_i64(1))
                                as Rc<dyn PyObject>;

                            let submodule = self.invoke(
                                import_func,
                                vec![
                                    name_obj,
                                    Rc::new(globals_dict) as Rc<dyn PyObject>,
                                    none.clone(),
                                    none,
                                    level_obj,
                                ],
                                std::collections::HashMap::new(),
                            )?;

                            // Set submodule as attribute on module
                            module.set_attr(attr_name, Rc::clone(&submodule))?;
                            submodule
                        } else {
                            return Err(e);
                        }
                    }
                };
                frame.push(module); // Push module back first
                frame.push(attr_val); // Then push attr on top
            }
            Opcode::ImportStar => {
                let module = frame.pop()?;
                // Get module's __dict__
                let dict_obj = module.get_attr("__dict__")?;
                if let Some(dict) = dict_obj
                    .as_any()
                    .downcast_ref::<crate::objects::dict::PyDict>()
                {
                    let entries = dict.entries.borrow();
                    let mut env = frame.env.borrow_mut();
                    for bucket in entries.values() {
                        for (k, v) in bucket {
                            if let Some(s) = k.as_any().downcast_ref::<PyString>() {
                                // Skip private names (starting with _)
                                if !s.value.starts_with('_') {
                                    env.set(s.value.clone(), Rc::clone(v));
                                }
                            }
                        }
                    }
                } else {
                    return Err("TypeError: module.__dict__ is not a dict".to_string());
                }
            }
            Opcode::GetAwaitable => {
                let obj = frame.pop()?;
                let await_method = obj.get_attr("__await__")?;
                let iterator =
                    self.invoke(await_method, vec![], std::collections::HashMap::new())?;
                frame.push(iterator);
            }
            Opcode::YieldFrom => {
                let send_val = frame.pop()?;
                let iterator = frame.pop()?;

                let next_res = if send_val.get_type() != "NoneType" {
                    if let Some(generator) = iterator
                        .as_any()
                        .downcast_ref::<crate::objects::generator::PyGenerator>()
                    {
                        // Push send_val to the generator's frame
                        generator.frame.borrow_mut().push(send_val);
                        iterator.get_next()
                    } else if let Some(coroutine) = iterator
                        .as_any()
                        .downcast_ref::<crate::objects::coroutine::PyCoroutine>(
                    ) {
                        coroutine.frame.borrow_mut().push(send_val);
                        iterator.get_next()
                    } else {
                        return Err(format!(
                            "AttributeError: '{}' object has no attribute 'send'",
                            iterator.get_type()
                        ));
                    }
                } else {
                    iterator.get_next()
                };

                loop {
                    match next_res? {
                        Some(val) => {
                            // The inner coroutine yielded. Re-yield this value.
                            frame.push(iterator);
                            // YieldFrom pushes the yielded value to the caller
                            // The YieldFrom instruction pauses the generator.
                            // When it resumes (e.g. via next()), it expects the send_val to be on the stack.
                            // Currently our `get_next()` doesn't push the send_val, so we simulate it by pushing None.
                            frame.push(Rc::new(crate::objects::none::PyNone::new()));
                            frame.ip -= 1;
                            return Ok(Some(val));
                        }
                        None => {
                            // Inner coroutine completed. Check for return value.
                            let ret_val = if let Some(generator) =
                                iterator
                                    .as_any()
                                    .downcast_ref::<crate::objects::generator::PyGenerator>()
                            {
                                let f = generator.frame.borrow();
                                f.return_value
                                    .clone()
                                    .unwrap_or_else(|| Rc::new(crate::objects::none::PyNone::new()))
                            } else if let Some(coroutine) = iterator
                                .as_any()
                                .downcast_ref::<crate::objects::coroutine::PyCoroutine>(
                            ) {
                                let f = coroutine.frame.borrow();
                                f.return_value
                                    .clone()
                                    .unwrap_or_else(|| Rc::new(crate::objects::none::PyNone::new()))
                            } else {
                                Rc::new(crate::objects::none::PyNone::new())
                            };
                            frame.push(ret_val);
                            break;
                        }
                    }
                }
            }
            Opcode::MatchMapping => {
                let subj = frame.pop()?;
                let is_dict = subj.get_type() == "dict";
                frame.push(subj);
                frame.push(Rc::new(crate::objects::bool::PyBool::new(is_dict)));
            }
            Opcode::MatchClassCheck => {
                let class_obj = frame.pop()?;
                let subj = frame.pop()?;
                let class_name = if let Some(cls) = class_obj
                    .as_any()
                    .downcast_ref::<crate::objects::class::PyClass>()
                {
                    Some(cls.name.clone())
                } else if let Some(nf) = class_obj
                    .as_any()
                    .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                ) {
                    Some(nf.name.clone())
                } else if let Some(tp) = class_obj
                    .as_any()
                    .downcast_ref::<crate::objects::typeobj::PyType>()
                {
                    Some(tp.name.clone())
                } else {
                    None
                };
                let is_match = if let Some(name) = class_name {
                    if let Some(inst) = subj
                        .as_any()
                        .downcast_ref::<crate::objects::instance::PyInstance>()
                    {
                        inst.class.name == name
                            || inst.class.mro.iter().any(|base| {
                                base.as_any()
                                    .downcast_ref::<crate::objects::class::PyClass>()
                                    .map_or(false, |c| c.name == name)
                            })
                    } else if let Some(exc) = subj
                        .as_any()
                        .downcast_ref::<crate::objects::exception::PyException>()
                    {
                        exc.exc_type == name
                    } else {
                        // For built-in types, match by type name
                        subj.get_type() == name || name == "object"
                    }
                } else {
                    false
                };
                frame.push(subj);
                frame.push(Rc::new(crate::objects::bool::PyBool::new(is_match)));
            }
            Opcode::MatchClassGetPos(idx) => {
                let subj = frame.pop()?;
                let mut attr_val = None;

                let class_obj_opt = if let Some(inst) =
                    subj.as_any()
                        .downcast_ref::<crate::objects::instance::PyInstance>()
                {
                    Some(Rc::clone(&inst.class) as Rc<dyn PyObject>)
                } else {
                    subj.get_attr("__class__").ok()
                };

                if let Some(class_obj) = class_obj_opt {
                    let match_args: Rc<dyn PyObject> = class_obj
                        .get_attr("__match_args__")
                        .unwrap_or_else(|_| Rc::new(crate::objects::none::PyNone::new()));
                    if let Some(t) = match_args
                        .as_any()
                        .downcast_ref::<crate::objects::tuple::PyTuple>()
                    {
                        if *idx < t.elements.len() {
                            if let Some(s) = t.elements[*idx]
                                .as_any()
                                .downcast_ref::<crate::objects::string::PyString>()
                            {
                                attr_val = subj.get_attr(&s.value).ok();
                            }
                        }
                    }
                }

                if let Some(val) = attr_val {
                    frame.push(val);
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(true)));
                } else {
                    frame.push(Rc::new(crate::objects::none::PyNone::new()));
                    frame.push(Rc::new(crate::objects::bool::PyBool::new(false)));
                }
            }
            Opcode::CheckSequence(count) => {
                let subj = frame.pop()?;
                let mut is_seq = false;
                if let Some(t) = subj
                    .as_any()
                    .downcast_ref::<crate::objects::tuple::PyTuple>()
                {
                    is_seq = t.elements.len() == *count;
                } else if let Some(l) = subj.as_any().downcast_ref::<crate::objects::list::PyList>()
                {
                    is_seq = l.elements.borrow().len() == *count;
                }
                frame.push(subj);
                frame.push(Rc::new(crate::objects::bool::PyBool::new(is_seq)));
            }
        }
        Ok(None)
    }

    pub fn invoke(
        &mut self,
        func_obj: Rc<dyn PyObject>,
        args: Vec<Rc<dyn PyObject>>,
        kwargs: std::collections::HashMap<String, Rc<dyn PyObject>>,
    ) -> Result<Rc<dyn PyObject>, String> {
        // Set up the thread-local VM pointer once so PyInstance::call_dunder
        // can invoke Python functions (PyFunction).
        let vm_ptr: *mut VirtualMachine = self;
        crate::objects::instance::set_vm_ptr(vm_ptr as *mut ());
        self.invoke_inner(func_obj, args, kwargs)
    }

    fn invoke_inner(
        &mut self,
        func_obj: Rc<dyn PyObject>,
        args: Vec<Rc<dyn PyObject>>,
        kwargs: std::collections::HashMap<String, Rc<dyn PyObject>>,
    ) -> Result<Rc<dyn PyObject>, String> {
        if let Some(func) = func_obj.as_any().downcast_ref::<PyFunction>() {
            let new_env = Environment::new_enclosed(Rc::clone(&func.env));

            let mut vararg_extra: Vec<Rc<dyn PyObject>> = Vec::new();
            let param_count = func.params.len();

            // 1. Bind positional arguments (to posonly + regular params only)
            for (i, arg) in args.iter().enumerate() {
                if i < param_count {
                    new_env
                        .borrow_mut()
                        .set(func.params[i].clone(), Rc::clone(arg));
                } else if func.code.vararg.is_some() {
                    vararg_extra.push(Rc::clone(arg));
                } else {
                    return Err(format!(
                        "TypeError: {}() takes {} positional arguments but {} were given",
                        func.code.name,
                        param_count,
                        args.len()
                    ));
                }
            }

            // 2. Store *vararg as a tuple
            if let Some(vararg) = &func.code.vararg {
                let tup = Rc::new(crate::objects::tuple::PyTuple::new(vararg_extra));
                new_env.borrow_mut().set(vararg.clone(), tup);
            }

            // 3. Apply default parameter values for missing positional arguments
            let default_count = func.defaults.len();
            let pos_count = args.len().min(param_count);
            let mandatory_count = param_count.saturating_sub(default_count);
            if pos_count < param_count {
                for i in pos_count..param_count {
                    if i >= mandatory_count {
                        let default_idx = i - mandatory_count;
                        if default_idx < default_count {
                            new_env.borrow_mut().set(
                                func.params[i].clone(),
                                Rc::clone(&func.defaults[default_idx]),
                            );
                        }
                    }
                }
            }

            // 4. Bind keyword arguments
            let mut unused_kwargs = std::collections::HashMap::new();
            for (key, val) in kwargs {
                // Check if it's a positional-only param -> error
                if let Some(idx) = func.params.iter().position(|p| p == &key) {
                    if idx < func.posonly_count {
                        return Err(format!(
                            "TypeError: '{}' is a positional-only parameter",
                            key
                        ));
                    }
                }
                if func.params.contains(&key) {
                    new_env.borrow_mut().set(key, val);
                } else if func.kwonly_params.contains(&key) {
                    new_env.borrow_mut().set(key, val);
                } else {
                    unused_kwargs.insert(key, val);
                }
            }

            // 5. Apply defaults for keyword-only arguments (right-aligned)
            let kw_default_offset = func
                .kwonly_params
                .len()
                .saturating_sub(func.kwonly_defaults.len());
            for (i, param) in func.kwonly_params.iter().enumerate() {
                if new_env.borrow().get(param).is_none() {
                    if i >= kw_default_offset {
                        let kw_idx = i - kw_default_offset;
                        if kw_idx < func.kwonly_defaults.len() {
                            new_env
                                .borrow_mut()
                                .set(param.clone(), Rc::clone(&func.kwonly_defaults[kw_idx]));
                            continue;
                        }
                    }
                    return Err(format!(
                        "TypeError: missing required keyword-only argument '{}'",
                        param
                    ));
                }
            }

            // 6. Handle **kwargs if kwarg is present
            if let Some(kwarg) = &func.code.kwarg {
                let mut pairs: Vec<(Rc<dyn PyObject>, Rc<dyn PyObject>)> = Vec::new();
                for (k, v) in unused_kwargs {
                    pairs.push((
                        Rc::new(crate::objects::string::PyString::new(k)) as Rc<dyn PyObject>,
                        v,
                    ));
                }
                let dict = Rc::new(crate::objects::dict::PyDict::from_pairs(pairs));
                new_env.borrow_mut().set(kwarg.clone(), dict);
            } else if !unused_kwargs.is_empty() {
                let first_unexpected = unused_kwargs.keys().next().unwrap();
                return Err(format!(
                    "TypeError: {}() got an unexpected keyword argument '{}'",
                    func.code.name, first_unexpected
                ));
            }

            // 7. Check all required positional params are bound
            for i in 0..param_count {
                if new_env.borrow().get(&func.params[i]).is_none() {
                    return Err(format!(
                        "TypeError: missing required positional argument '{}'",
                        func.params[i]
                    ));
                }
            }

            let mut new_frame = Frame::new(func.code.clone(), new_env);

            if func.code.is_async {
                Ok(Rc::new(crate::objects::coroutine::PyCoroutine::new(
                    new_frame,
                )))
            } else if func.code.is_generator {
                Ok(Rc::new(crate::objects::generator::PyGenerator::new(
                    new_frame,
                )))
            } else {
                let run_res = self.run(&mut new_frame);
                match run_res {
                    Ok(result) => {
                        if let Some(res) = result {
                            Ok(res)
                        } else {
                            Err("Function returned without a value".to_string())
                        }
                    }
                    Err(e) => {
                        let traceback_started =
                            e.starts_with("Traceback (most recent call last):\n");
                        let new_err = if traceback_started {
                            let mut lines: Vec<String> =
                                e.split('\n').map(|s| s.to_string()).collect();
                            let last_line = lines.pop().unwrap_or_default();
                            lines.insert(
                                1,
                                format!(
                                    "  File \"{}\", in {}",
                                    new_frame.code.filename, new_frame.code.name
                                ),
                            );
                            lines.push(last_line);
                            lines.join("\n")
                        } else {
                            format!(
                                "Traceback (most recent call last):\n  File \"{}\", in {}\n{}",
                                new_frame.code.filename, new_frame.code.name, e
                            )
                        };
                        Err(new_err)
                    }
                }
            }
        } else if let Some(native_func) = func_obj
            .as_any()
            .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
        ) {
            (native_func.func)(args, kwargs)
        } else if let Some(class_obj) = func_obj
            .as_any()
            .downcast_ref::<crate::objects::class::PyClass>()
        {
            let instance = Rc::new(crate::objects::instance::PyInstance::new(Rc::new(
                class_obj.clone(),
            )));
            if let Ok(init_func) = instance.get_attr("__init__") {
                if let Some(_bound_method) = init_func
                    .as_any()
                    .downcast_ref::<crate::objects::bound_method::PyBoundMethod>(
                ) {
                    self.invoke(init_func, args, kwargs)?;
                }
            }
            Ok(instance)
        } else if let Some(inst) = func_obj
            .as_any()
            .downcast_ref::<crate::objects::instance::PyInstance>()
        {
            let call_method = inst.get_attr("__call__")?;
            self.invoke(call_method, args, kwargs)
        } else if let Some(bound_method) = func_obj
            .as_any()
            .downcast_ref::<crate::objects::bound_method::PyBoundMethod>(
        ) {
            let mut bound_args = vec![Rc::clone(&bound_method.instance)];
            bound_args.extend(args);
            self.invoke(Rc::clone(&bound_method.func), bound_args, kwargs)
        } else {
            Err(format!(
                "TypeError: '{}' object is not callable",
                func_obj.get_type()
            ))
        }
    }
}
