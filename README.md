# RustPy

A lightweight, correct **Python 3.10+ interpreter written in Rust** from scratch.

## Features

- Full four-stage pipeline: Lexer → Parser → Compiler → VM
- 28 object types, 60+ opcode variants
- Pattern matching (`match`/`case`), async/await, generators
- Classes, inheritance, `@property`, `@staticmethod`, `@classmethod`
- 62+ built-in functions, stdlib modules (`math`, `sys`, `os`)
- Zero `unsafe` in production code
- 695+ tests with 100% CPython output parity

## Install

```bash
cargo install rustpy
```

## Usage

```bash
# Run a Python file
rustpy script.py

# Start the REPL
rustpy
```

## Build from source

```bash
git clone https://github.com/sumu/rustpy
cd rustpy
cargo build --release
./target/release/rustpy
```

## Run tests

```bash
cargo test
python3 tests/compat_test.py
python3 tests/run_all_tests.py
```

## License

MIT
