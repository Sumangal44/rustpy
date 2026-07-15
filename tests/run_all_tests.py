#!/usr/bin/env python3
import subprocess
import sys

def run_step(name, cmd):
    print(f"\n============================================================\nRunning: {name}\nCommand: {' '.join(cmd)}\n============================================================")
    res = subprocess.run(cmd, text=True)
    if res.returncode != 0:
        print(f"❌ {name} FAILED! (exit code: {res.returncode})")
        return False
    print(f"✅ {name} PASSED!")
    return True

def main():
    steps = [
        ("Rust Unit Tests (cargo test)", ["cargo", "test"]),
        ("Python Compatibility Tests", [sys.executable, "tests/compat_test.py"]),
        ("Python Tier 3 Feature Tests", [sys.executable, "tests/tier3_test.py"]),
        ("CPython vs RustPy Benchmarking/Correctness", [sys.executable, "tests/cpython_comparison_tester.py"]),
        ("Custom Print Arguments Parity", [sys.executable, "tests/cpython_tester.py", "tests/print_test.py"]),
        ("Custom Function Arguments Parity", [sys.executable, "tests/cpython_tester.py", "tests/func_args_test.py"]),
        ("Custom Closures and Scoping Parity", [sys.executable, "tests/cpython_tester.py", "tests/func_features_test.py"]),
        ("Custom Lambdas Parity", [sys.executable, "tests/cpython_tester.py", "tests/lambda_test.py"]),
        ("Custom Built-in Functions Parity", [sys.executable, "tests/cpython_tester.py", "tests/builtins_test.py"]),
        ("Custom Conditionals Parity", [sys.executable, "tests/cpython_tester.py", "tests/conditionals_test.py"]),
        ("Custom Loop Structures Parity", [sys.executable, "tests/cpython_tester.py", "tests/loops_test.py"]),
        ("Custom Loops + Conditionals Integration Parity", [sys.executable, "tests/cpython_tester.py", "tests/loops_conditionals_test.py"]),
    ]

    all_passed = True
    failed_steps = []

    for name, cmd in steps:
        if not run_step(name, cmd):
            all_passed = False
            failed_steps.append(name)

    print("\n" + "=" * 60)
    print("FINAL TEST EXECUTION SUMMARY")
    print("=" * 60)
    if all_passed:
        print("🎉 ALL TEST SUITES PASSED SUCCESSFULLY!")
        sys.exit(0)
    else:
        print(f"❌ SOME TEST SUITES FAILED:")
        for step in failed_steps:
            print(f"   - {step}")
        sys.exit(1)

if __name__ == "__main__":
    main()
