pub mod code;
pub mod opcodes;

use crate::ast::{BinOpKind, Expr, Module, Stmt, UnaryOpKind};
use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::{PyObject, int::PyInt};
use std::rc::Rc;

pub struct Compiler {
    code: CodeObject,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            code: CodeObject::new(),
        }
    }

    pub fn compile(mut self, module: &Module) -> Result<CodeObject, String> {
        for stmt in &module.body {
            self.compile_stmt(stmt)?;
        }

        // Modules implicitly return None if they reach the end
        // But for now, we'll just emit a ReturnValue for safety
        self.code.instructions.push(Opcode::ReturnValue);

        Ok(self.code)
    }

    fn emit(&mut self, opcode: Opcode) -> usize {
        let pos = self.code.instructions.len();
        self.code.instructions.push(opcode);
        pos
    }

    fn add_constant(&mut self, obj: Rc<dyn PyObject>) -> usize {
        self.code.constants.push(obj);
        self.code.constants.len() - 1
    }

    fn get_or_add_name(&mut self, name: &str) -> usize {
        if let Some(pos) = self.code.names.iter().position(|n| n == name) {
            pos
        } else {
            self.code.names.push(name.to_string());
            self.code.names.len() - 1
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Assign { targets, value } => {
                self.compile_expr(value)?;

                // For multiple targets (a = b = 1), we would need to DUP the top of the stack.
                // For simplicity right now, we assume 1 target.
                if targets.len() != 1 {
                    return Err("Multiple assignment targets not yet supported".to_string());
                }

                if let Expr::Identifier(name) = &targets[0] {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::StoreName(name_idx));
                } else {
                    return Err("Assignment target must be an identifier".to_string());
                }
            }
            Stmt::ExprStmt { value } => {
                self.compile_expr(value)?;
                self.emit(Opcode::PopTop); // Discard the result of the expression
            }
            Stmt::If { test, body, orelse } => {
                self.compile_expr(test)?;

                let jump_if_false_idx = self.emit(Opcode::PopJumpIfFalse(0)); // Placeholder

                for s in body {
                    self.compile_stmt(s)?;
                }

                if !orelse.is_empty() {
                    let jump_forward_idx = self.emit(Opcode::JumpAbsolute(0)); // Placeholder

                    // Backpatch jump_if_false to jump past the if block to the else block
                    self.code.instructions[jump_if_false_idx] =
                        Opcode::PopJumpIfFalse(self.code.instructions.len());

                    for s in orelse {
                        self.compile_stmt(s)?;
                    }

                    // Backpatch jump_forward to jump past the else block
                    self.code.instructions[jump_forward_idx] =
                        Opcode::JumpAbsolute(self.code.instructions.len());
                } else {
                    // Backpatch jump_if_false to jump past the if block
                    self.code.instructions[jump_if_false_idx] =
                        Opcode::PopJumpIfFalse(self.code.instructions.len());
                }
            }
            Stmt::While { test, body } => {
                let loop_start = self.code.instructions.len();

                self.compile_expr(test)?;
                let jump_if_false_idx = self.emit(Opcode::PopJumpIfFalse(0)); // Placeholder

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::JumpAbsolute(loop_start));

                // Backpatch exit jump
                self.code.instructions[jump_if_false_idx] =
                    Opcode::PopJumpIfFalse(self.code.instructions.len());
            }
            Stmt::Pass => {
                // Do nothing
            }
            Stmt::Return { value } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    // If no value, we should load None, but we lack a PyNone type right now.
                    // For now, load a dummy int or just emit ReturnValue assuming it handles empty stack.
                    // We'll fix this when PyNone is implemented.
                    // Temporary hack: load a 0.
                    let idx = self.add_constant(Rc::new(PyInt::new(0)));
                    self.emit(Opcode::LoadConst(idx));
                }
                self.emit(Opcode::ReturnValue);
            }
            Stmt::FunctionDef { .. } => {
                return Err("Function compilation not yet supported in this phase".to_string());
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLiteral(val) => {
                let obj = Rc::new(PyInt::new(*val));
                let idx = self.add_constant(obj);
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::Identifier(name) => {
                let idx = self.get_or_add_name(name);
                self.emit(Opcode::LoadName(idx));
            }
            Expr::BinOp { left, op, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                let opcode = match op {
                    BinOpKind::Add => Opcode::BinaryAdd,
                    BinOpKind::Sub => Opcode::BinarySubtract,
                    BinOpKind::Mult => Opcode::BinaryMultiply,
                    BinOpKind::Div => Opcode::BinaryTrueDivide,
                    BinOpKind::FloorDiv => Opcode::BinaryFloorDivide,
                    BinOpKind::Mod => Opcode::BinaryModulo,
                    BinOpKind::Pow => Opcode::BinaryPower,
                    BinOpKind::Eq => Opcode::CompareEq,
                    BinOpKind::NotEq => Opcode::CompareNotEq,
                    BinOpKind::Lt => Opcode::CompareLt,
                    BinOpKind::LtEq => Opcode::CompareLtEq,
                    BinOpKind::Gt => Opcode::CompareGt,
                    BinOpKind::GtEq => Opcode::CompareGtEq,
                };

                self.emit(opcode);
            }
            Expr::BooleanLiteral(_) => {
                return Err("Boolean literals not yet implemented in compiler".to_string());
            }
            Expr::StringLiteral(_) => {
                return Err("String literals not yet implemented in compiler".to_string());
            }
            Expr::FloatLiteral(_) => {
                return Err("Float literals not yet implemented in compiler".to_string());
            }
            Expr::NoneLiteral => {
                return Err("None literal not yet implemented in compiler".to_string());
            }
            Expr::UnaryOp { .. } => {
                return Err("Unary ops not yet implemented in compiler".to_string());
            }
            Expr::Call { .. } => {
                return Err("Function calls not yet implemented in compiler".to_string());
            }
        }
        Ok(())
    }
}
