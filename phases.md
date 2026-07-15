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
- [ ] Generator `.throw()`, `.close()`
- [ ] Relative imports (`from . import x`)
- [ ] Complete `__format__` on all types
- [ ] Full exception hierarchy (not flat)
- [ ] `del` with multiple targets: `del a, b`
- [ ] `@=` augmented assignment

## Phase 3 — Polish & Hardening (Future)
- [ ] `__slots__`
- [ ] `memoryview`
- [ ] Match keyword patterns: `case Point(x=1):`
- [ ] Walrus operator in nested expressions
- [ ] `map()` / `filter()` with Python functions (not just native)
- [ ] Better error messages with tracebacks
- [ ] f-string `=` debug format for complex expressions
- [ ] `super()` outside of methods
- [ ] Negative/advanced slice handling in `slice()` builtin
- [ ] `open()` encoding parameter support
- [ ] `bytes.decode()` with non-utf-8 encodings

## Phase 4 — Performance & Scale (Future)
- [ ] Recursion limit / stack overflow handling (done)
- [ ] Reference counting / GC correctness
- [ ] Bytecode optimizations (constant folding, peephole)
- [ ] Tail call optimization
- [ ] Large program stress testing
