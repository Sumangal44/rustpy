pub mod frame;

use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::PyObject;
use crate::objects::function::PyFunction;
use crate::runtime::Environment;
use frame::Frame;
use std::rc::Rc;

pub struct VirtualMachine {
    pub last_exception: Option<Rc<dyn PyObject>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            last_exception: None,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) -> Result<Option<Rc<dyn PyObject>>, String> {
        let instructions = frame.code.instructions.clone();

        while frame.ip < instructions.len() {
            let opcode = instructions[frame.ip].clone();
            frame.ip += 1;

            match self.execute_opcode(&opcode, frame) {
                Ok(Some(ret)) => return Ok(Some(ret)),
                Ok(None) => {} // continue
                Err(e) => {
                    if let Some(crate::vm::frame::Block::SetupExcept {
                        handler_ip,
                        stack_size,
                    }) = frame.block_stack.pop()
                    {
                        frame.stack.truncate(stack_size);

                        let exc_obj = self.last_exception.take().unwrap_or_else(|| {
                            // If there's no exception object, create a generic one
                            Rc::new(crate::objects::exception::PyException::new(
                                "RuntimeError".to_string(),
                                Some(e.clone()),
                            ))
                        });

                        frame.push(exc_obj);
                        frame.ip = handler_ip;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

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
                frame.env.borrow_mut().set(name, obj);
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
                if let Some(code) = code_obj.as_any().downcast_ref::<CodeObject>() {
                    let name = code.name.clone();
                    let params = code
                        .names
                        .iter()
                        .take(code.instructions.len())
                        .cloned()
                        .collect::<Vec<_>>(); // Simplification
                    // We actually should pass the real params in CodeObject, let's just grab them from names for now.
                    // Wait, in compiler we added params to names! So they are at the beginning.
                    // It's a bit hacky to slice without knowing exactly how many params there are if we don't store arg_count.
                    // Let's assume the VM doesn't check param names strictly right now, just arg count when calling.
                    let func = Rc::new(PyFunction::new(
                        name,
                        code.names.clone(),
                        code.clone(),
                        Rc::clone(&frame.env),
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
                if let Some(func) = func_obj.as_any().downcast_ref::<PyFunction>() {
                    // Create a new environment bounded to the function's closure env
                    let new_env = Environment::new_enclosed(Rc::clone(&func.env));

                    // Bind arguments to parameters (we assume names[0..argc] are the params)
                    for (i, arg) in args.into_iter().enumerate() {
                        if i < func.params.len() {
                            let param_name = func.params[i].clone();
                            new_env.borrow_mut().set(param_name, arg);
                        }
                    }

                    // Create a new frame and execute it!
                    let mut new_frame = Frame::new(func.code.clone(), new_env);
                    if let Some(result) = self.run(&mut new_frame)? {
                        frame.push(result);
                    } else {
                        return Err("Function returned without a value".to_string());
                    }
                } else if let Some(native_func) = func_obj
                    .as_any()
                    .downcast_ref::<crate::objects::native_function::PyNativeFunction>(
                ) {
                    // Execute native Rust function directly
                    let result = (native_func.func)(args)?;
                    frame.push(result);
                } else if let Some(class_obj) = func_obj
                    .as_any()
                    .downcast_ref::<crate::objects::class::PyClass>()
                {
                    // Instantiation!
                    let instance = Rc::new(crate::objects::instance::PyInstance::new(Rc::new(
                        class_obj.clone(),
                    )));

                    // Check for __init__
                    if let Ok(init_func) = instance.get_attr("__init__") {
                        // It returns a bound method, so we can call it.
                        if let Some(bound_method) = init_func
                            .as_any()
                            .downcast_ref::<crate::objects::bound_method::PyBoundMethod>(
                        ) {
                            // Extract the underlying function
                            if let Some(func) =
                                bound_method.func.as_any().downcast_ref::<PyFunction>()
                            {
                                let new_env = Environment::new_enclosed(Rc::clone(&func.env));

                                // Bind 'self' as the first parameter
                                if !func.params.is_empty() {
                                    new_env.borrow_mut().set(
                                        func.params[0].clone(),
                                        Rc::clone(&instance) as Rc<dyn PyObject>,
                                    );
                                }

                                // Bind the rest of the arguments
                                for (i, arg) in args.into_iter().enumerate() {
                                    if i + 1 < func.params.len() {
                                        new_env.borrow_mut().set(func.params[i + 1].clone(), arg);
                                    }
                                }

                                let mut new_frame = Frame::new(func.code.clone(), new_env);
                                self.run(&mut new_frame)?; // __init__ returns None in Python
                            }
                        }
                    }

                    frame.push(instance);
                } else if let Some(bound_method) = func_obj
                    .as_any()
                    .downcast_ref::<crate::objects::bound_method::PyBoundMethod>(
                ) {
                    if let Some(func) = bound_method.func.as_any().downcast_ref::<PyFunction>() {
                        let new_env = Environment::new_enclosed(Rc::clone(&func.env));

                        // Bind 'self' as the first parameter
                        if !func.params.is_empty() {
                            // Convert PyInstance to Rc<dyn PyObject>. Since we cloned PyInstance, we need to wrap it.
                            let instance_rc = Rc::new(bound_method.instance.clone());
                            new_env
                                .borrow_mut()
                                .set(func.params[0].clone(), instance_rc as Rc<dyn PyObject>);
                        }

                        // Bind the rest of the arguments
                        for (i, arg) in args.into_iter().enumerate() {
                            if i + 1 < func.params.len() {
                                new_env.borrow_mut().set(func.params[i + 1].clone(), arg);
                            }
                        }

                        let mut new_frame = Frame::new(func.code.clone(), new_env);
                        if let Some(result) = self.run(&mut new_frame)? {
                            frame.push(result);
                        } else {
                            return Err("Function returned without a value".to_string());
                        }
                    } else {
                        return Err("Bound method does not wrap a PyFunction".to_string());
                    }
                } else {
                    return Err(format!(
                        "TypeError: '{}' object is not callable",
                        func_obj.get_type()
                    ));
                }
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
            Opcode::BuildMap(count) => {
                let mut entries = std::collections::HashMap::new();
                // Each pair was compiled as key, then value.
                // This means value is on top of stack, then key.
                // Actually let's pop pairs.
                for _ in 0..*count {
                    let value = frame.pop()?;
                    let key_obj = frame.pop()?;

                    // We only support strings for keys currently
                    if let Some(str_key) = key_obj
                        .as_any()
                        .downcast_ref::<crate::objects::string::PyString>()
                    {
                        entries.insert(str_key.value.clone(), value);
                    } else {
                        return Err(format!(
                            "TypeError: unhashable type: '{}' (Only strings supported as dict keys)",
                            key_obj.get_type()
                        ));
                    }
                }
                let dict = Rc::new(crate::objects::dict::PyDict::new(entries));
                frame.push(dict);
            }
            Opcode::BinarySubscript => {
                let idx = frame.pop()?;
                let collection = frame.pop()?;
                let result = collection.get_item(idx)?;
                frame.push(result);
            }
            Opcode::GetIter => {
                let collection = frame.pop()?;
                let iterator = collection.get_iter()?;
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
            Opcode::BuildClass => {
                // Pops code_obj, pops name (wait, compile_stmt for ClassDef pushes name, then code_obj)
                // So top is code_obj, under is name.
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

                // We need a way to run this frame without returning the final result (or just discard it)
                // The methods are now stored in class_env
                self.run(&mut class_frame)?;

                // Extract methods from class_env
                // Wait, get_all_locals() returns all variables, including ones from the enclosing environment?
                // No, get_all_locals() is only `self.variables`! So it only gets the class locals!
                let attributes = class_env.borrow().get_all_locals();

                let class = Rc::new(crate::objects::class::PyClass::new(name, attributes));
                frame.push(class);
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
                if frame.stack.is_empty() {
                    return Ok(Some(Rc::new(crate::objects::none::PyNone)));
                }
                return Ok(Some(frame.pop()?));
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
            Opcode::Raise => {
                let exc = frame.pop()?;
                self.last_exception = Some(exc.clone());
                // We expect exc to be an Exception object
                return Err(exc.repr());
            }
            _ => return Err(format!("Opcode {:?} not yet implemented in VM", opcode)),
        }
        Ok(None)
    }
}
