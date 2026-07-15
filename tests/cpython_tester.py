#!/usr/bin/env python3
import sys
import subprocess

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 tests/cpython_tester.py <file.py>")
        sys.exit(1)

    target_file = sys.argv[1]
    rustpy_bin = "./target/release/rustpy"

    print(f"Comparing execution of {target_file}...")

    # Run CPython
    cpython_res = subprocess.run(
        [sys.executable, target_file],
        capture_output=True,
        text=True
    )

    # Run RustPy
    rustpy_res = subprocess.run(
        [rustpy_bin, target_file],
        capture_output=True,
        text=True
    )

    c_stdout = cpython_res.stdout.strip()
    r_stdout = rustpy_res.stdout.strip()
    c_stderr = cpython_res.stderr.strip()
    r_stderr = rustpy_res.stderr.strip()

    print("\n--- CPython Stdout ---")
    print(c_stdout if c_stdout else "<empty>")
    print("\n--- RustPy Stdout ---")
    print(r_stdout if r_stdout else "<empty>")

    match = True

    if c_stdout != r_stdout:
        print("\n❌ STDOUT MISMATCH!")
        match = False

    if cpython_res.returncode != rustpy_res.returncode:
        print(f"\n❌ RETURN CODE MISMATCH! CPython: {cpython_res.returncode}, RustPy: {rustpy_res.returncode}")
        match = False

    # If there is stderr, check if error type name matches (e.g. TypeError, ValueError)
    if c_stderr or r_stderr:
        print("\n--- CPython Stderr ---")
        print(c_stderr if c_stderr else "<empty>")
        print("\n--- RustPy Stderr ---")
        print(r_stderr if r_stderr else "<empty>")
        
        # Check if same exception type is present in both
        c_exc = c_stderr.split("\n")[-1] if c_stderr else ""
        r_exc = r_stderr.split("\n")[-1] if r_stderr else ""
        
        c_type = c_exc.split(":")[0] if ":" in c_exc else c_exc
        r_type = r_exc.split(":")[0] if ":" in r_exc else r_exc
        
        if c_type != r_type:
            print(f"\n❌ EXCEPTION TYPE MISMATCH! CPython: {c_type}, RustPy: {r_type}")
            match = False

    if match:
        print("\n✅ MATCH! Execution output is identical.")
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
