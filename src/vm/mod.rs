pub mod frame;

use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::PyObject;
use crate::objects::function::PyFunction;
use crate::runtime::Environment;
use frame::Frame;
use std::rc::Rc;

pub struct VirtualMachine;

impl VirtualMachine {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, frame: &mut Frame) -> Result<Option<Rc<dyn PyObject>>, String> {
        let instructions = frame.code.instructions.clone();

        while frame.ip < instructions.len() {
            let opcode = &instructions[frame.ip];
            frame.ip += 1;

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
                    } else if let Some(native_func) =
                        func_obj
                            .as_any()
                            .downcast_ref::<crate::objects::native_function::PyNativeFunction>()
                    {
                        // Execute native Rust function directly
                        let result = (native_func.func)(args)?;
                        frame.push(result);
                    } else {
                        return Err(format!(
                            "TypeError: '{}' object is not callable",
                            func_obj.get_type()
                        ));
                    }
                }
                Opcode::ReturnValue => {
                    if frame.stack.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(frame.pop()?));
                }
                _ => return Err(format!("Opcode {:?} not yet implemented in VM", opcode)),
            }
        }

        Ok(None)
    }
}
