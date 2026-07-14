pub mod code;
pub mod opcodes;

use crate::ast::{BinOpKind, CompKind, Expr, FStringSegment, Module, Pattern, Stmt};
use crate::compiler::code::CodeObject;
use crate::compiler::opcodes::Opcode;
use crate::objects::{PyObject, bool::PyBool, int::PyInt, none::PyNone, string::PyString};
use std::rc::Rc;

struct LoopInfo {
    start: usize,
    break_targets: Vec<usize>,
}

pub struct Compiler {
    code: CodeObject,
    loop_stack: Vec<LoopInfo>,
}

impl Compiler {
    pub fn new(name: String) -> Self {
        Self {
            code: CodeObject::new(name),
            loop_stack: Vec::new(),
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

                if targets.len() == 1 {
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
                        Expr::Subscript { value, slice } => {
                            self.compile_expr(value)?;
                            self.compile_expr(slice)?;
                            self.emit(Opcode::StoreSubscript);
                        }
                        _ => {
                            return Err("CompilerError: Unsupported assignment target".to_string());
                        }
                    }
                } else {
                    self.emit(Opcode::UnpackSequence(targets.len()));
                    for target in targets.iter() {
                        if let Expr::Identifier(name) = target {
                            let name_idx = self.get_or_add_name(name);
                            self.emit(Opcode::StoreName(name_idx));
                        } else {
                            return Err("CompilerError: Unsupported unpack target".to_string());
                        }
                    }
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
                    Expr::Attribute { value: target_val, attr } => {
                        self.compile_expr(target_val)?;
                        // Dup the object by compiling twice (simplified)
                        self.compile_expr(target_val)?;
                        self.emit(Opcode::LoadAttr(attr.clone()));
                        self.compile_expr(value)?;
                        self.emit_binop(op.clone())?;
                        self.emit(Opcode::StoreAttr(attr.clone()));
                    }
                    _ => {
                        return Err("CompilerError: Unsupported augmented assignment target".to_string());
                    }
                }
            }
            Stmt::Break => {
                if self.loop_stack.is_empty() {
                    return Err("SyntaxError: 'break' outside loop".to_string());
                }
                let idx_in_stack = self.loop_stack.len() - 1;
                let break_idx = self.emit(Opcode::JumpAbsolute(0));
                self.loop_stack[idx_in_stack].break_targets.push(break_idx);
            }
            Stmt::Continue => {
                let start = self.loop_stack.last().ok_or_else(|| {
                    "SyntaxError: 'continue' outside loop".to_string()
                })?.start;
                self.emit(Opcode::JumpAbsolute(start));
            }
            Stmt::Del { target } => {
                match target.as_ref() {
                    Expr::Identifier(name) => {
                        let name_idx = self.get_or_add_name(name);
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
            Stmt::Global { names } => {
                // For now, we just ensure the name is stored at global scope
                // In a real implementation we'd use a StoreGlobal opcode
                // For simplicity, we mark it in the code object
                for name in names {
                    self.get_or_add_name(name);
                    // We'll handle this in the VM by looking at the code object's globals
                }
                // Emit nothing for now - the effect is that StoreName will
                // check if the name should be stored globally
                // For a proper implementation we'd need StoreGlobal opcode
            }
            Stmt::Nonlocal { names } => {
                for name in names {
                    self.get_or_add_name(name);
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

                    if let Some(guard) = &case.guard {
                        self.compile_expr(guard)?;
                        let guard_false_idx = self.emit(Opcode::PopJumpIfFalse(0));
                        for s in &case.body {
                            self.compile_stmt(s)?;
                        }
                        self.code.instructions[guard_false_idx] =
                            Opcode::PopJumpIfFalse(self.code.instructions.len());
                    } else {
                        for s in &case.body {
                            self.compile_stmt(s)?;
                        }
                    }

                    if case_idx < cases.len() - 1 {
                        next_case_indices.push(self.emit(Opcode::JumpAbsolute(0)));
                    }

                    self.code.instructions[jump_false_idx] =
                        Opcode::PopJumpIfFalse(self.code.instructions.len());
                }

                let end_pos = self.code.instructions.len();
                for idx in next_case_indices {
                    self.code.instructions[idx] = Opcode::JumpAbsolute(end_pos);
                }
            }
            Stmt::Assert { test, msg } => {
                self.compile_expr(test)?;
                let jump_if_true_idx = self.emit(Opcode::PopJumpIfTrue(0));

                // Load "Exception" name (it's always available as a builtin)
                let exc_name_idx = self.get_or_add_name("Exception");
                self.emit(Opcode::LoadName(exc_name_idx));

                if let Some(msg_expr) = msg {
                    self.compile_expr(msg_expr)?;
                } else {
                    let idx = self.add_constant(Rc::new(crate::objects::string::PyString::new("AssertionError".to_string())));
                    self.emit(Opcode::LoadConst(idx));
                }

                self.emit(Opcode::CallFunction(1));
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
            Stmt::For { target, iter, body, orelse } => {
                self.compile_expr(iter)?;
                self.emit(Opcode::GetIter);
                let loop_start = self.code.instructions.len();
                self.loop_stack.push(LoopInfo {
                    start: loop_start,
                    break_targets: Vec::new(),
                });
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
                
                let info = self.loop_stack.pop().unwrap();

                for s in orelse {
                    self.compile_stmt(s)?;
                }

                let after_orelse = self.code.instructions.len();
                for idx in &info.break_targets {
                    self.code.instructions[*idx] = Opcode::JumpAbsolute(after_orelse);
                }
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
            Stmt::Try { body, handlers, else_body, finally_body } => {
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

                    for (_exc_type, handler_body) in handlers {
                        self.emit(Opcode::PopTop);
                        for s in handler_body {
                            self.compile_stmt(s)?;
                        }
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
            Stmt::FunctionDef { name, params, vararg, kwarg, body, decorators, is_async } => {
                // Compile decorators first so they are on the stack
                for decorator in decorators {
                    self.compile_expr(decorator)?;
                }

                let mut child_compiler = Compiler::new(name.clone());
                child_compiler.code.arg_count = params.len();
                child_compiler.code.is_async = *is_async;
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
            Stmt::Import { names } => {
                for alias in names {
                    let name = &alias.name;

                    // For dotted imports, only the top-level name is bound
                    let top_level = name.split('.').next().unwrap_or(name).to_string();
                    let top_store = alias.asname.as_ref().unwrap_or(&top_level);

                    // Push level=0, fromlist=()
                    let zero_idx = self.add_constant(Rc::new(crate::objects::int::PyInt::from_i64(0)));
                    self.emit(Opcode::LoadConst(zero_idx));
                    let empty_tuple_idx = self.add_constant(Rc::new(crate::objects::tuple::PyTuple::new(vec![])));
                    self.emit(Opcode::LoadConst(empty_tuple_idx));
                    // IMPORT_NAME with the full module name
                    let name_idx = self.get_or_add_name(name);
                    self.emit(Opcode::ImportName(name_idx));
                    // Store the top-level module
                    let store_idx = self.get_or_add_name(top_store);
                    self.emit(Opcode::StoreName(store_idx));
                }
            }
            Stmt::ImportFrom { module, names, level: _level } => {
                let mut from_names: Vec<Rc<dyn PyObject>> = Vec::new();
                let is_star = names.len() == 1 && names[0].name == "*";

                if is_star {
                    // fromlist has a sentinel to distinguish star import
                    let zero = Rc::new(crate::objects::int::PyInt::from_i64(0));
                    from_names.push(zero);
                } else {
                    for alias in names {
                        let name_obj = Rc::new(crate::objects::string::PyString::new(alias.name.clone()));
                        from_names.push(name_obj);
                    }
                }

                // Push level, fromlist
                let level_idx = self.add_constant(Rc::new(crate::objects::int::PyInt::from_i64(0)));
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
                        self.emit(Opcode::PopTop); // pop the module
                    }
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
            Pattern::Sequence(_subpatterns) => {
                // Simple sequence matching: compare each element
                // For now, just check that subject is a list and compare elements
                // This is a simplified version
                let true_idx = self.add_constant(Rc::new(PyBool::new(true)));
                self.emit(Opcode::LoadConst(true_idx));
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

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLiteral(val) => {
                let bigint: num_bigint::BigInt = val.parse().map_err(|_| format!("Invalid integer literal: {}", val))?;
                let idx = self.add_constant(Rc::new(PyInt::new(bigint)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::FloatLiteral(val) => {
                let idx = self.add_constant(Rc::new(crate::objects::float::PyFloat::new(*val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::ImagLiteral(val) => {
                let idx = self.add_constant(Rc::new(crate::objects::complex::PyComplex::new(0.0, *val)));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::StringLiteral(val) => {
                let idx = self.add_constant(Rc::new(PyString::new(val.clone())));
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::BytesLiteral(val) => {
                let idx = self.add_constant(Rc::new(crate::objects::bytes::PyBytes::new(val.clone())));
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
                match op {
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
                }
            }
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
            Expr::Slice { value, start, stop, step } => {
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
            Expr::Lambda { params, vararg, kwarg, body } => {
                let mut child_compiler = Compiler::new("<lambda>".to_string());
                child_compiler.code.arg_count = params.len();
                child_compiler.code.vararg = vararg.clone();
                child_compiler.code.kwarg = kwarg.clone();

                for param in params {
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

                let code_obj = Rc::new(child_compiler.code);
                let code_idx = self.add_constant(code_obj);
                self.emit(Opcode::LoadConst(code_idx));
                self.emit(Opcode::MakeFunction);
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
                        FStringSegment::Expr(expr) => {
                            let str_idx = self.get_or_add_name("str");
                            self.emit(Opcode::LoadName(str_idx));
                            self.compile_expr(expr)?;
                            self.emit(Opcode::CallFunction(1));
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
                self.emit(Opcode::YieldFrom);
            }
            Expr::Ellipsis => {
                let idx = self.add_constant(Rc::new(PyNone::new())); // TODO: Replace with Ellipsis object
                self.emit(Opcode::LoadConst(idx));
            }
            Expr::Comprehension { kind, elt, key, target, iter } => {
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
                        child.compile_expr(elt)?;
                        child.emit(Opcode::YieldValue);
                        child.emit(Opcode::JumpAbsolute(loop_start));

                        let loop_end = child.code.instructions.len();
                        child.code.instructions[for_iter_idx] = Opcode::ForIter(loop_end);

                        // No code after loop exit — run() returns None -> StopIteration

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
