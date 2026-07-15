# RustPy — AI Agent Memory

## Identity
RustPy is a Python interpreter written in Rust at `/Users/sumu/rustpy/`. It is NOT "opencode" or any AI tool — it is the USER's project that you are helping to build.

## Current State

**Tests**: 259 unit tests pass, 405 compat tests pass, 0 failures.

**Compatibility target**: Python 3.10+ syntax. Not production-ready for real libraries.

## Key Commands

```bash
cargo test                    # Run all unit tests
python3 tests/compat_test.py  # Run compatibility suite
python3 tests/compat_test.py --fix  # Auto-update expected outputs
cargo build                   # Build debug binary
/Users/sumu/rustpy/target/debug/rustpy <file>  # Run a file
```

## Recent Work (Session History)

### Added: 11 major Python language features
All Tier-2 language features implemented in a single session:

1. **Keyword-only args** (`def f(*, a, b):`) — Parser handles bare `*` separator, VM binds keyword-only params with right-aligned defaults.
2. **Positional-only args** (`def f(a, /, b):`) — `/` token in lexer, parser separates posonly from regular params, VM rejects posonly params as kwargs.
3. **Type annotations** (`x: int`, `-> int`) — Parser consumes `:` and `->` annotation expressions, discards at runtime (Python semantics).
4. **Starred assignment** (`a, *b = range(10)`) — New `UnpackEx(usize, usize)` opcode handles starred unpacking. Supports star at any position.
5. **For-loop tuple unpacking** (`for a, b in pairs:`) — Parser accepts tuple targets, compiler uses `UnpackSequence`.
6. **Multiple except types** (`except (A, B):`) — Parser expands tuple types into separate handler entries.
7. **Exception chaining** (`raise X from Y`) — New `cause` field on `Stmt::Raise`, VM sets `__cause__` on exception.
8. **Bare raise** (re-raise) — `exc` field made `Option`, VM re-raises `last_exception`.
9. **Multiple context managers** (`with a, b:`) — `WithItem` struct, compiler emits nested SetupWith/WithCleanup.
10. **`async for` / `async with`** — `is_async` flag on For/With, parsed in `parse_async_statement`.
11. **C3 linearization** — C3 merge algorithm in `class.rs` for correct MRO on diamond inheritance.

### Fixed: Semicolons in function bodies
- **Root cause**: `parse_assign_or_expr` and `consume_stmt_end` consumed `;` internally, preventing `parse_suite` from seeing them to parse subsequent same-line statements.
- **Fix**: Modified 5 locations:
  1. `consume_stmt_end` — only consume `\n`, not `;`
  2. `parse_assign_or_expr` — only consume `\n`, not `;` (3 branches: AugAssign, Assign, ExprStmt)
  3. `parse_module` — added `;` dispatch loop after `parse_statement()`
  4. `parse_block` — added `;` dispatch loop after `parse_statement()`
  5. `parse_suite` (indented branch) — added `;` dispatch loop after `parse_statement()`
- The inline branch of `parse_suite` already worked correctly once `;` was preserved.

### Fixed: elif parsing
- **Root cause**: `parse_if()` consumed `If` token explicitly, but when called recursively for `elif`, current token was `Elif` (not `If`), causing `consume(If)` to fail.
- **Fix**: Changed `parse_if()` to check for either `If` or `Elif` at entry and advance past whichever it finds.

### Previous fixes (all resolved)
- StoreSubscript VM opcode pop order
- Dict insertion ordering (`ordered_keys`)
- Dict copy, dict.fromkeys
- `dir()` with no args
- `__next__` on all iterators
- `@staticmethod` / `@classmethod` descriptor unwrapping
- `issubclass` / `isinstance` for `object` base
- `del` attribute / subscript / variable opcodes
- List add/mul operations
- Set copy (was correct — test bug fixed)
- Hex/oct/bin integer literal parsing
- Triple-quoted string lexing
- Default parameter values
- `if` filter clauses in comprehensions
- Generator expressions (was already working)
- `Expr::IfExp` (ternary expressions)
- F-string format specs
- Ellipsis literal
- `@property` getter for user-defined functions
- `super()` Rc wrapping
- `raise` with message
- `assert` to use `AssertionError`
- Multi-except type checking
- `ZeroDivisionError` for div by zero
- `global` keyword support
- Recursion depth limit
- Pattern matching `MatchClassCheck`
- Try handler type-checking code
- `bound_method.rs` instance type
- Dict iteration with `PyDictKeyIterator`

## Missing Features (see `phases.md` for full list)

**Tier 2** — All 11 major features implemented (keyword-only args, positional-only args, type annotations, starred assignment, for-loop unpacking, multiple except types, exception chaining, bare raise, multiple context managers, async for/with, C3 linearization).

**Tier 3** — Minor remaining gaps: `__slots__`, generator `.throw()`/`.close()`, relative imports, complete `__format__`, `memoryview`, match keyword patterns, `@=` augmented assignment.

## Important Design Details

- `parse_suite()` has two branches: indented (Newline → Indent) and inline (same-line body)
- Semicolons MUST be visible to `parse_suite()` — they are NOT consumed by `parse_statement()`
- `parse_assign_or_expr()` consumes only `\n`, never `;`
- `PyObject` trait uses `Option<Rc<dyn PyObject>>` returns — `None` = unsupported, triggers reverse op fallback
- CodeObject holds flat `names: Vec<String>` and `constants: Vec<Rc<dyn PyObject>>` indexed by opcode operands
- Blocks in the VM: SetupExcept/PopExcept, SetupFinally/PopFinally, SetupWith/WithCleanup
- The project has ZERO external dependencies beyond `num-bigint` and `num-traits`

## SOP for Adding a Feature

1. Check `prd.md` for the feature's tier; update it if adding something new
2. Modify lexer (if new tokens) → parser (if new syntax) → AST (if new node types) → compiler (emit opcodes) → VM (handle opcodes) → objects (if new type)
3. Add compat test(s) to `tests/compat_test.py`
4. Run `cargo test && python3 tests/compat_test.py`
5. Update `phases.md` to mark the feature complete
