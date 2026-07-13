pub mod code;
pub mod opcodes;

use crate::ast::{BinOpKind, Expr, Module, Stmt, UnaryOpKind};
use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::{PyObject, bool::PyBool, int::PyInt, none::PyNone, string::PyString};
use std::rc::Rc;

pub struct Compiler {
    code: CodeObject,
}

impl Compiler {
    pub fn new(name: String) -> Self {
        Self {
            code: CodeObject::new(name),
        }
    }

    pub fn compile(mut self, module: &Module) -> Result<CodeObject, String> {
        for stmt in &module.body {
            self.compile_stmt(stmt)?;
        }

        let idx = self.add_constant(Rc::new(PyNone::new()));
        self.emit(Opcode::LoadConst(idx));
        self.emit(Opcode::ReturnValue);

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

                if targets.len() != 1 {
                    return Err("Multiple assignment targets not yet supported".to_string());
                }

                match &targets[0] {
                    Expr::Identifier(name) => {
                        let name_idx = self.get_or_add_name(name);
                        self.emit(Opcode::StoreName(name_idx));
                    }
                    Expr::Attribute {
                        value: target_value,
                        attr,
                    } => {
                        self.compile_expr(target_value)?;
                        self.emit(Opcode::StoreAttr(attr.clone()));
                    }
                    _ => {
                        return Err("CompilerError: Unsupported assignment target".to_string());
                    }
                }
            }
            Stmt::ExprStmt { value } => {
                self.compile_expr(value)?;
                self.emit(Opcode::PopTop);
            }
            Stmt::If { test, body, orelse } => {
                self.compile_expr(test)?;
                let jump_if_false_idx = self.emit(Opcode::PopJumpIfFalse(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                if !orelse.is_empty() {
                    let jump_forward_idx = self.emit(Opcode::JumpAbsolute(0));
                    self.code.instructions[jump_if_false_idx] =
                        Opcode::PopJumpIfFalse(self.code.instructions.len());

                    for s in orelse {
                        self.compile_stmt(s)?;
                    }

                    self.code.instructions[jump_forward_idx] =
                        Opcode::JumpAbsolute(self.code.instructions.len());
                } else {
                    self.code.instructions[jump_if_false_idx] =
                        Opcode::PopJumpIfFalse(self.code.instructions.len());
                }
            }
            Stmt::While { test, body } => {
                let loop_start = self.code.instructions.len();

                self.compile_expr(test)?;
                let jump_if_false_idx = self.emit(Opcode::PopJumpIfFalse(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::JumpAbsolute(loop_start));
                self.code.instructions[jump_if_false_idx] =
                    Opcode::PopJumpIfFalse(self.code.instructions.len());
            }
            Stmt::For { target, iter, body } => {
                self.compile_expr(iter)?;
                self.emit(Opcode::GetIter);
                let loop_start = self.code.instructions.len();
                let for_iter_idx = self.emit(Opcode::ForIter(0));
                if let Expr::Identifier(name) = target {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::StoreName(name_idx));
                } else {
                    return Err(format!(
                        "CompilerError: Expected identifier for loop target, got {:?}",
                        target
                    ));
                }
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.emit(Opcode::JumpAbsolute(loop_start));
                let loop_end = self.code.instructions.len();
                self.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);
            }
            Stmt::ClassDef { name, body } => {
                // To build a class, we evaluate its body, which defines its methods.
                // We'll push the name, then evaluate the body to build a dict, then BuildClass.
                // Since our VM doesn't have a concept of class body execution namespace directly yet,
                // we will fake it by compiling the body inside a new compiler, getting a CodeObject,
                // creating a function, calling it to get a dict?
                // No, that's complex. Let's do it inline:
                // We will just compile the methods and store them into a map.
                // Actually, the simplest way is to create a map, then for each method, compile it,
                let mut class_compiler = Compiler::new(name.clone());
                for s in body {
                    class_compiler.compile_stmt(s)?;
                }

                let none_obj = std::rc::Rc::new(crate::objects::none::PyNone);
                class_compiler.code.constants.push(none_obj);
                let none_idx = class_compiler.code.constants.len() - 1;
                class_compiler.emit(Opcode::LoadConst(none_idx));
                class_compiler.emit(Opcode::ReturnValue);

                let code_obj = class_compiler.code;

                let name_obj = std::rc::Rc::new(crate::objects::string::PyString {
                    value: name.clone(),
                });
                self.code.constants.push(name_obj);
                let name_idx = self.code.constants.len() - 1;
                self.emit(Opcode::LoadConst(name_idx));

                let code_idx = self.code.constants.len();
                self.code.constants.push(std::rc::Rc::new(code_obj));
                self.emit(Opcode::LoadConst(code_idx));

                self.emit(Opcode::BuildClass);

                let name_idx = self.get_or_add_name(&name);
                self.emit(Opcode::StoreName(name_idx));
            }
            Stmt::Try { body, handlers } => {
                let setup_except_idx = self.emit(Opcode::SetupExcept(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::PopExcept);

                let jump_forward_idx = self.emit(Opcode::JumpAbsolute(0));

                let except_target = self.code.instructions.len();
                self.code.instructions[setup_except_idx] = Opcode::SetupExcept(except_target);

                // Currently assuming one global handler
                if let Some((_, handler_body)) = handlers.first() {
                    // Python pushes the exception object onto the stack for the except block
                    // Since we aren't binding it yet, we just pop it to clean the stack
                    self.emit(Opcode::PopTop);
                    for s in handler_body {
                        self.compile_stmt(s)?;
                    }
                }

                self.code.instructions[jump_forward_idx] =
                    Opcode::JumpAbsolute(self.code.instructions.len());
            }
            Stmt::Raise { exc } => {
                self.compile_expr(exc)?;
                self.emit(Opcode::Raise);
            }
            Stmt::Pass => {}
            Stmt::Return { value } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    let idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(idx));
                }
                self.emit(Opcode::ReturnValue);
            }
            Stmt::FunctionDef { name, params, body } => {
                let mut child_compiler = Compiler::new(name.clone());
                // Add parameter names to the child code object's names list implicitly
                // The VM will bind arguments to these names when creating the frame
                for param in params {
                    child_compiler.get_or_add_name(param);
                }

                for s in body {
                    child_compiler.compile_stmt(s)?;
                }

                // Ensure the function returns None if it doesn't have an explicit return
                let none_idx = child_compiler.add_constant(Rc::new(PyNone::new()));
                child_compiler.emit(Opcode::LoadConst(none_idx));
                child_compiler.emit(Opcode::ReturnValue);

                let code_obj = Rc::new(child_compiler.code);
                let code_idx = self.add_constant(code_obj);

                self.emit(Opcode::LoadConst(code_idx));
                self.emit(Opcode::MakeFunction);

                let name_idx = self.get_or_add_name(name);
                self.emit(Opcode::StoreName(name_idx));
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLiteral(val) => {
                let idx = self.add_constant(Rc::new(PyInt::new(*val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::FloatLiteral(_) => {
                return Err("Float literals not yet implemented in compiler".to_string());
            }
            Expr::StringLiteral(val) => {
                let idx = self.add_constant(Rc::new(PyString::new(val.clone())));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::BooleanLiteral(val) => {
                let idx = self.add_constant(Rc::new(PyBool::new(*val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::NoneLiteral => {
                let idx = self.add_constant(Rc::new(PyNone::new()));
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
            Expr::UnaryOp { .. } => {
                return Err("Unary ops not yet implemented in compiler".to_string());
            }
            Expr::Call { func, args } => {
                // Compile function to call
                self.compile_expr(func)?;

                // Compile arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }

                self.emit(Opcode::CallFunction(args.len()));
            }
            Expr::List(elements) => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.emit(Opcode::BuildList(elements.len()));
            }
            Expr::Dict(pairs) => {
                for (key, value) in pairs {
                    // Python dict literals evaluate value first, then key? Or key then value?
                    // Actually usually key then value is evaluated. CPython evaluates key then value, but stacks them value, key.
                    // Wait, let's just evaluate key, then value, and VM pops value, then key.
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.emit(Opcode::BuildMap(pairs.len()));
            }
            Expr::Subscript { value, slice } => {
                self.compile_expr(value)?;
                self.compile_expr(slice)?;
                self.emit(Opcode::BinarySubscript);
            }
            Expr::Attribute { value, attr } => {
                self.compile_expr(value)?;
                self.emit(Opcode::LoadAttr(attr.clone()));
            }
        }
        Ok(())
    }
}
