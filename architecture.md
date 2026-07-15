# RustPy — Architecture

## Pipeline

```
Source Code
    │
    ▼
┌─────────────┐
│   Lexer     │  src/lexer/mod.rs — Produces Vec<Token>
│             │  tokens.rs — TokenKind enum (66+ token types)
└─────────────┘
    │
    ▼
┌─────────────┐
│   Parser    │  src/parser/mod.rs — Recursive descent
│             │  Produces ast::Module { body: Vec<Stmt> }
└─────────────┘
    │
    ▼
┌─────────────┐
│  Compiler   │  src/compiler/mod.rs — Walks AST, emits
│             │  CodeObject { instructions: Vec<Opcode> }
└─────────────┘
    │           │  code.rs — CodeObject struct
    │           │  opcodes.rs — 60+ opcode variants
    ▼
┌─────────────┐
│     VM      │  src/vm/mod.rs — Stack-based execution
│             │  frame.rs — Frame { stack, locals, blocks }
└─────────────┘
    │
    ▼
┌─────────────┐
│  Runtime    │  src/runtime/mod.rs — Environment
│             │  Nested scopes (parent pointers)
└─────────────┘
    │
    ▼
┌─────────────┐
│  Objects    │  src/objects/ — 28 Rust types implementing
│             │  the PyObject trait
└─────────────┘
```

## Component Details

### Lexer (`src/lexer/`)
- Character-by-character iterator
- Tracks `indent_stack` for INDENT/DEDENT tokens
- Skips whitespace/comments, handles string escapes
- Produces tokens with span info for error reporting

### Parser (`src/parser/`)
- Recursive descent, one-token lookahead
- Operator precedence via precedence climbing
- `parse_suite()` handles both indented blocks and single-line `:` bodies
- Produces `ast::Module` (see `src/ast/mod.rs` for full AST enum)

### Compiler (`src/compiler/`)
- Walks the AST, emits `Opcode` instructions into a `CodeObject`
- Child compiler for nested scopes (function bodies, comprehensions, generators)
- Manages names list (variable names) and constants pool
- Handles scope: Local vs Global vs Cell (for closures)
- `Opcode` enum has 60+ variants (see `opcodes.rs`)

### VM (`src/vm/`)
- Stack-based bytecode interpreter
- `Frame` holds operand stack, locals, block stack (for try/finally/with/loop)
- `execute_opcode()` — large match on `Opcode` variants
- `invoke_inner()` — function call setup: binds positional/kwargs/vararg/kwarg
- `call_function()` — method dispatch: decides between native, RustPy function, class, etc.

### Objects (`src/objects/`)
- `PyObject` trait: the core abstraction
  - `get_type()` → type name string
  - `repr()`, `str()` → string representations
  - `add()`, `sub()`, `mul()`, ... → operator dispatch (returns `None` for unsupported)
  - `eq()`, `lt()`, `contains()`, `hash()` → comparisons
  - `get_attr()`, `set_attr()`, `del_attr()` → attribute access
  - `get_item()`, `del_item()` → subscript access
  - `get_iter()`, `get_next()` → iteration protocol
- 28 object types: bool, int, float, complex, string, bytes, bytearray, list, tuple, dict, set, frozenset, range, slice, NoneType, EllipsisType, function, native_function, bound_method, instance, class, module, generator, coroutine, property, staticmethod, classmethod, exception, file, map

### Runtime (`src/runtime/`)
- `Environment` — linked-list scope chain
- `set()`, `get()`, `remove()` — variable operations
- `get_root()`, `set_root()` — for `global` keyword support

### Stdlib (`src/stdlib/`)
- `builtins.rs` — 62+ built-in functions implemented as Rust-native functions
- `math.rs` — math module (sqrt, sin, cos, factorial, pi, etc.)
- `sys.rs` — sys module (argv, path, modules, exit, version)
- `os.rs` — os module (getcwd, listdir, etc.)
- `import.rs` — filesystem import resolution

## Data Flow Example

```python
def f(x): return x + 1
```

1. **Lexer** → tokens: `Def`, `Identifier("f")`, `LParen`, `Identifier("x")`, `RParen`, `Colon`, `Newline`, `Indent`, `Return`, `Identifier("x")`, `Plus`, `Int(1)`, `Newline`, `Dedent`, `EOF`
2. **Parser** → `Stmt::FunctionDef { name: "f", params: ["x"], body: [Stmt::Return { value: Some(BinOp { left: Identifier("x"), op: Add, right: IntLiteral("1") }) }], ... }`
3. **Compiler** → `CodeObject { instructions: [LoadConst(0), LoadName(0), BinaryAdd, ReturnValue], names: ["x", ...], constants: [PyInt(1), ...], arg_count: 1, ... }`
4. **VM** → `MakeFunction` wraps the code object in a `PyFunction`; calling it creates a frame, binds `x` to argument, executes the instructions
