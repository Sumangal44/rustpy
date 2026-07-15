# RustPy — Development Rules

## Code Style

- **No comments** in code (unless explaining a non-obvious workaround)
- Follow existing patterns: look at neighboring code before writing
- Match the project's naming conventions (`snake_case` for fn/vars, `CamelCase` for types)
- Error messages should match CPython's format exactly (compat tests compare output)

## Architecture Rules

- **No new dependencies** unless absolutely necessary. Currently only `num-bigint` and `num-traits`. Adding dependencies is a last resort.
- The `PyObject` trait in `objects/mod.rs` is the central abstraction — all objects must implement it
- Object operators return `Option<Rc<dyn PyObject>>` — `None` means "not supported" (the VM will try reverse operation or raise TypeError)
- The VM NEVER calls object methods for unsupported operations directly — it always checks for `Some`/`None`

## Testing Rules

- **Always run both test suites before committing**:
  ```bash
  cargo test
  python3 tests/compat_test.py
  ```
- All 259 unit tests must pass (regression-free)
- All 405+ compat tests must pass (regression-free)
- Add a compat test for every new feature
- Unit tests go in `src/main.rs` (the test module at the bottom)

## Parsing Rules

- Recursive descent, one-token lookahead
- `parse_suite()` MUST handle both indented blocks and same-line bodies separated by `;`
- Semicolons are NOT consumed by `parse_statement()` — they must be visible to `parse_suite()`
- `consume_stmt_end()` consumes only `\n`, never `;`

## Compiler Rules

- The `CodeObject` holds `names: Vec<String>` and `constants: Vec<Rc<dyn PyObject>>`
- Names are referenced by index — use `add_name()` / `add_constant()`
- Child compiler for every new scope (function, comprehension, generator)
- `MakeFunction` opcode expects: defaults tuple (bottom), code object (top)

## VM Rules

- `invoke_inner()` handles all callable dispatch
- Block stack for structured control flow: try, finally, with, loop
- Always use `Rc<dyn PyObject>` for object references
- Never unwrap — propagate errors via `Result`

## Process

- Before starting a feature, check `prd.md` for Tier
- After implementing, add test(s) and run both suites
- Update `phases.md` when completing a phase
- Update this file if adding a new rule that future work should follow
