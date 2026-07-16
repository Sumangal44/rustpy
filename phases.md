# RustPy — Implementation Phases

## Phase 1 — Core Interpreter (Complete)
- [x] Lexer: full token set including INDENT/DEDENT, f-strings, bytes
- [x] Parser: all statement and expression types (recursive descent)
- [x] Compiler: AST → bytecode for all AST nodes
- [x] VM: stack-based execution engine
- [x] Object system: `PyObject` trait with 28 implementations
- [x] Runtime: scoped environment with parent chains
- [x] All operators: arithmetic, bitwise, comparison, boolean, unary
- [x] Control flow: `if`, `while`, `for`, `break`, `continue`
- [x] Functions: def, lambda, closures, `*args`, `**kwargs`, decorators
- [x] Classes: inheritance, `@staticmethod`, `@classmethod`, `@property`, `super()`
- [x] Data structures: list, dict, set, tuple, str, bytes, bytearray, range, slice
- [x] Comprehensions: list, dict, set, generator expressions
- [x] Exception handling: try/except/else/finally, raise
- [x] Imports: absolute, from, import as, star
- [x] 259 unit tests, 405 compat tests passing

## Phase 2 — Missing Language Features (Complete)

### Priority A (blockers for real-world code)
- [x] Type annotations: parse `x: int`, `def f() -> int`, store but don't enforce
- [x] Keyword-only arguments: `def f(*, a, b):`
- [x] Positional-only arguments: `def f(a, /, b):`
- [x] Starred assignment: `a, *b = range(10)`
- [x] For-loop tuple unpacking: `for a, b in pairs:`
- [x] Multiple except types: `except (A, B):`
- [x] Exception chaining: `raise X from Y`
- [x] Bare raise: `raise` with no argument (re-raise)
- [x] Multiple context managers: `with a, b:`
- [x] `async for` / `async with`
- [x] C3 linearization for proper MRO

### Priority B (important but less blocking)
- [x] Generator `.throw()`, `.close()`
- [x] Relative imports (`from . import x`)
- [x] Complete `__format__` on all types
- [x] Full exception hierarchy (not flat)
- [x] `del` with multiple targets: `del a, b`
- [x] `@=` augmented assignment

## Phase 3 — Polish & Hardening (Complete)
- [x] `__slots__`
- [x] `memoryview`
- [x] Match keyword patterns: `case Point(x=1):`
- [x] Walrus operator in nested expressions
- [x] `map()` / `filter()` with Python functions (not just native)
- [x] f-string `=` debug format for complex expressions
- [x] `super()` outside of methods
- [x] Negative/advanced slice handling in `slice()` builtin
- [x] `open()` encoding parameter support
- [x] `bytes.decode()` with non-utf-8 encodings
- [x] Better error messages with tracebacks

## Session 5 (Jul 16) — Phase 3 Completion + Python callables in builtins
- [x] `del a, b, c`: multiple comma-separated targets (AST: `Del { targets: Vec<Expr> }`)
- [x] `@=` augmented assignment (parser `parse_aug_op` -> `TokenKind::AtEqual`)
- [x] Walrus operator in nested expressions (already worked, no change)
- [x] f-string `=` debug format (`FStringSegment::Expr { debug: bool }`, compiles to `repr()`)
- [x] `super()` outside methods (message now matches CPython 3.14)
- [x] Negative/advanced slice iteration on all 5 collections (while i > stop, no .max(1))
- [x] `open()` encoding parameter (new `src/encoding.rs`: utf-8, ascii, latin-1, utf-16 BOM)
- [x] `bytes.decode()` / `bytearray.decode()` / `str.encode()` with non-utf-8 encodings
- [x] `map()` / `filter()` with Python functions (uses `VirtualMachine::invoke` on `Rc<dyn PyObject>`, removes native-only restriction)
- [x] All 13 test suites pass: `cargo test` 259/259, `compat_test.py` 410/410 (was 405), `tier3_test.py` 22/22, `run_all_tests.py` 13/13

## Phase 4 — Performance & Scale (Future)
- [x] Recursion limit / stack overflow handling (done)
- [x] Reference counting / GC correctness
- [x] Bytecode optimizations (constant folding, peephole)
- [x] Tail call optimization
- [x] Large program stress testing
