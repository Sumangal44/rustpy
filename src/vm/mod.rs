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
                    let exc_obj = self.last_exception.take().unwrap_or_else(|| {
                        Rc::new(crate::objects::exception::PyException::new(
                            "RuntimeError".to_string(),
                            Some(e.clone()),
                        ))
                    });
                    self.last_exception = Some(exc_obj.clone());

                    let mut handled = false;
                    while let Some(block) = frame.block_stack.pop() {
                        match block {
                            crate::vm::frame::Block::SetupExcept { handler_ip, stack_size } => {
                                frame.stack.truncate(stack_size);
                                frame.push(exc_obj.clone());
                                frame.ip = handler_ip;
                                handled = true;
                                break;
                            }
                            crate::vm::frame::Block::SetupWith { handler_ip, stack_size, exit_func } => {
                                frame.stack.truncate(stack_size);
                                let none = Rc::new(crate::objects::none::PyNone) as Rc<dyn PyObject>;
                                let exc_type = Rc::new(crate::objects::string::PyString::new("Exception".to_string())) as Rc<dyn PyObject>;
                                let exc_val = exc_obj.clone();
                                
                                let exit_res = self.invoke(exit_func, vec![exc_type, exc_val, none], std::collections::HashMap::new());
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
            Opcode::DupTop => {
                let val = frame.last()?;
                frame.push(val);
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
                // For simplicity, we just set the attribute to None (we can't remove it from the trait easily)
                // Actually, let's just set it to PyNone
                obj.set_attr(attr_name, Rc::new(crate::objects::none::PyNone))?;
            }
            Opcode::DeleteSubscript => {
                let idx = frame.pop()?;
                let collection = frame.pop()?;
                // For simplicity, we set the item to None via the subscript
                // Actually, we need a del_item method. Let's just error for now.
                return Err("TypeError: '{}' object does not support item deletion".to_string());
            }
            Opcode::StoreSubscript => {
                let value = frame.pop()?;
                let idx = frame.pop()?;
                let collection = frame.pop()?;
                // Call set_item on the collection
                // We don't have set_item on the trait, so let's implement it
                // For now, return an error for non-list types
                if let Some(list) = collection.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    if let Some(int_idx) = idx.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                        let mut elements = list.elements.borrow_mut();
                        let mut idx_val = int_idx.value;
                        if idx_val < 0 {
                            idx_val += elements.len() as i64;
                        }
                        if idx_val >= 0 && (idx_val as usize) < elements.len() {
                            elements[idx_val as usize] = value;
                        } else {
                            return Err("IndexError: list assignment index out of range".to_string());
                        }
                    } else {
                        return Err("TypeError: list indices must be integers, not ...".to_string());
                    }
                } else {
                    return Err("TypeError: '{}' object does not support item assignment".to_string());
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
                if let Some(code) = code_obj.as_any().downcast_ref::<CodeObject>() {
                    let name = code.name.clone();
                    let params = code
                        .names
                        .iter()
                        .take(code.arg_count)
                        .cloned()
                        .collect::<Vec<_>>();

                    let func = Rc::new(PyFunction::new(
                        name,
                        params,
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
                let result = self.invoke(func_obj, args, std::collections::HashMap::new())?;
                frame.push(result);
            }
            Opcode::CallFunctionKw(argc) => {
                let kwarg_names_obj = frame.pop()?;
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(frame.pop()?);
                }
                args.reverse();

                let func_obj = frame.pop()?;

                let mut kwargs = std::collections::HashMap::new();
                // This opcode isn't generated currently (we use BuildMap and CallFunctionEx instead), 
                // but implemented just in case for parity.
                let result = self.invoke(func_obj, args, kwargs)?;
                frame.push(result);
            }
            Opcode::CallFunctionEx(flags) => {
                let mut kwargs = std::collections::HashMap::new();
                if *flags & 1 != 0 {
                    let kwargs_dict_obj = frame.pop()?;
                    if let Some(dict) = kwargs_dict_obj.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                        for (k, v) in dict.entries.borrow().iter() {
                            kwargs.insert(k.clone(), Rc::clone(v));
                        }
                    } else {
                        return Err("TypeError: kwargs must be a dict".to_string());
                    }
                }

                let args_iter_obj = frame.pop()?;
                let mut args = Vec::new();
                if let Some(list) = args_iter_obj.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    for el in list.elements.borrow().iter() {
                        args.push(Rc::clone(el));
                    }
                } else {
                    return Err("TypeError: args must be an iterable".to_string());
                }

                let func_obj = frame.pop()?;
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
            Opcode::BuildMap(count) => {
                let mut entries = std::collections::HashMap::new();
                for _ in 0..*count {
                    let value = frame.pop()?;
                    let key_obj = frame.pop()?;

                    if let Some(str_key) = key_obj.as_any().downcast_ref::<crate::objects::string::PyString>() {
                        entries.insert(str_key.value.clone(), value);
                    } else {
                        return Err(format!("TypeError: unhashable type: '{}' (Only strings supported as dict keys)", key_obj.get_type()));
                    }
                }
                let dict = Rc::new(crate::objects::dict::PyDict::new(entries));
                frame.push(dict);
            }
            Opcode::ListExtend => {
                let iterable = frame.pop()?;
                let list_obj = frame.pop()?;

                if let Some(list) = list_obj.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    if let Some(other_list) = iterable.as_any().downcast_ref::<crate::objects::list::PyList>() {
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

                if let Some(dict1) = dict1_obj.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                    if let Some(dict2) = dict2_obj.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                        for (k, v) in dict2.entries.borrow().iter() {
                            dict1.entries.borrow_mut().insert(k.clone(), Rc::clone(v));
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
                if let Some(list) = list_obj.as_any().downcast_ref::<crate::objects::list::PyList>() {
                    list.elements.borrow_mut().push(item);
                    frame.push(list_obj);
                } else {
                    return Err("TypeError: ListAppend expected a list".to_string());
                }
            }
            Opcode::MapAdd => {
                let value = frame.pop()?;
                let key = frame.pop()?;
                let dict_obj = frame.pop()?;
                if let Some(dict) = dict_obj.as_any().downcast_ref::<crate::objects::dict::PyDict>() {
                    let key_str = key.str();
                    dict.entries.borrow_mut().insert(key_str, value);
                    frame.push(dict_obj);
                } else {
                    return Err("TypeError: MapAdd expected a dict".to_string());
                }
            }
            Opcode::BuildSlice => {
                let step_obj = frame.pop()?;
                let stop_obj = frame.pop()?;
                let start_obj = frame.pop()?;
                let start = if start_obj.as_any().downcast_ref::<crate::objects::none::PyNone>().is_some() {
                    None
                } else if let Some(i) = start_obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Some(i.value)
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                let stop = if stop_obj.as_any().downcast_ref::<crate::objects::none::PyNone>().is_some() {
                    None
                } else if let Some(i) = stop_obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Some(i.value)
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                let step = if step_obj.as_any().downcast_ref::<crate::objects::none::PyNone>().is_some() {
                    None
                } else if let Some(i) = step_obj.as_any().downcast_ref::<crate::objects::int::PyInt>() {
                    Some(i.value)
                } else {
                    return Err("TypeError: slice indices must be integers or None".to_string());
                };
                if let Some(s) = step {
                    if s == 0 { return Err("ValueError: slice step cannot be zero".to_string()); }
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
            Opcode::BuildClass(num_bases) => {
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

                // We need a way to run this frame without returning the final result (or just discard it)
                // The methods are now stored in class_env
                self.run(&mut class_frame)?;

                // Extract methods from class_env
                let attributes = class_env.borrow().get_all_locals();

                let class = crate::objects::class::PyClass::new(name, attributes, bases)?;
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
                if frame.stack.is_empty() {
                    return Ok(Some(Rc::new(crate::objects::none::PyNone)));
                }
                return Ok(Some(frame.pop()?));
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
                if let Some(result) = left.matmul(Rc::clone(&right)) {
                    frame.push(result);
                } else if let Some(result) = right.rmatmul(Rc::clone(&left)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for @: '{}' and '{}'", left.get_type(), right.get_type()));
                }
            }
            Opcode::BinaryBitAnd => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitand(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for &: '{}' and '{}'", left.get_type(), right.get_type()));
                }
            }
            Opcode::BinaryBitOr => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitor(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for |: '{}' and '{}'", left.get_type(), right.get_type()));
                }
            }
            Opcode::BinaryBitXor => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.bitxor(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for ^: '{}' and '{}'", left.get_type(), right.get_type()));
                }
            }
            Opcode::BinaryLShift => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.lshift(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for <<: '{}' and '{}'", left.get_type(), right.get_type()));
                }
            }
            Opcode::BinaryRShift => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.rshift(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!("TypeError: unsupported operand type(s) for >>: '{}' and '{}'", left.get_type(), right.get_type()));
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
                    return Err(format!("TypeError: bad operand type for unary ~: '{}'", val.get_type()));
                }
            }
            Opcode::CompareEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.eq(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with ==",
                        left.get_type(),
                        right.get_type()
                    ));
                }
            }
            Opcode::CompareNotEq => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                if let Some(result) = left.ne(Rc::clone(&right)) {
                    frame.push(result);
                } else {
                    return Err(format!(
                        "TypeError: '{}' and '{}' are not comparable with !=",
                        left.get_type(),
                        right.get_type()
                    ));
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
                frame.push(Rc::new(crate::objects::bool::PyBool::new(Rc::ptr_eq(&left, &right))));
            }
            Opcode::CompareIsNot => {
                let right = frame.pop()?;
                let left = frame.pop()?;
                frame.push(Rc::new(crate::objects::bool::PyBool::new(!Rc::ptr_eq(&left, &right))));
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
            Opcode::Raise => {
                let exc = frame.pop()?;
                self.last_exception = Some(exc.clone());
                // We expect exc to be an Exception object
                return Err(exc.repr());
            }
            Opcode::SetupWith(target) => {
                let context_manager = frame.pop()?;
                let enter_func = context_manager.get_attr("__enter__")?;
                let exit_func = context_manager.get_attr("__exit__")?;

                let enter_result = self.invoke(enter_func, vec![], std::collections::HashMap::new())?;
                
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
                    self.invoke(exit_func, vec![none.clone(), none.clone(), none], std::collections::HashMap::new())?;
                } else {
                    return Err("CompilerError: WithCleanup expected SetupWith block".to_string());
                }
            }
            _ => return Err(format!("Opcode {:?} not yet implemented in VM", opcode)),
        }
        Ok(None)
    }

    pub fn invoke(
        &mut self,
        func_obj: Rc<dyn PyObject>,
        args: Vec<Rc<dyn PyObject>>,
        kwargs: std::collections::HashMap<String, Rc<dyn PyObject>>,
    ) -> Result<Rc<dyn PyObject>, String> {
        if let Some(func) = func_obj.as_any().downcast_ref::<PyFunction>() {
            let new_env = Environment::new_enclosed(Rc::clone(&func.env));
            
            // 1. Bind positional arguments
            for (i, arg) in args.iter().enumerate() {
                if i < func.params.len() {
                    new_env.borrow_mut().set(func.params[i].clone(), Rc::clone(arg));
                } else {
                    // Collect into *args if vararg is present
                    if let Some(vararg) = &func.code.vararg {
                        let mut env = new_env.borrow_mut();
                        let existing = env.get(vararg);
                        if let Some(existing_tuple) = existing {
                            // Append to tuple (list for now)
                            if let Some(list) = existing_tuple.as_any().downcast_ref::<crate::objects::list::PyList>() {
                                list.elements.borrow_mut().push(Rc::clone(arg));
                            }
                        } else {
                            // Create new tuple (list for now)
                            let list = Rc::new(crate::objects::list::PyList::new(vec![Rc::clone(arg)]));
                            env.set(vararg.clone(), list);
                        }
                    } else {
                        return Err(format!("TypeError: {}() takes {} positional arguments but {} were given", func.code.name, func.params.len(), args.len()));
                    }
                }
            }

            // 2. Initialize empty *args if vararg is present but no extra args were given
            if let Some(vararg) = &func.code.vararg {
                let mut env = new_env.borrow_mut();
                if env.get(vararg).is_none() {
                    let list = Rc::new(crate::objects::list::PyList::new(Vec::new()));
                    env.set(vararg.clone(), list);
                }
            }

            // 3. Bind keyword arguments
            let mut unused_kwargs = std::collections::HashMap::new();
            for (key, val) in kwargs {
                if func.params.contains(&key) {
                    new_env.borrow_mut().set(key, val);
                } else {
                    unused_kwargs.insert(key, val);
                }
            }

            // 4. Handle **kwargs if kwarg is present
            if let Some(kwarg) = &func.code.kwarg {
                let dict = Rc::new(crate::objects::dict::PyDict::new(unused_kwargs));
                new_env.borrow_mut().set(kwarg.clone(), dict);
            } else if !unused_kwargs.is_empty() {
                let first_unexpected = unused_kwargs.keys().next().unwrap();
                return Err(format!("TypeError: {}() got an unexpected keyword argument '{}'", func.code.name, first_unexpected));
            }

            let mut new_frame = Frame::new(func.code.clone(), new_env);

            if func.code.is_generator {
                Ok(Rc::new(crate::objects::generator::PyGenerator::new(new_frame)))
            } else {
                if let Some(result) = self.run(&mut new_frame)? {
                    Ok(result)
                } else {
                    Err("Function returned without a value".to_string())
                }
            }
        } else if let Some(native_func) = func_obj.as_any().downcast_ref::<crate::objects::native_function::PyNativeFunction>() {
            // We ignore kwargs for native functions for now to keep it simple, or pass them if we update NativeFunction signatures.
            // Let's just pass positional args.
            (native_func.func)(args)
        } else if let Some(class_obj) = func_obj.as_any().downcast_ref::<crate::objects::class::PyClass>() {
            let instance = Rc::new(crate::objects::instance::PyInstance::new(Rc::new(class_obj.clone())));
            if let Ok(init_func) = instance.get_attr("__init__") {
                if let Some(bound_method) = init_func.as_any().downcast_ref::<crate::objects::bound_method::PyBoundMethod>() {
                    // Call the underlying function but manually prepend 'self'.
                    // Or we can just call self.invoke on the bound_method!
                    self.invoke(init_func, args, kwargs)?;
                }
            }
            Ok(instance)
        } else if let Some(bound_method) = func_obj.as_any().downcast_ref::<crate::objects::bound_method::PyBoundMethod>() {
            let mut bound_args = vec![Rc::new(bound_method.instance.clone()) as Rc<dyn PyObject>];
            bound_args.extend(args);
            self.invoke(Rc::clone(&bound_method.func), bound_args, kwargs)
        } else {
            Err(format!("TypeError: '{}' object is not callable", func_obj.get_type()))
        }
    }
}
