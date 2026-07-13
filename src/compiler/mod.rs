pub mod code;
pub mod opcodes;

use crate::ast::{BinOpKind, Expr, Module, Stmt};
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
            Stmt::YieldStmt(expr) => {
                self.compile_expr(expr)?;
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
            Stmt::ClassDef { name, bases, body, decorators } => {
                // Compile decorators first so they are on the stack
                for decorator in decorators {
                    self.compile_expr(decorator)?;
                }

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

                // Compile the base classes
                for base in bases {
                    self.compile_expr(base)?;
                }

                self.emit(Opcode::BuildClass(bases.len()));

                // Apply decorators
                for _ in 0..decorators.len() {
                    self.emit(Opcode::CallFunction(1));
                }

                let name_idx = self.get_or_add_name(name);
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
            Stmt::With {
                context_expr,
                optional_vars,
                body,
            } => {
                self.compile_expr(context_expr)?;
                let setup_with_idx = self.emit(Opcode::SetupWith(0));

                if let Some(var) = optional_vars {
                    if let Expr::Identifier(name) = var {
                        let name_idx = self.get_or_add_name(&name);
                        self.emit(Opcode::StoreName(name_idx));
                    } else {
                        self.emit(Opcode::PopTop);
                    }
                } else {
                    self.emit(Opcode::PopTop);
                }

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::WithCleanup);

                let except_target = self.code.instructions.len();
                self.code.instructions[setup_with_idx] = Opcode::SetupWith(except_target);
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
            Stmt::FunctionDef { name, params, vararg, kwarg, body, decorators } => {
                // Compile decorators first so they are on the stack
                for decorator in decorators {
                    self.compile_expr(decorator)?;
                }

                let mut child_compiler = Compiler::new(name.clone());
                child_compiler.code.arg_count = params.len();
                child_compiler.code.vararg = vararg.clone();
                child_compiler.code.kwarg = kwarg.clone();

                // Add parameter names to the child code object's names list implicitly
                for param in params {
                    child_compiler.get_or_add_name(param);
                }
                if let Some(v) = vararg {
                    child_compiler.get_or_add_name(v);
                }
                if let Some(k) = kwarg {
                    child_compiler.get_or_add_name(k);
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

                // Apply decorators
                for _ in 0..decorators.len() {
                    self.emit(Opcode::CallFunction(1));
                }

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
            Expr::FloatLiteral(val) => {
                let idx = self.add_constant(Rc::new(crate::objects::float::PyFloat::new(*val)));
                self.emit(Opcode::LoadConst(idx));
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
            Expr::Yield(value_opt) => {
                self.code.is_generator = true;
                if let Some(val) = value_opt {
                    self.compile_expr(val)?;
                } else {
                    let idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(idx));
                }
                self.emit(Opcode::YieldValue);
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
            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand)?;
                let opcode = match op {
                    crate::ast::UnaryOpKind::Minus => Opcode::UnaryNegative,
                    crate::ast::UnaryOpKind::Plus => Opcode::UnaryPositive,
                    crate::ast::UnaryOpKind::Not => Opcode::UnaryNot,
                };
                self.emit(opcode);
            }
            Expr::Call { func, args, kwargs, starargs, kwargs_unpack } => {
                // Compile function to call
                self.compile_expr(func)?;

                if kwargs.is_empty() && starargs.is_empty() && kwargs_unpack.is_empty() {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.emit(Opcode::CallFunction(args.len()));
                } else {
                    // 1. Build the *args list
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.emit(Opcode::BuildList(args.len()));
                    
                    for stararg in starargs {
                        self.compile_expr(stararg)?;
                        self.emit(Opcode::ListExtend);
                    }
                    
                    let has_kwargs = !kwargs.is_empty() || !kwargs_unpack.is_empty();
                    if has_kwargs {
                        // 2. Build the **kwargs dict
                        for (key, value) in kwargs {
                            let idx = self.add_constant(std::rc::Rc::new(crate::objects::string::PyString::new(key.clone())));
                            self.emit(Opcode::LoadConst(idx));
                            self.compile_expr(value)?;
                        }
                        self.emit(Opcode::BuildMap(kwargs.len()));
                        
                        for kwarg_up in kwargs_unpack {
                            self.compile_expr(kwarg_up)?;
                            self.emit(Opcode::DictMerge);
                        }
                        self.emit(Opcode::CallFunctionEx(1));
                    } else {
                        self.emit(Opcode::CallFunctionEx(0));
                    }
                }
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
