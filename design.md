# RustPy — Design Decisions

## Why a bytecode VM instead of AST-walking?

An AST walker is simpler but couples interpretation with parsing. The bytecode VM:
- Separates compilation from execution (can pre-compile, cache)
- Makes control flow explicit (jump targets, block stack)
- Enables future optimizations (peephole, constant folding)
- Matches CPython's architecture, making the mental model transferable

## Why `Rc<dyn PyObject>` and not an enum?

Two approaches for dynamic typing in Rust:
1. **Enum**: `enum PyObject { Int(i64), Float(f64), Str(String), ... }`
2. **Trait object**: `Rc<dyn PyObject>`

We chose trait objects because:
- Adding a new type doesn't require editing every match on the enum
- Traits model the protocol-based dispatch of Python better
- `dyn PyObject` matches Python's "everything is an object" philosophy
- Downside: heap allocation for every value, dynamic dispatch overhead
- The enum approach would be faster but less extensible

## Why operator methods return `Option`?

Python's data model tries `a.__add__(b)` first, then `b.__radd__(a)` if the first returns `NotImplemented`. In RustPy:
- Methods return `Option<Rc<dyn PyObject>>` — `None` means "I don't support this"
- The VM handles the fallback logic: if `a.add(b)` returns `None`, try `b.radd(a)`
- This avoids a special `NotImplemented` sentinel object

## Why `Vec<Opcode>` instead of bytecode encoding?

Each `Opcode` is a Rust enum variant, which is larger than bytecode bytes but:
- Easier to debug (enums show their variant name)
- No manual encoding/decoding
- Safe (no invalid opcodes at runtime)
- The JIT/unrolling benefit of bytecode is not relevant at this scale

## Why single-token lookahead in the parser?

- Recursive descent with one-token lookahead is the simplest correct parser
- Python's grammar is designed for LL(1) parsing
- We avoid parser generators (peg, lalrpop) to minimize dependencies and keep the build simple

## Error handling strategy

- All errors propagate via `Result<T, String>` in the VM
- The parser has a `ParseError` type with span info
- `RuntimeError: ...` prefixes runtime errors (matching CPython's style somewhat)
- No panic/unwrap in production paths (only in test code)
- The compat test suite validates error messages are close to CPython

## Why not use `num-bigint`?

Actually we DO use `num-bigint` — arbitrary precision integers are required for Python's big ints. The `BigInt` type from `num-bigint` handles this. The only other dependency is `num-traits`.

## Memory management

- `Rc<RefCell<>>` for shared mutable state
- `Rc` reference counting (no cycle detection)
- Potential leak: cycles in object graphs (e.g., self-referential containers) won't be collected
- This is acceptable for an educational interpreter

## Why no C3 linearization yet?

The simple MRO (depth-first, left-to-right) works for basic single-inheritance patterns. C3 linearization is needed for correct diamond inheritance resolution (common in complex class hierarchies). This is a known gap tracked in `phases.md`.
