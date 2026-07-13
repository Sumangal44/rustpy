pub mod frame;

use crate::compiler::opcodes::Opcode;
use crate::objects::PyObject;
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
                    if let Some(obj) = frame.env.get(name) {
                        frame.push(obj);
                    } else {
                        return Err(format!("NameError: name '{}' is not defined", name));
                    }
                }
                Opcode::StoreName(idx) => {
                    let name = frame.code.names[*idx].clone();
                    let obj = frame.pop()?;
                    frame.env.set(name, obj);
                }
                Opcode::BinaryAdd => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    if let Some(result) = left.add(right) {
                        frame.push(result);
                    } else {
                        return Err(format!(
                            "TypeError: unsupported operand type(s) for +: '{}' and '{}'",
                            left.get_type(),
                            left.get_type()
                        ));
                    }
                }
                Opcode::BinarySubtract => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    if let Some(result) = left.sub(right) {
                        frame.push(result);
                    } else {
                        return Err(format!(
                            "TypeError: unsupported operand type(s) for -: '{}' and '{}'",
                            left.get_type(),
                            left.get_type()
                        ));
                    }
                }
                Opcode::BinaryMultiply => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    if let Some(result) = left.mul(right) {
                        frame.push(result);
                    } else {
                        return Err(format!(
                            "TypeError: unsupported operand type(s) for *: '{}' and '{}'",
                            left.get_type(),
                            left.get_type()
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
                Opcode::ReturnValue => {
                    // Python usually returns None if stack is empty, but our compiler guarantees
                    // something is on the stack if it emits an explicit return (once we have PyNone).
                    // For now, if the stack has something, pop it. Otherwise return None.
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
