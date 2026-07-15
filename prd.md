# RustPy — Product Requirements Document

## Vision
A lightweight, correct Python interpreter written in Rust for educational and embedded use. Not a CPython replacement — a clean, auditable reimplementation.

## Target
- **Language**: Python 3.10+ (core syntax + pattern matching)
- **Compatibility**: Pass all common Python idioms correctly (405+ compat tests)
- **Non-goals**: Full stdlib, C extensions, JIT, performance parity, pip/package management

## Feature Requirements

### Tier 1 — Core (implemented)
- Arithmetic, bitwise, comparison, boolean operators
- Control flow: `if`/`elif`/`else`, `while`, `for`, `match`
- Functions: `def`, lambda, closures, decorators, `*args`/`**kwargs`
- Classes: inheritance, `@staticmethod`, `@classmethod`, `@property`, `super()`
- Data structures: `list`, `dict`, `set`, `tuple`, `str`, `bytes`, `bytearray`, `range`, `slice`
- Comprehensions: list, dict, set, generator
- Exception handling: `try`/`except`/`else`/`finally`, `raise`
- Imports: absolute, `from`, `import as`, `*`
- `async`/`await` (basic), generators, `yield`, `yield from`
- f-strings, walrus operator, augmented assignment
- Pattern matching: literals, captures, OR, sequences, mappings, classes, guards
- Most built-in functions: `len`, `range`, `print`, `type`, `int`, `float`, `str`, `list`, `dict`, `set`, `tuple`, `open`, `sorted`, `enumerate`, `zip`, `map`, `filter`, `pow`, `round`, `abs`, `chr`, `ord`, `hex`, `oct`, `bin`, `eval`, `exec`, `compile`, `dir`, `vars`, `isinstance`, `issubclass`, `super`, `slice`, `ascii`, `repr`
- Stdlib modules: `math`, `sys`, `os`, `builtins`

### Tier 2 — Missing (needs implementation)
- Type annotations (syntax + storage, not runtime enforcement)
- Keyword-only arguments: `def f(*, a, b)`
- Positional-only arguments: `def f(a, /, b)`
- Starred assignment: `a, *b = range(10)`
- For-loop tuple unpacking: `for a, b in list_of_pairs:`
- Multiple except types: `except (A, B):`
- Exception chaining: `raise X from Y`
- Bare `raise` (re-raise)
- Multiple context managers: `with a, b:`
- `async for` / `async with`
- Proper C3 linearization for MRO

### Tier 3 — Would-be-nice
- `__slots__`
- Generator `.throw()` / `.close()`
- Relative imports
- Complete `__format__` on all types
- Full exception hierarchy
- `memoryview`
- Match keyword patterns: `case Point(x=1):`

## Success Criteria
1. All 405+ compat tests pass (done)
2. All 259 unit tests pass (done)
3. Tier 2 features implemented
4. Can run a non-trivial Python script (> 100 LOC) without hitting unimplemented features
