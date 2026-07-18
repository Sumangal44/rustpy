pub mod code;
pub mod opcodes;

use crate::ast::{BinOpKind, CompKind, Expr, FStringSegment, Module, Pattern, Stmt, UnaryOpKind};
use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::{bool::PyBool, int::PyInt, none::PyNone, string::PyString, PyObject};
use std::rc::Rc;

struct LoopInfo {
    start: usize,
    break_targets: Vec<usize>,
    is_for: bool,
}

pub struct Compiler {
    code: CodeObject,
    loop_stack: Vec<LoopInfo>,
    global_names: Vec<String>,
    filename: String,
}

fn parse_bigint(val: &str) -> Option<num_bigint::BigInt> {
    if val.starts_with("0x") || val.starts_with("0X") {
        num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 16)
    } else if val.starts_with("0o") || val.starts_with("0O") {
        num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 8)
    } else if val.starts_with("0b") || val.starts_with("0B") {
        num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 2)
    } else {
        val.parse().ok()
    }
}

impl Compiler {
    pub fn new(filename: String) -> Self {
        let mut code = CodeObject::new(filename.clone());
        code.filename = filename.clone();
        Self {
            code,
            loop_stack: Vec::new(),
            global_names: Vec::new(),
            filename,
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

    pub fn compile_expression(mut self, expr: &Expr) -> Result<CodeObject, String> {
        self.compile_expr(expr)?;
        self.emit(Opcode::ReturnValue);
        Ok(self.code)
    }

    fn emit(&mut self, opcode: Opcode) -> usize {
        let pos = self.code.instructions.len();
        self.code.instructions.push(opcode);
        pos
    }

    fn add_constant(&mut self, obj: Rc<dyn PyObject>) -> usize {
        for (i, c) in self.code.constants.iter().enumerate() {
            if c.get_type() == obj.get_type() {
                if let Some(eq_res) = c.eq(Rc::clone(&obj)) {
                    if eq_res.is_truthy() {
                        return i;
                    }
                }
            }
        }
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

                let star_count = targets
                    .iter()
                    .filter(|t| matches!(t, Expr::Starred { .. }))
                    .count();

                if star_count == 0 && targets.len() == 1 {
                    match &targets[0] {
                        Expr::Identifier(name) => {
                            let name_idx = self.get_or_add_name(name);
                            if self.global_names.contains(name) {
                                self.emit(Opcode::StoreGlobal(name_idx));
                            } else {
                                self.emit(Opcode::StoreName(name_idx));
                            }
                        }
                        Expr::Attribute {
                            value: target_value,
                            attr,
                        } => {
                            self.compile_expr(target_value)?;
                            self.emit(Opcode::StoreAttr(attr.clone()));
                        }
                        Expr::Subscript { value, slice } => {
                            self.compile_expr(value)?;
                            self.compile_expr(slice)?;
                            self.emit(Opcode::StoreSubscript);
                        }
                        _ => {
                            return Err("CompilerError: Unsupported assignment target".to_string());
                        }
                    }
                } else if star_count == 0 && targets.len() > 1 {
                    self.emit(Opcode::UnpackSequence(targets.len()));
                    for target in targets.iter() {
                        match target {
                            Expr::Identifier(name) => {
                                let name_idx = self.get_or_add_name(name);
                                if self.global_names.contains(name) {
                                    self.emit(Opcode::StoreGlobal(name_idx));
                                } else {
                                    self.emit(Opcode::StoreName(name_idx));
                                }
                            }
                            Expr::Attribute { value, attr } => {
                                self.compile_expr(value)?;
                                self.emit(Opcode::StoreAttr(attr.clone()));
                            }
                            Expr::Subscript { value, slice } => {
                                self.compile_expr(value)?;
                                self.compile_expr(slice)?;
                                self.emit(Opcode::StoreSubscript);
                            }
                            _ => {
                                return Err("CompilerError: Unsupported unpack target".to_string());
                            }
                        }
                    }
                } else if star_count == 1 {
                    let star_index = targets
                        .iter()
                        .position(|t| matches!(t, Expr::Starred { .. }))
                        .unwrap();
                    let before = star_index;
                    let after = targets.len() - star_index - 1;
                    self.emit(Opcode::UnpackEx(before, after));
                    for target in targets.iter() {
                        match target {
                            Expr::Identifier(name) => {
                                let name_idx = self.get_or_add_name(name);
                                if self.global_names.contains(name) {
                                    self.emit(Opcode::StoreGlobal(name_idx));
                                } else {
                                    self.emit(Opcode::StoreName(name_idx));
                                }
                            }
                            Expr::Attribute { value, attr } => {
                                self.compile_expr(value)?;
                                self.emit(Opcode::StoreAttr(attr.clone()));
                            }
                            Expr::Subscript { value, slice } => {
                                self.compile_expr(value)?;
                                self.compile_expr(slice)?;
                                self.emit(Opcode::StoreSubscript);
                            }
                            Expr::Starred { value } => match value.as_ref() {
                                Expr::Identifier(name) => {
                                    let name_idx = self.get_or_add_name(name);
                                    if self.global_names.contains(name) {
                                        self.emit(Opcode::StoreGlobal(name_idx));
                                    } else {
                                        self.emit(Opcode::StoreName(name_idx));
                                    }
                                }
                                Expr::Attribute { value, attr } => {
                                    self.compile_expr(value)?;
                                    self.emit(Opcode::StoreAttr(attr.clone()));
                                }
                                Expr::Subscript { value, slice } => {
                                    self.compile_expr(value)?;
                                    self.compile_expr(slice)?;
                                    self.emit(Opcode::StoreSubscript);
                                }
                                _ => {
                                    return Err(
                                        "CompilerError: Unsupported starred target".to_string()
                                    );
                                }
                            },
                            _ => {
                                return Err(
                                    "CompilerError: Unsupported assignment target".to_string()
                                );
                            }
                        }
                    }
                } else {
                    return Err("CompilerError: Multiple starred targets not supported".to_string());
                }
            }
            Stmt::AugAssign { target, op, value } => {
                match target.as_ref() {
                    Expr::Identifier(name) => {
                        let name_idx = self.get_or_add_name(name);
                        self.emit(Opcode::LoadName(name_idx));
                        self.compile_expr(value)?;
                        self.emit_binop(op.clone())?;
                        self.emit(Opcode::StoreName(name_idx));
                    }
                    Expr::Attribute {
                        value: target_val,
                        attr,
                    } => {
                        self.compile_expr(target_val)?;
                        // Dup the object by compiling twice (simplified)
                        self.compile_expr(target_val)?;
                        self.emit(Opcode::LoadAttr(attr.clone()));
                        self.compile_expr(value)?;
                        self.emit_binop(op.clone())?;
                        self.emit(Opcode::StoreAttr(attr.clone()));
                    }
                    _ => {
                        return Err(
                            "CompilerError: Unsupported augmented assignment target".to_string()
                        );
                    }
                }
            }
            Stmt::AnnAssign {
                target,
                annotation: _,
                value,
            } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                    match target.as_ref() {
                        Expr::Identifier(name) => {
                            let name_idx = self.get_or_add_name(name);
                            if self.global_names.contains(name) {
                                self.emit(Opcode::StoreGlobal(name_idx));
                            } else {
                                self.emit(Opcode::StoreName(name_idx));
                            }
                        }
                        _ => {
                            return Err("CompilerError: Unsupported AnnAssign target".to_string());
                        }
                    }
                }
            }
            Stmt::Break => {
                if self.loop_stack.is_empty() {
                    return Err("SyntaxError: 'break' outside loop".to_string());
                }
                let idx_in_stack = self.loop_stack.len() - 1;
                if self.loop_stack[idx_in_stack].is_for {
                    self.emit(Opcode::PopTop);
                }
                let break_idx = self.emit(Opcode::JumpAbsolute(0));
                self.loop_stack[idx_in_stack].break_targets.push(break_idx);
            }
            Stmt::Continue => {
                let start = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| "SyntaxError: 'continue' outside loop".to_string())?
                    .start;
                self.emit(Opcode::JumpAbsolute(start));
            }
            Stmt::Del { targets } => {
                for target in targets {
                    match target {
                        Expr::Identifier(name) => {
                            let name_idx = self.get_or_add_name(&name);
                            self.emit(Opcode::DeleteName(name_idx));
                        }
                        Expr::Attribute { value: obj, attr } => {
                            self.compile_expr(obj)?;
                            self.emit(Opcode::DeleteAttr(attr.clone()));
                        }
                        Expr::Subscript { value, slice } => {
                            self.compile_expr(value)?;
                            self.compile_expr(slice)?;
                            self.emit(Opcode::DeleteSubscript);
                        }
                        _ => {
                            return Err("CompilerError: Unsupported del target".to_string());
                        }
                    }
                }
            }
            Stmt::Global { names } => {
                for name in names {
                    self.get_or_add_name(name);
                    if !self.global_names.contains(name) {
                        self.global_names.push(name.clone());
                    }
                }
            }
            Stmt::Nonlocal { names } => {
                for name in names {
                    self.get_or_add_name(name);
                    self.code.nonlocal_names.push(name.clone());
                }
            }
            Stmt::Match { subject, cases } => {
                let subj_name = "__match_subj";
                self.compile_expr(subject)?;
                let subj_idx = self.get_or_add_name(subj_name);
                self.emit(Opcode::StoreName(subj_idx));

                let mut next_case_indices = Vec::new();

                for (case_idx, case) in cases.iter().enumerate() {
                    self.emit(Opcode::LoadName(subj_idx));
                    self.compile_pattern(&case.pattern)?;

                    let jump_false_idx = self.emit(Opcode::PopJumpIfFalse(0));

                    let mut guard_false_idx = None;
                    if let Some(guard) = &case.guard {
                        self.compile_expr(guard)?;
                        guard_false_idx = Some(self.emit(Opcode::PopJumpIfFalse(0)));
                    }

                    for s in &case.body {
                        self.compile_stmt(s)?;
                    }

                    if case_idx < cases.len() - 1 {
                        next_case_indices.push(self.emit(Opcode::JumpAbsolute(0)));
                    }

                    let next_case_pos = self.code.instructions.len();
                    self.code.instructions[jump_false_idx] = Opcode::PopJumpIfFalse(next_case_pos);
                    if let Some(idx) = guard_false_idx {
                        self.code.instructions[idx] = Opcode::PopJumpIfFalse(next_case_pos);
                    }
                }

                let end_pos = self.code.instructions.len();
                for idx in next_case_indices {
                    self.code.instructions[idx] = Opcode::JumpAbsolute(end_pos);
                }
            }
            Stmt::Assert { test, msg } => {
                self.compile_expr(test)?;
                let jump_if_true_idx = self.emit(Opcode::PopJumpIfTrue(0));

                let exc_name_idx = self.get_or_add_name("AssertionError");
                self.emit(Opcode::LoadName(exc_name_idx));

                if let Some(msg_expr) = msg {
                    self.compile_expr(msg_expr)?;
                    self.emit(Opcode::CallFunction(1));
                } else {
                    self.emit(Opcode::CallFunction(0));
                }

                self.emit(Opcode::Raise);

                self.code.instructions[jump_if_true_idx] =
                    Opcode::PopJumpIfTrue(self.code.instructions.len());
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
            Stmt::While { test, body, orelse } => {
                let loop_start = self.code.instructions.len();
                self.loop_stack.push(LoopInfo {
                    start: loop_start,
                    break_targets: Vec::new(),
                    is_for: false,
                });

                self.compile_expr(test)?;
                let jump_if_false_idx = self.emit(Opcode::PopJumpIfFalse(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::JumpAbsolute(loop_start));

                let condition_false_target = self.code.instructions.len();
                self.code.instructions[jump_if_false_idx] =
                    Opcode::PopJumpIfFalse(condition_false_target);

                let info = self.loop_stack.pop().unwrap();

                for s in orelse {
                    self.compile_stmt(s)?;
                }

                let after_orelse = self.code.instructions.len();
                for idx in &info.break_targets {
                    self.code.instructions[*idx] = Opcode::JumpAbsolute(after_orelse);
                }
            }
            Stmt::For {
                target,
                iter,
                body,
                orelse,
                is_async,
            } if *is_async => {
                let aiter_name = format!("__aiter_{}", self.code.names.len());
                let aiter_name = self.get_or_add_name(&aiter_name);
                self.compile_expr(iter)?;
                self.emit(Opcode::LoadAttr("__aiter__".to_string()));
                self.emit(Opcode::CallFunction(0));
                self.emit(Opcode::StoreName(aiter_name));

                let loop_start = self.code.instructions.len();
                self.loop_stack.push(LoopInfo {
                    start: loop_start,
                    break_targets: Vec::new(),
                    is_for: false,
                });

                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx));
                let jump_out = self.emit(Opcode::PopJumpIfFalse(0));

                let except_setup = self.emit(Opcode::SetupExcept(0));

                self.emit(Opcode::LoadName(aiter_name));
                self.emit(Opcode::LoadAttr("__anext__".to_string()));
                self.emit(Opcode::CallFunction(0));
                self.emit(Opcode::GetAwaitable);
                let none_idx = self.add_constant(Rc::new(PyNone::new()));
                self.emit(Opcode::LoadConst(none_idx));
                self.emit(Opcode::YieldFrom);

                if let Expr::Identifier(name) = target {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::StoreName(name_idx));
                } else if let Expr::Tuple(elements) = target {
                    self.emit(Opcode::UnpackSequence(elements.len()));
                    for el in elements.iter() {
                        if let Expr::Identifier(name) = el {
                            let name_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(name_idx));
                        } else {
                            return Err(
                                "CompilerError: Async for tuple target elements must be identifiers"
                                    .to_string(),
                            );
                        }
                    }
                } else {
                    return Err(format!(
                        "CompilerError: Expected identifier or tuple for async for target, got {:?}",
                        target
                    ));
                }

                self.emit(Opcode::PopExcept);

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit(Opcode::JumpAbsolute(loop_start));

                let handler_start = self.code.instructions.len();
                self.code.instructions[except_setup] = Opcode::SetupExcept(handler_start);

                self.emit(Opcode::DupTop);
                self.emit(Opcode::ExceptionMatch("StopAsyncIteration".to_string()));
                let not_stop = self.emit(Opcode::PopJumpIfFalse(0));

                // StopAsyncIteration: normal loop exit, run orelse
                self.emit(Opcode::PopTop);
                let normal_exit = self.emit(Opcode::JumpAbsolute(0));

                let re_raise_target = self.code.instructions.len();
                self.code.instructions[not_stop] = Opcode::PopJumpIfFalse(re_raise_target);
                self.emit(Opcode::Raise);

                let info = self.loop_stack.pop().unwrap();
                let after_orelse = self.code.instructions.len();
                self.code.instructions[jump_out] = Opcode::PopJumpIfFalse(after_orelse);
                self.code.instructions[normal_exit] = Opcode::JumpAbsolute(after_orelse);

                for s in orelse {
                    self.compile_stmt(s)?;
                }

                let after_else = self.code.instructions.len();
                for idx in &info.break_targets {
                    self.code.instructions[*idx] = Opcode::JumpAbsolute(after_else);
                }
            }
            Stmt::For {
                target,
                iter,
                body,
                orelse,
                is_async: _,
            } => {
                self.compile_expr(iter)?;
                self.emit(Opcode::GetIter);
                let loop_start = self.code.instructions.len();
                self.loop_stack.push(LoopInfo {
                    start: loop_start,
                    break_targets: Vec::new(),
                    is_for: true,
                });
                let for_iter_idx = self.emit(Opcode::ForIter(0));
                if let Expr::Identifier(name) = target {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::StoreName(name_idx));
                } else if let Expr::Tuple(elements) = target {
                    self.emit(Opcode::UnpackSequence(elements.len()));
                    for el in elements.iter() {
                        if let Expr::Identifier(name) = el {
                            let name_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(name_idx));
                        } else {
                            return Err(
                                "CompilerError: For loop tuple target elements must be identifiers"
                                    .to_string(),
                            );
                        }
                    }
                } else {
                    return Err(format!(
                        "CompilerError: Expected identifier or tuple for loop target, got {:?}",
                        target
                    ));
                }
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.emit(Opcode::JumpAbsolute(loop_start));
                let loop_end = self.code.instructions.len();
                self.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                let info = self.loop_stack.pop().unwrap();

                for s in orelse {
                    self.compile_stmt(s)?;
                }

                let after_orelse = self.code.instructions.len();
                for idx in &info.break_targets {
                    self.code.instructions[*idx] = Opcode::JumpAbsolute(after_orelse);
                }
            }
            Stmt::ClassDef {
                name,
                bases,
                keywords,
                body,
                decorators,
            } => {
                // Compile decorators first so they are on the stack
                for decorator in decorators {
                    self.compile_expr(decorator)?;
                }

                let mut class_compiler = Compiler::new(name.clone());
                class_compiler.code.filename = self.filename.clone();
                class_compiler.filename = self.filename.clone();
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

                // Compile keyword arguments (metaclass=Meta, etc.)
                for (key, value) in keywords {
                    // Push key name as string constant
                    let key_obj =
                        std::rc::Rc::new(crate::objects::string::PyString { value: key.clone() });
                    self.code.constants.push(key_obj);
                    let key_idx = self.code.constants.len() - 1;
                    self.emit(Opcode::LoadConst(key_idx));
                    self.compile_expr(value)?;
                }

                self.emit(Opcode::BuildClass {
                    bases: bases.len(),
                    keywords: keywords.len(),
                });

                // Apply decorators
                for _ in 0..decorators.len() {
                    self.emit(Opcode::CallFunction(1));
                }

                let name_idx = self.get_or_add_name(name);
                self.emit(Opcode::StoreName(name_idx));
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                let has_handlers = !handlers.is_empty();
                let has_else = else_body.is_some();
                let has_finally = finally_body.is_some();

                let mut finally_jumps: Vec<usize> = Vec::new();
                let mut end_jumps: Vec<usize> = Vec::new();
                let mut else_jumps: Vec<usize> = Vec::new();

                let setup_finally_idx = if has_finally {
                    Some(self.emit(Opcode::SetupFinally(0)))
                } else {
                    None
                };

                let setup_except_idx = if has_handlers {
                    Some(self.emit(Opcode::SetupExcept(0)))
                } else {
                    None
                };

                for s in body {
                    self.compile_stmt(s)?;
                }

                match (has_handlers, has_else, has_finally) {
                    (true, true, true) => {
                        self.emit(Opcode::PopExcept);
                        self.emit(Opcode::PopFinally);
                        else_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (true, true, false) => {
                        self.emit(Opcode::PopExcept);
                        else_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (true, false, true) => {
                        self.emit(Opcode::PopExcept);
                        self.emit(Opcode::PopFinally);
                        finally_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (true, false, false) => {
                        self.emit(Opcode::PopExcept);
                        end_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (false, true, true) => {
                        self.emit(Opcode::PopFinally);
                        else_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (false, true, false) => {}
                    (false, false, true) => {
                        self.emit(Opcode::PopFinally);
                        finally_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                    (false, false, false) => {}
                }

                if let Some(except_idx) = setup_except_idx {
                    let handler_start = self.code.instructions.len();
                    self.code.instructions[except_idx] = Opcode::SetupExcept(handler_start);

                    let mut handler_end_jumps = Vec::new();

                    for (exc_type_opt, as_name_opt, handler_body) in handlers.iter() {
                        if let Some(exc_type) = exc_type_opt {
                            self.emit(Opcode::DupTop);
                            self.emit(Opcode::ExceptionMatch(exc_type.clone()));
                            let type_mismatch_jump = self.emit(Opcode::PopJumpIfFalse(0));
                            // The original exception is on stack for binding
                            if let Some(as_name) = as_name_opt {
                                let as_idx = self.get_or_add_name(as_name);
                                self.emit(Opcode::StoreName(as_idx));
                            } else {
                                self.emit(Opcode::PopTop);
                            }
                            for s in handler_body {
                                self.compile_stmt(s)?;
                            }
                            handler_end_jumps.push(self.emit(Opcode::JumpAbsolute(0)));

                            let next_handler = self.code.instructions.len();
                            self.code.instructions[type_mismatch_jump] =
                                Opcode::PopJumpIfFalse(next_handler);
                        } else {
                            // Bare except - always matches
                            if let Some(as_name) = as_name_opt {
                                let as_idx = self.get_or_add_name(as_name);
                                self.emit(Opcode::StoreName(as_idx));
                            } else {
                                self.emit(Opcode::PopTop);
                            }
                            for s in handler_body {
                                self.compile_stmt(s)?;
                            }
                            handler_end_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                        }
                    }

                    // If no handler matched, re-raise the exception
                    let _after_all_handlers = self.code.instructions.len();
                    self.emit(Opcode::Raise);

                    let end_of_try = self.code.instructions.len();
                    for jmp in &handler_end_jumps {
                        self.code.instructions[*jmp] = Opcode::JumpAbsolute(end_of_try);
                    }

                    if has_finally {
                        self.emit(Opcode::PopFinally);
                        finally_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    } else {
                        end_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                }

                if let Some(else_body) = else_body {
                    let else_start = self.code.instructions.len();
                    for jmp_idx in &else_jumps {
                        self.code.instructions[*jmp_idx] = Opcode::JumpAbsolute(else_start);
                    }

                    for s in else_body {
                        self.compile_stmt(s)?;
                    }

                    if has_finally {
                        self.emit(Opcode::PopFinally);
                        finally_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    } else {
                        end_jumps.push(self.emit(Opcode::JumpAbsolute(0)));
                    }
                }

                if let Some(finally_idx) = setup_finally_idx {
                    let finally_start = self.code.instructions.len();
                    self.code.instructions[finally_idx] = Opcode::SetupFinally(finally_start);

                    for jmp_idx in &finally_jumps {
                        self.code.instructions[*jmp_idx] = Opcode::JumpAbsolute(finally_start);
                    }

                    if let Some(finally_body) = finally_body {
                        for s in finally_body {
                            self.compile_stmt(s)?;
                        }
                    }

                    self.emit(Opcode::EndFinally);
                }

                let end = self.code.instructions.len();
                for jmp_idx in &end_jumps {
                    self.code.instructions[*jmp_idx] = Opcode::JumpAbsolute(end);
                }
            }
            Stmt::With {
                items,
                body,
                is_async: _,
            } => {
                let mut setup_indices = Vec::new();
                for item in items.iter() {
                    self.compile_expr(&item.context_expr)?;
                    let setup_idx = self.emit(Opcode::SetupWith(0));
                    setup_indices.push(setup_idx);
                    if let Some(var) = &item.optional_vars {
                        if let Expr::Identifier(name) = var {
                            let name_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(name_idx));
                        } else {
                            self.emit(Opcode::PopTop);
                        }
                    } else {
                        self.emit(Opcode::PopTop);
                    }
                }

                for s in body {
                    self.compile_stmt(s)?;
                }

                for _ in items.iter() {
                    self.emit(Opcode::WithCleanup);
                }

                for setup_idx in setup_indices {
                    let except_target = self.code.instructions.len();
                    self.code.instructions[setup_idx] = Opcode::SetupWith(except_target);
                }
            }
            Stmt::Raise { exc, cause } => match (exc, cause) {
                (Some(exc_expr), Some(cause_expr)) => {
                    self.compile_expr(cause_expr)?;
                    self.compile_expr(exc_expr)?;
                    self.emit(Opcode::Raise);
                }
                (Some(exc_expr), None) => {
                    let none_idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(none_idx));
                    self.compile_expr(exc_expr)?;
                    self.emit(Opcode::Raise);
                }
                (None, _) => {
                    self.emit(Opcode::Raise);
                }
            },
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
            Stmt::FunctionDef {
                name,
                posonly_params,
                params,
                kwonly_params,
                defaults,
                vararg,
                kwarg,
                body,
                decorators,
                is_async,
                returns: _,
            } => {
                // Compile decorators first so they are on the stack
                for decorator in decorators {
                    self.compile_expr(decorator)?;
                }

                let mut child_compiler = Compiler::new(name.clone());
                child_compiler.code.filename = self.filename.clone();
                child_compiler.filename = self.filename.clone();
                child_compiler.code.arg_count = posonly_params.len() + params.len();
                child_compiler.code.posonly_count = posonly_params.len();
                child_compiler.code.kwonly_params = kwonly_params.clone();
                child_compiler.code.is_async = *is_async;
                child_compiler.code.vararg = vararg.clone();
                child_compiler.code.kwarg = kwarg.clone();

                // Add parameter names to the child code object's names list implicitly
                for param in posonly_params {
                    child_compiler.get_or_add_name(param);
                }
                for param in params {
                    child_compiler.get_or_add_name(param);
                }
                for param in kwonly_params {
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

                // Compile keyword-only default parameter values
                let pos_count = posonly_params.len() + params.len();
                let kw_defaults: Vec<&Option<Expr>> = defaults.iter().skip(pos_count).collect();
                let kw_defaults_count = kw_defaults.iter().filter(|d| d.is_some()).count();
                for d in kw_defaults {
                    if let Some(expr) = d {
                        self.compile_expr(expr)?;
                    }
                }
                self.emit(Opcode::BuildTuple(kw_defaults_count));

                // Compile positional default parameter values (right-aligned)
                let pos_defaults: Vec<&Option<Expr>> = defaults.iter().take(pos_count).collect();
                let pos_defaults_count = pos_defaults.iter().filter(|d| d.is_some()).count();
                for d in pos_defaults {
                    if let Some(expr) = d {
                        self.compile_expr(expr)?;
                    }
                }
                self.emit(Opcode::BuildTuple(pos_defaults_count));

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
            Stmt::Import { names } => {
                for alias in names {
                    let name = &alias.name;

                    // For dotted imports, only the top-level name is bound
                    let top_level = name.split('.').next().unwrap_or(name).to_string();
                    let top_store = alias.asname.as_ref().unwrap_or(&top_level);

                    // Push level=0, fromlist=()
                    let zero_idx =
                        self.add_constant(Rc::new(crate::objects::int::PyInt::from_i64(0)));
                    self.emit(Opcode::LoadConst(zero_idx));
                    let empty_tuple_idx =
                        self.add_constant(Rc::new(crate::objects::tuple::PyTuple::new(vec![])));
                    self.emit(Opcode::LoadConst(empty_tuple_idx));
                    // IMPORT_NAME with the full module name
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::ImportName(name_idx));
                    // Store the top-level module
                    let store_idx = self.get_or_add_name(top_store);
                    self.emit(Opcode::StoreName(store_idx));
                }
            }
            Stmt::ImportFrom {
                module,
                names,
                level,
            } => {
                let mut from_names: Vec<Rc<dyn PyObject>> = Vec::new();
                let is_star = names.len() == 1 && names[0].name == "*";

                if is_star {
                    // fromlist has a sentinel to distinguish star import
                    let zero = Rc::new(crate::objects::int::PyInt::from_i64(0));
                    from_names.push(zero);
                } else {
                    for alias in names {
                        let name_obj =
                            Rc::new(crate::objects::string::PyString::new(alias.name.clone()));
                        from_names.push(name_obj);
                    }
                }

                // Push level, fromlist (level > 0 means relative import)
                let level_idx =
                    self.add_constant(Rc::new(crate::objects::int::PyInt::from_i64(*level as i64)));
                self.emit(Opcode::LoadConst(level_idx));
                let fromlist = Rc::new(crate::objects::tuple::PyTuple::new(from_names));
                let fromlist_idx = self.add_constant(fromlist);
                self.emit(Opcode::LoadConst(fromlist_idx));

                // Import the module
                let mod_idx = self.get_or_add_name(module);
                self.emit(Opcode::ImportName(mod_idx));

                if is_star {
                    self.emit(Opcode::ImportStar);
                } else {
                    for alias in names {
                        let attr_idx = self.get_or_add_name(&alias.name);
                        self.emit(Opcode::ImportFrom(attr_idx));
                        let store_name = alias.asname.as_ref().unwrap_or(&alias.name);
                        let store_idx = self.get_or_add_name(store_name);
                        self.emit(Opcode::StoreName(store_idx));
                    }
                    self.emit(Opcode::PopTop); // pop the module
                }
            }
        }
        Ok(())
    }

    fn compile_pattern(&mut self, pattern: &Pattern) -> Result<(), String> {
        match pattern {
            Pattern::Literal(expr) => {
                // Compare subject (already on stack) with literal
                self.compile_expr(expr)?;
                self.emit(Opcode::CompareEq);
            }
            Pattern::Capture(name) => {
                if name != "_" {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::DupTop);
                    self.emit(Opcode::StoreName(name_idx));
                }
                // Pop subject, pattern always matches: push True
                self.emit(Opcode::PopTop);
                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx));
            }
            Pattern::Or(subpatterns) => {
                let subj_name_idx = self.get_or_add_name("__match_subj");
                let mut match_jumps = Vec::new();
                for (i, sub) in subpatterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(Opcode::LoadName(subj_name_idx));
                    }
                    self.compile_pattern(sub)?;
                    if i < subpatterns.len() - 1 {
                        // On match, push True and jump to end
                        let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                        self.emit(Opcode::LoadConst(true_idx));
                        let jmp = self.emit(Opcode::JumpAbsolute(0));
                        match_jumps.push(jmp);
                        // Pop the False result from compile_pattern, continue
                        self.emit(Opcode::PopTop);
                    }
                }
                let end = self.code.instructions.len();
                for jmp in &match_jumps {
                    self.code.instructions[*jmp] = Opcode::JumpAbsolute(end);
                }
            }
            Pattern::Sequence(subpatterns) => {
                let count = subpatterns.len();
                self.emit(Opcode::DupTop); // dup subj
                self.emit(Opcode::CheckSequence(count));

                let fail_jump = self.emit(Opcode::PopJumpIfFalse(0));

                let mut sub_fails = Vec::new();
                for (i, subpattern) in subpatterns.iter().enumerate() {
                    self.emit(Opcode::DupTop); // dup subj
                    let idx_idx = self.add_constant(Rc::new(crate::objects::int::PyInt::new(
                        num_bigint::BigInt::from(i as i64),
                    )));
                    self.emit(Opcode::LoadConst(idx_idx));
                    self.emit(Opcode::BinarySubscript);

                    self.compile_pattern(subpattern)?;
                    sub_fails.push(self.emit(Opcode::PopJumpIfFalse(0)));
                }

                self.emit(Opcode::PopTop); // pop subj
                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx));
                let success_jump = self.emit(Opcode::JumpAbsolute(0));

                let fail_target = self.code.instructions.len();
                self.code.instructions[fail_jump] = Opcode::PopJumpIfFalse(fail_target);
                for sub_fail in sub_fails {
                    self.code.instructions[sub_fail] = Opcode::PopJumpIfFalse(fail_target);
                }

                self.emit(Opcode::PopTop); // pop subj
                let false_idx = self.add_constant(Rc::new(PyBool::new(false)));
                self.emit(Opcode::LoadConst(false_idx));

                let end_target = self.code.instructions.len();
                self.code.instructions[success_jump] = Opcode::JumpAbsolute(end_target);
            }
            Pattern::Mapping(elements) => {
                self.emit(Opcode::DupTop); // dup subj
                self.emit(Opcode::MatchMapping); // pushes True/False

                let fail_jump = self.emit(Opcode::PopJumpIfFalse(0));

                let mut sub_fails = Vec::new();
                for (key_expr, val_pattern) in elements {
                    self.emit(Opcode::DupTop); // stack: subj, subj
                    self.compile_expr(key_expr)?; // stack: subj, subj, key

                    self.emit(Opcode::DupTop); // stack: subj, subj, key, key
                    self.emit(Opcode::RotThree); // stack: subj, key, subj, key
                    self.emit(Opcode::RotTwo); // stack: subj, key, key, subj
                    self.emit(Opcode::DupTop); // stack: subj, key, key, subj, subj
                    self.emit(Opcode::RotThree); // stack: subj, key, subj, key, subj

                    self.emit(Opcode::CompareIn); // stack: subj, key, subj, is_in
                    let jump_if_in = self.emit(Opcode::PopJumpIfTrue(0)); // stack: subj, key, subj

                    // Fail path for CompareIn
                    self.emit(Opcode::PopTop); // pop subj
                    self.emit(Opcode::PopTop); // pop key
                    self.emit(Opcode::PopTop); // pop subj (leaves the original subj)
                    sub_fails.push(self.emit(Opcode::JumpAbsolute(0))); // jump to fail_target

                    let in_target = self.code.instructions.len();
                    self.code.instructions[jump_if_in] = Opcode::PopJumpIfTrue(in_target);
                    // stack: subj, key, subj

                    self.emit(Opcode::RotTwo); // stack: subj, subj, key
                    self.emit(Opcode::BinarySubscript); // stack: subj, subj[key]

                    self.compile_pattern(val_pattern)?; // stack: subj, is_match
                    let jump_if_match = self.emit(Opcode::PopJumpIfTrue(0)); // stack: subj

                    // Fail path for compile_pattern
                    sub_fails.push(self.emit(Opcode::JumpAbsolute(0))); // jump to fail_target

                    let match_target = self.code.instructions.len();
                    self.code.instructions[jump_if_match] = Opcode::PopJumpIfTrue(match_target);
                    // stack: subj (ready for next iteration)
                }

                self.emit(Opcode::PopTop); // pop subj
                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx));
                let success_jump = self.emit(Opcode::JumpAbsolute(0));

                let fail_target = self.code.instructions.len();
                self.code.instructions[fail_jump] = Opcode::PopJumpIfFalse(fail_target);
                for sub_fail in sub_fails {
                    self.code.instructions[sub_fail] = Opcode::JumpAbsolute(fail_target);
                }

                self.emit(Opcode::PopTop); // pop subj
                let false_idx = self.add_constant(Rc::new(PyBool::new(false)));
                self.emit(Opcode::LoadConst(false_idx));

                let end_target = self.code.instructions.len();
                self.code.instructions[success_jump] = Opcode::JumpAbsolute(end_target);
            }
            Pattern::Class(name, pos, kw) => {
                let name_idx = self.get_or_add_name(name);
                self.emit(Opcode::LoadName(name_idx)); // push class
                self.emit(Opcode::MatchClassCheck); // stack: subj, is_match

                let fail_jump = self.emit(Opcode::PopJumpIfFalse(0)); // stack: subj

                let mut sub_fails = Vec::new();

                // Handle positional patterns via __match_args__
                for (i, subpattern) in pos.iter().enumerate() {
                    self.emit(Opcode::DupTop); // stack: subj, subj
                    self.emit(Opcode::MatchClassGetPos(i)); // stack: subj, attr, has_attr

                    let has_attr_jump = self.emit(Opcode::PopJumpIfFalse(0)); // stack: subj, attr

                    self.compile_pattern(subpattern)?; // stack: subj, is_match
                    sub_fails.push(self.emit(Opcode::PopJumpIfFalse(0))); // stack: subj

                    let success_continue = self.emit(Opcode::JumpAbsolute(0));

                    let attr_target = self.code.instructions.len();
                    self.code.instructions[has_attr_jump] = Opcode::PopJumpIfFalse(attr_target);
                    self.emit(Opcode::PopTop); // pop dummy attr
                    sub_fails.push(self.emit(Opcode::JumpAbsolute(0))); // jump to fail_target

                    let continue_target = self.code.instructions.len();
                    self.code.instructions[success_continue] =
                        Opcode::JumpAbsolute(continue_target);
                }

                // Handle keyword patterns: case Point(x=val_pattern, y=val_pattern)
                // These match obj.attr against the sub-pattern
                for (attr_name, subpattern) in kw {
                    self.emit(Opcode::DupTop); // stack: ..., subj, subj
                    self.emit(Opcode::LoadAttr(attr_name.clone())); // stack: ..., subj, subj.attr_name
                    self.compile_pattern(subpattern)?; // stack: ..., subj, is_match
                    sub_fails.push(self.emit(Opcode::PopJumpIfFalse(0))); // stack: ..., subj
                }

                self.emit(Opcode::PopTop); // pop subj
                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx)); // stack: True
                let success_jump = self.emit(Opcode::JumpAbsolute(0));

                let fail_target = self.code.instructions.len();
                self.code.instructions[fail_jump] = Opcode::PopJumpIfFalse(fail_target);
                for sub_fail in sub_fails {
                    if matches!(self.code.instructions[sub_fail], Opcode::PopJumpIfFalse(_)) {
                        self.code.instructions[sub_fail] = Opcode::PopJumpIfFalse(fail_target);
                    } else {
                        self.code.instructions[sub_fail] = Opcode::JumpAbsolute(fail_target);
                    }
                }

                self.emit(Opcode::PopTop); // pop subj
                let false_idx = self.add_constant(Rc::new(PyBool::new(false)));
                self.emit(Opcode::LoadConst(false_idx)); // stack: False

                let end_target = self.code.instructions.len();
                self.code.instructions[success_jump] = Opcode::JumpAbsolute(end_target);
            }
        }
        Ok(())
    }

    fn emit_binop(&mut self, op: BinOpKind) -> Result<(), String> {
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
            BinOpKind::In => Opcode::CompareIn,
            BinOpKind::NotIn => Opcode::CompareNotIn,
            BinOpKind::Is => Opcode::CompareIs,
            BinOpKind::IsNot => Opcode::CompareIsNot,
            BinOpKind::MatMul => Opcode::BinaryMatMul,
            BinOpKind::BitAnd => Opcode::BinaryBitAnd,
            BinOpKind::BitOr => Opcode::BinaryBitOr,
            BinOpKind::BitXor => Opcode::BinaryBitXor,
            BinOpKind::LShift => Opcode::BinaryLShift,
            BinOpKind::RShift => Opcode::BinaryRShift,
            BinOpKind::And | BinOpKind::Or => {
                return Err("Compiler error: and/or should be handled in compile_expr".to_string());
            }
        };
        self.emit(opcode);
        Ok(())
    }

    fn fold_binop(&self, left: &Expr, op: BinOpKind, right: &Expr) -> Option<Expr> {
        match (left, right) {
            (Expr::IntLiteral(l_str), Expr::IntLiteral(r_str)) => {
                let l_val = parse_bigint(l_str)?;
                let r_val = parse_bigint(r_str)?;
                let result = match op {
                    BinOpKind::Add => l_val + r_val,
                    BinOpKind::Sub => l_val - r_val,
                    BinOpKind::Mult => l_val * r_val,
                    BinOpKind::FloorDiv if r_val != num_bigint::BigInt::from(0) => l_val / r_val,
                    BinOpKind::Mod if r_val != num_bigint::BigInt::from(0) => l_val % r_val,
                    _ => return None,
                };
                Some(Expr::IntLiteral(result.to_string()))
            }
            (Expr::FloatLiteral(l_val), Expr::FloatLiteral(r_val)) => {
                let result = match op {
                    BinOpKind::Add => l_val + r_val,
                    BinOpKind::Sub => l_val - r_val,
                    BinOpKind::Mult => l_val * r_val,
                    BinOpKind::Div if *r_val != 0.0 => l_val / r_val,
                    _ => return None,
                };
                Some(Expr::FloatLiteral(result))
            }
            (Expr::StringLiteral(l_val), Expr::StringLiteral(r_val)) if op == BinOpKind::Add => {
                Some(Expr::StringLiteral(format!("{}{}", l_val, r_val)))
            }
            _ => None,
        }
    }

    fn fold_unaryop(&self, op: UnaryOpKind, operand: &Expr) -> Option<Expr> {
        match operand {
            Expr::IntLiteral(val_str) => {
                let val = parse_bigint(val_str)?;
                match op {
                    UnaryOpKind::Minus => Some(Expr::IntLiteral((-val).to_string())),
                    UnaryOpKind::Plus => Some(Expr::IntLiteral(val.to_string())),
                    UnaryOpKind::Not => {
                        Some(Expr::BooleanLiteral(val == num_bigint::BigInt::from(0)))
                    }
                    _ => None,
                }
            }
            Expr::FloatLiteral(val) => match op {
                UnaryOpKind::Minus => Some(Expr::FloatLiteral(-val)),
                UnaryOpKind::Plus => Some(Expr::FloatLiteral(*val)),
                UnaryOpKind::Not => Some(Expr::BooleanLiteral(*val == 0.0)),
                _ => None,
            },
            Expr::BooleanLiteral(val) => match op {
                UnaryOpKind::Not => Some(Expr::BooleanLiteral(!val)),
                _ => None,
            },
            _ => None,
        }
    }

    fn try_fold_expr(&self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::BinOp { left, op, right } => {
                let folded_left = self.try_fold_expr(left);
                let folded_right = self.try_fold_expr(right);

                let l = folded_left.as_ref().unwrap_or(left.as_ref());
                let r = folded_right.as_ref().unwrap_or(right.as_ref());

                if let Some(folded_binop) = self.fold_binop(l, *op, r) {
                    Some(folded_binop)
                } else if folded_left.is_some() || folded_right.is_some() {
                    Some(Expr::BinOp {
                        left: Box::new(folded_left.unwrap_or_else(|| *left.clone())),
                        op: *op,
                        right: Box::new(folded_right.unwrap_or_else(|| *right.clone())),
                    })
                } else {
                    None
                }
            }
            Expr::UnaryOp { op, operand } => {
                let folded_operand = self.try_fold_expr(operand);
                let o = folded_operand.as_ref().unwrap_or(operand.as_ref());

                if let Some(folded_unaryop) = self.fold_unaryop(*op, o) {
                    Some(folded_unaryop)
                } else if folded_operand.is_some() {
                    Some(Expr::UnaryOp {
                        op: *op,
                        operand: Box::new(folded_operand.unwrap()),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        if let Some(folded) = self.try_fold_expr(expr) {
            return self.compile_expr(&folded);
        }
        match expr {
            Expr::IntLiteral(val) => {
                let bigint = if val.starts_with("0x") || val.starts_with("0X") {
                    num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 16)
                } else if val.starts_with("0o") || val.starts_with("0O") {
                    num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 8)
                } else if val.starts_with("0b") || val.starts_with("0B") {
                    num_bigint::BigInt::parse_bytes(&val.as_bytes()[2..], 2)
                } else {
                    val.parse().ok()
                }
                .ok_or_else(|| format!("Invalid integer literal: {}", val))?;
                let idx = self.add_constant(Rc::new(PyInt::new(bigint)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::FloatLiteral(val) => {
                let idx = self.add_constant(Rc::new(crate::objects::float::PyFloat::new(*val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::ImagLiteral(val) => {
                let idx =
                    self.add_constant(Rc::new(crate::objects::complex::PyComplex::new(0.0, *val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::StringLiteral(val) => {
                let idx = self.add_constant(Rc::new(PyString::new(val.clone())));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::BytesLiteral(val) => {
                let idx =
                    self.add_constant(Rc::new(crate::objects::bytes::PyBytes::new(val.clone())));
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
            Expr::IfExp { test, body, orelse } => {
                self.compile_expr(test)?;
                let else_jump = self.emit(Opcode::PopJumpIfFalse(0));
                self.compile_expr(body)?;
                let end_jump = self.emit(Opcode::JumpAbsolute(0));
                let else_target = self.code.instructions.len();
                self.code.instructions[else_jump] = Opcode::PopJumpIfFalse(else_target);
                self.compile_expr(orelse)?;
                let end_target = self.code.instructions.len();
                self.code.instructions[end_jump] = Opcode::JumpAbsolute(end_target);
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
                if self.global_names.contains(name) {
                    self.emit(Opcode::LoadGlobal(idx));
                } else {
                    self.emit(Opcode::LoadName(idx));
                }
            }
            Expr::BinOp { left, op, right } => match op {
                BinOpKind::And | BinOpKind::Or => {
                    self.compile_expr(left)?;
                    self.emit(Opcode::DupTop);
                    let is_and = *op == BinOpKind::And;
                    let jump_idx = if is_and {
                        self.emit(Opcode::PopJumpIfFalse(0))
                    } else {
                        self.emit(Opcode::PopJumpIfTrue(0))
                    };
                    self.emit(Opcode::PopTop);
                    self.compile_expr(right)?;
                    let end = self.code.instructions.len();
                    self.code.instructions[jump_idx] = if is_and {
                        Opcode::PopJumpIfFalse(end)
                    } else {
                        Opcode::PopJumpIfTrue(end)
                    };
                }
                _ => {
                    self.compile_expr(left)?;
                    self.compile_expr(right)?;
                    self.emit_binop(*op)?;
                }
            },
            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand)?;
                let opcode = match op {
                    crate::ast::UnaryOpKind::Minus => Opcode::UnaryNegative,
                    crate::ast::UnaryOpKind::Plus => Opcode::UnaryPositive,
                    crate::ast::UnaryOpKind::Not => Opcode::UnaryNot,
                    crate::ast::UnaryOpKind::Invert => Opcode::UnaryInvert,
                };
                self.emit(opcode);
            }
            Expr::Call {
                func,
                args,
                kwargs,
                starargs,
                kwargs_unpack,
            } => {
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
                            let idx = self.add_constant(std::rc::Rc::new(
                                crate::objects::string::PyString::new(key.clone()),
                            ));
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
            Expr::Tuple(elements) => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.emit(Opcode::BuildTuple(elements.len()));
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
            Expr::Slice {
                value,
                start,
                stop,
                step,
            } => {
                self.compile_expr(value)?;
                // Push start (or None)
                if let Some(start_expr) = start {
                    self.compile_expr(start_expr)?;
                } else {
                    let idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(idx));
                }
                // Push stop (or None)
                if let Some(stop_expr) = stop {
                    self.compile_expr(stop_expr)?;
                } else {
                    let idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(idx));
                }
                // Push step (or None)
                if let Some(step_expr) = step {
                    self.compile_expr(step_expr)?;
                } else {
                    let idx = self.add_constant(Rc::new(PyNone::new()));
                    self.emit(Opcode::LoadConst(idx));
                }
                self.emit(Opcode::BuildSlice);
                self.emit(Opcode::BinarySubscript);
            }
            Expr::Set(elements) => {
                let set_name = self.get_or_add_name("set");
                self.emit(Opcode::LoadName(set_name));
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                self.emit(Opcode::BuildList(elements.len()));
                self.emit(Opcode::CallFunction(1));
            }
            Expr::Attribute { value, attr } => {
                self.compile_expr(value)?;
                self.emit(Opcode::LoadAttr(attr.clone()));
            }
            Expr::Lambda {
                params,
                posonly_params,
                kwonly_params,
                vararg,
                kwarg,
                body,
            } => {
                let mut child_compiler = Compiler::new("<lambda>".to_string());
                child_compiler.code.filename = self.filename.clone();
                child_compiler.filename = self.filename.clone();
                child_compiler.code.arg_count = params.len() + posonly_params.len();
                child_compiler.code.vararg = vararg.clone();
                child_compiler.code.kwarg = kwarg.clone();

                for param in posonly_params {
                    child_compiler.get_or_add_name(param);
                }
                for param in params {
                    child_compiler.get_or_add_name(param);
                }
                for param in kwonly_params {
                    child_compiler.get_or_add_name(param);
                }
                if let Some(v) = vararg {
                    child_compiler.get_or_add_name(v);
                }
                if let Some(k) = kwarg {
                    child_compiler.get_or_add_name(k);
                }

                child_compiler.compile_expr(body)?;
                child_compiler.emit(Opcode::ReturnValue);

                self.emit(Opcode::BuildTuple(0)); // no kwonly defaults
                self.emit(Opcode::BuildTuple(0)); // no positional defaults
                let code_obj = Rc::new(child_compiler.code);
                let code_idx = self.add_constant(code_obj);
                self.emit(Opcode::LoadConst(code_idx));
                self.emit(Opcode::MakeFunction);
            }
            Expr::Starred { value } => {
                self.compile_expr(value)?;
            }
            Expr::NamedExpr { target, value } => {
                self.compile_expr(value)?;
                self.emit(Opcode::DupTop);
                if let Expr::Identifier(name) = target.as_ref() {
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::StoreName(name_idx));
                } else {
                    return Err("CompilerError: NamedExpr target must be an identifier".to_string());
                }
            }
            Expr::FString(segments) => {
                for (i, seg) in segments.iter().enumerate() {
                    match seg {
                        FStringSegment::Text(text) => {
                            let idx = self.add_constant(Rc::new(PyString::new(text.clone())));
                            self.emit(Opcode::LoadConst(idx));
                        }
                        FStringSegment::Expr {
                            expr,
                            format_spec,
                            debug,
                        } => {
                            if *debug {
                                let repr_idx = self.get_or_add_name("repr");
                                self.emit(Opcode::LoadName(repr_idx));
                                self.compile_expr(expr)?;
                                self.emit(Opcode::CallFunction(1));
                                if let Some(spec) = format_spec {
                                    let format_idx = self.get_or_add_name("format");
                                    self.emit(Opcode::LoadName(format_idx));
                                    self.emit(Opcode::RotTwo);
                                    let spec_idx =
                                        self.add_constant(Rc::new(PyString::new(spec.clone())));
                                    self.emit(Opcode::LoadConst(spec_idx));
                                    self.emit(Opcode::CallFunction(2));
                                }
                            } else if let Some(spec) = format_spec {
                                let format_idx = self.get_or_add_name("format");
                                self.emit(Opcode::LoadName(format_idx));
                                self.compile_expr(expr)?;
                                let spec_idx =
                                    self.add_constant(Rc::new(PyString::new(spec.clone())));
                                self.emit(Opcode::LoadConst(spec_idx));
                                self.emit(Opcode::CallFunction(2));
                            } else {
                                let str_idx = self.get_or_add_name("str");
                                self.emit(Opcode::LoadName(str_idx));
                                self.compile_expr(expr)?;
                                self.emit(Opcode::CallFunction(1));
                            }
                        }
                    }
                    if i > 0 {
                        self.emit(Opcode::BinaryAdd);
                    }
                }
                if segments.is_empty() {
                    let idx = self.add_constant(Rc::new(PyString::new(String::new())));
                    self.emit(Opcode::LoadConst(idx));
                }
            }
            Expr::Await(expr) => {
                // For async functions, we need to GET_AWAITABLE and YIELD_FROM
                // But the code object should already be marked is_async by the function def
                self.code.is_async = true;
                self.compile_expr(expr)?;
                self.emit(Opcode::GetAwaitable);
                let idx = self.add_constant(Rc::new(PyNone::new()));
                self.emit(Opcode::LoadConst(idx));
                self.emit(Opcode::YieldFrom);
            }
            Expr::YieldFrom(expr) => {
                self.code.is_generator = true;
                self.compile_expr(expr)?;
                self.emit(Opcode::GetIter);
                let idx = self.add_constant(Rc::new(PyNone::new()));
                self.emit(Opcode::LoadConst(idx));
                self.emit(Opcode::YieldFrom);
            }
            Expr::Ellipsis => {
                let idx = self.add_constant(Rc::new(crate::objects::constants::PyEllipsis));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::Comprehension {
                kind,
                elt,
                key,
                target,
                iter,
                ifs,
            } => {
                match kind {
                    CompKind::List => {
                        self.compile_expr(iter)?;
                        self.emit(Opcode::GetIter);
                        let iter_name = self.get_or_add_name("__comp_iter");
                        self.emit(Opcode::StoreName(iter_name));

                        self.emit(Opcode::BuildList(0));
                        let result_name = self.get_or_add_name("__comp_result");
                        self.emit(Opcode::StoreName(result_name));

                        let loop_start = self.code.instructions.len();

                        self.emit(Opcode::LoadName(iter_name));
                        let for_iter_idx = self.emit(Opcode::ForIter(0));
                        // FOR_ITER pushes [iter, item]; store item first, then iter

                        if let Expr::Identifier(name) = target.as_ref() {
                            let target_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(target_idx));
                        } else {
                            return Err("Comprehension target must be an identifier".to_string());
                        }

                        self.emit(Opcode::StoreName(iter_name));

                        // Filter checks
                        for if_cond in ifs {
                            self.compile_expr(if_cond)?;
                            self.emit(Opcode::PopJumpIfFalse(loop_start));
                        }

                        self.emit(Opcode::LoadName(result_name));
                        self.compile_expr(elt)?;
                        self.emit(Opcode::ListAppend);
                        self.emit(Opcode::StoreName(result_name));

                        self.emit(Opcode::JumpAbsolute(loop_start));

                        let loop_end = self.code.instructions.len();
                        self.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                        self.emit(Opcode::LoadName(result_name));
                    }
                    CompKind::Set => {
                        self.compile_expr(iter)?;
                        self.emit(Opcode::GetIter);
                        let iter_name = self.get_or_add_name("__comp_iter");
                        self.emit(Opcode::StoreName(iter_name));

                        self.emit(Opcode::BuildSet(0));
                        let result_name = self.get_or_add_name("__comp_result");
                        self.emit(Opcode::StoreName(result_name));

                        let loop_start = self.code.instructions.len();

                        self.emit(Opcode::LoadName(iter_name));
                        let for_iter_idx = self.emit(Opcode::ForIter(0));

                        if let Expr::Identifier(name) = target.as_ref() {
                            let target_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(target_idx));
                        } else {
                            return Err("Comprehension target must be an identifier".to_string());
                        }

                        self.emit(Opcode::StoreName(iter_name));

                        // Filter checks
                        for if_cond in ifs {
                            self.compile_expr(if_cond)?;
                            self.emit(Opcode::PopJumpIfFalse(loop_start));
                        }

                        self.emit(Opcode::LoadName(result_name));
                        self.compile_expr(elt)?;
                        self.emit(Opcode::SetAdd);
                        self.emit(Opcode::StoreName(result_name));

                        self.emit(Opcode::JumpAbsolute(loop_start));

                        let loop_end = self.code.instructions.len();
                        self.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                        self.emit(Opcode::LoadName(result_name));
                    }
                    CompKind::Generator => {
                        let mut child = Compiler::new("<genexpr>".to_string());
                        child.code.filename = self.filename.clone();
                        child.filename = self.filename.clone();
                        child.code.is_generator = true;

                        child.compile_expr(iter)?;
                        child.emit(Opcode::GetIter);
                        let iter_name = child.get_or_add_name("__comp_iter");
                        child.emit(Opcode::StoreName(iter_name));

                        let loop_start = child.code.instructions.len();
                        child.emit(Opcode::LoadName(iter_name));
                        let for_iter_idx = child.emit(Opcode::ForIter(0));

                        if let Expr::Identifier(name) = target.as_ref() {
                            let target_idx = child.get_or_add_name(name);
                            child.emit(Opcode::StoreName(target_idx));
                        } else {
                            return Err("Generator target must be an identifier".to_string());
                        }

                        child.emit(Opcode::StoreName(iter_name));

                        // Filter checks
                        for if_cond in ifs {
                            child.compile_expr(if_cond)?;
                            child.emit(Opcode::PopJumpIfFalse(loop_start));
                        }

                        child.compile_expr(elt)?;
                        child.emit(Opcode::YieldValue);
                        child.emit(Opcode::JumpAbsolute(loop_start));

                        let loop_end = child.code.instructions.len();
                        child.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                        // No code after loop exit — run() returns None -> StopIteration

                        self.emit(Opcode::BuildTuple(0)); // no kwonly defaults
                        self.emit(Opcode::BuildTuple(0)); // no positional defaults
                        let code_obj = Rc::new(child.code);
                        let code_idx = self.add_constant(code_obj);
                        self.emit(Opcode::LoadConst(code_idx));
                        self.emit(Opcode::MakeFunction);
                        self.emit(Opcode::CallFunction(0));
                    }
                    CompKind::Dict => {
                        self.compile_expr(iter)?;
                        self.emit(Opcode::GetIter);
                        let iter_name = self.get_or_add_name("__comp_iter");
                        self.emit(Opcode::StoreName(iter_name));

                        self.emit(Opcode::BuildMap(0));
                        let result_name = self.get_or_add_name("__comp_result");
                        self.emit(Opcode::StoreName(result_name));

                        let loop_start = self.code.instructions.len();

                        self.emit(Opcode::LoadName(iter_name));
                        let for_iter_idx = self.emit(Opcode::ForIter(0));

                        if let Expr::Identifier(name) = target.as_ref() {
                            let target_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(target_idx));
                        } else {
                            return Err("Comprehension target must be an identifier".to_string());
                        }

                        self.emit(Opcode::StoreName(iter_name));

                        // Filter checks
                        for if_cond in ifs {
                            self.compile_expr(if_cond)?;
                            self.emit(Opcode::PopJumpIfFalse(loop_start));
                        }

                        self.emit(Opcode::LoadName(result_name));
                        if let Some(key_expr) = key {
                            self.compile_expr(key_expr)?;
                        }
                        self.compile_expr(elt)?;
                        self.emit(Opcode::MapAdd);
                        self.emit(Opcode::StoreName(result_name));

                        self.emit(Opcode::JumpAbsolute(loop_start));

                        let loop_end = self.code.instructions.len();
                        self.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                        self.emit(Opcode::LoadName(result_name));
                    }
                }
            }
        }
        Ok(())
    }
}
