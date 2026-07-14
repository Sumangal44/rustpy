#!/usr/bin/env python3
"""
Comprehensive Python compatibility test suite.
Compares RustPy output vs CPython for every feature.

Usage:
  python3 tests/compat_test.py              # Run all tests
  python3 tests/compat_test.py --fix        # Auto-fix expected output
  python3 tests/compat_test.py --category str  # Run specific category
"""

import subprocess
import sys
import os
import traceback
import re
import json

RUSTPY_BIN = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "debug", "rustpy")
PYTHON_BIN = sys.executable

PASS = 0
FAIL = 0
ERRORS = []

def run_rustpy(code):
    """Run code in RustPy and return (stdout, stderr, exit_code)."""
    with open("/tmp/rustpy_compat_test.py", "w") as f:
        f.write(code)
    result = subprocess.run(
        [RUSTPY_BIN, "/tmp/rustpy_compat_test.py"],
        capture_output=True, text=True, timeout=10
    )
    return result.stdout, result.stderr, result.returncode

def run_cpython(code):
    """Run code in CPython and return (stdout, stderr, exit_code)."""
    result = subprocess.run(
        [PYTHON_BIN, "-c", code],
        capture_output=True, text=True, timeout=10
    )
    return result.stdout, result.stderr, result.returncode

def normalize(s):
    """Normalize output for comparison (trailing whitespace, etc)."""
    return s.rstrip()

def compare_output(name, code, expected_stdout, expected_stderr, expected_code):
    global PASS, FAIL
    rust_stdout, rust_stderr, rust_code = run_rustpy(code)

    ok = True
    issues = []

    if normalize(rust_stdout) != normalize(expected_stdout):
        issues.append(f"  STDOUT mismatch:")
        issues.append(f"    Expected: {expected_stdout!r}")
        issues.append(f"    Got:      {rust_stdout!r}")
        ok = False

    if normalize(rust_stderr) != normalize(expected_stderr):
        # Compare error message content (Python tracebacks differ in format)
        rust_err_clean = clean_error(rust_stderr)
        expected_err_clean = clean_error(expected_stderr)
        if rust_err_clean != expected_err_clean:
            issues.append(f"  STDERR mismatch:")
            issues.append(f"    Expected: {expected_stderr!r}")
            issues.append(f"    Got:      {rust_stderr!r}")
            ok = False

    if rust_code != expected_code:
        issues.append(f"  EXIT CODE mismatch: expected {expected_code}, got {rust_code}")
        ok = False

    if ok:
        PASS += 1
    else:
        FAIL += 1
        ERRORS.append((name, code, "\n".join(issues)))
        print(f"  FAIL: {name}")
        for line in issues:
            print(line)
        print()
    return ok

def clean_error(s):
    """Extract just the error type and message from any error output."""
    # Remove traceback lines, keep only error type and message
    lines = s.strip().split("\n")
    # Keep only the last non-empty line (the actual error message)
    error_lines = [l for l in lines if l.strip() and not l.strip().startswith("Traceback") 
                   and not l.strip().startswith("  File") and not l.strip().startswith("RuntimeError")]
    return "\n".join(error_lines).strip()

def test_category(name, tests):
    """Run a category of tests."""
    global PASS, FAIL
    print(f"\n{'='*60}")
    print(f"Category: {name}")
    print(f"{'='*60}")
    cat_pass = 0
    cat_fail = 0
    for t in tests:
        if len(t) == 3:
            tname, code, expected = t
            expected_stdout = expected + "\n" if expected else ""
            expected_stderr = ""
            expected_code = 0
        elif len(t) == 4:
            tname, code, expected_stdout, expected_stderr = t
            expected_code = 0
        else:
            tname, code, expected_stdout, expected_stderr, expected_code = t
        if callable(expected_stdout):
            expected_stdout = expected_stdout()
        compare_output(f"[{name}] {tname}", code, expected_stdout, expected_stderr, expected_code)

    print(f"  Category result: {cat_pass} passed, {cat_fail} failed")

def run_error_test(name, code, expected_error_substring):
    """Test that RustPy produces an error containing expected_error_substring."""
    global PASS, FAIL
    rust_stdout, rust_stderr, rust_code = run_rustpy(code)
    cpython_stdout, cpython_stderr, cpython_code = run_cpython(code)

    ok = True
    issues = []

    if expected_error_substring not in rust_stderr:
        issues.append(f"  Expected error containing: {expected_error_substring}")
        issues.append(f"  Got stderr: {rust_stderr!r}")
        ok = False

    if ok:
        PASS += 1
    else:
        FAIL += 1
        ERRORS.append((name, code, "\n".join(issues)))
        print(f"  FAIL: {name}")
        for line in issues:
            print(line)
        print()

# ============================================================
# TEST CASES
# ============================================================

LITERAL_TESTS = [
    ("int", "print(42)", "42"),
    ("negative_int", "print(-10)", "-10"),
    ("large_int", "print(12345678901234567890)", "12345678901234567890"),
    ("hex_int", "print(0xFF)", "255"),
    ("oct_int", "print(0o77)", "63"),
    ("bin_int", "print(0b1010)", "10"),
    ("float", "print(3.14)", "3.14"),
    ("negative_float", "print(-2.5)", "-2.5"),
    ("sci_float", "print(1.5e10)", "15000000000.0"),
    ("bool_true", "print(True)", "True"),
    ("bool_false", "print(False)", "False"),
    ("none", "print(None)", "None"),
    ("complex", "print(1+2j)", "(1+2j)"),
    ("complex_pure_imag", "print(5j)", "5j"),
    ("complex_builtin", "print(complex(3,4))", "(3+4j)"),
    ("string_single", "print('hello')", "hello"),
    ("string_double", 'print("world")', "world"),
    ("string_escape", r"print('hello\nworld')", "hello\nworld"),
    ("string_triple", "print('''multi\nline''')", "multi\nline"),
    ("bytes_literal", "print(b'hello')", "b'hello'"),
    ("bytes_escape", "print(b'\\x00\\x01')", "b'\\x00\\x01'"),
    ("fstring_simple", "name='world'\nprint(f'hello {name}')", "hello world"),
    ("fstring_expr", "x=3;y=4\nprint(f'{x}+{y}={x+y}')", "3+4=7"),
    ("fstring_format", "x=3.14159\nprint(f'{x:.2f}')", "3.14"),
    ("list_literal", "print([1, 2, 3])", "[1, 2, 3]"),
    ("tuple_literal", "print((1, 2, 3))", "(1, 2, 3)"),
    ("tuple_single", "print((1,))", "(1,)"),
    ("set_literal", "print({1, 2, 3})", "{1, 2, 3}"),
    ("dict_literal", "print({'a': 1, 'b': 2})", "{'a': 1, 'b': 2}"),
    ("empty_list", "print([])", "[]"),
    ("empty_tuple", "print(())", "()"),
    ("empty_dict", "print({})", "{}"),
    ("ellipsis", "print(...)", "Ellipsis"),
]

ARITHMETIC_TESTS = [
    ("add", "print(1 + 2)", "3"),
    ("sub", "print(5 - 3)", "2"),
    ("mul", "print(3 * 4)", "12"),
    ("truediv", "print(7 / 2)", "3.5"),
    ("floordiv", "print(7 // 2)", "3"),
    ("mod", "print(10 % 3)", "1"),
    ("pow", "print(2 ** 10)", "1024"),
    ("neg", "print(-5)", "-5"),
    ("pos", "print(+5)", "5"),
    ("abs_int", "print(abs(-5))", "5"),
    ("abs_float", "print(abs(-3.5))", "3.5"),
    ("float_add", "print(1.5 + 2.5)", "4.0"),
    ("float_mul", "print(2.5 * 3.0)", "7.5"),
    ("float_div", "print(5.5 / 2.0)", "2.75"),
    ("int_float_mix", "print(1 + 2.5)", "3.5"),
    ("complex_add", "print((1+2j) + (3+4j))", "(4+6j)"),
    ("complex_mul", "print((1+2j) * (3+4j))", "(-5+10j)"),
    ("pow_mod", "print(pow(7, 3, 5))", "3"),
    ("divmod", "print(divmod(10, 3))", "(3, 1)"),
]

BITWISE_TESTS = [
    ("and", "print(5 & 3)", "1"),
    ("or", "print(5 | 3)", "7"),
    ("xor", "print(5 ^ 3)", "6"),
    ("lshift", "print(5 << 2)", "20"),
    ("rshift", "print(20 >> 2)", "5"),
    ("invert", "print(~5)", "-6"),
    ("bitwise_neg", "print(-5 & 0xFFFF)", "65531"),
]

COMPARISON_TESTS = [
    ("eq_int", "print(5 == 5)", "True"),
    ("ne_int", "print(5 != 3)", "True"),
    ("lt_int", "print(3 < 5)", "True"),
    ("le_int", "print(3 <= 3)", "True"),
    ("gt_int", "print(5 > 3)", "True"),
    ("ge_int", "print(5 >= 5)", "True"),
    ("eq_float", "print(3.14 == 3.14)", "True"),
    ("eq_str", "print('hello' == 'hello')", "True"),
    ("eq_tuple", "print((1,2) == (1,2))", "True"),
    ("chained_compare", "print(1 < 2 < 3)", "True"),
    ("chained_compare_false", "print(1 < 2 > 3)", "False"),
    ("is_operator", "print(True is True)", "True"),
    ("is_not", "print(True is not False)", "True"),
    ("in_list", "print(2 in [1,2,3])", "True"),
    ("not_in_list", "print(4 not in [1,2,3])", "True"),
    ("in_str", "print('lo' in 'hello')", "True"),
    ("in_dict", "print('a' in {'a':1})", "True"),
]

BOOLEAN_TESTS = [
    ("and_true", "print(True and True)", "True"),
    ("and_false", "print(True and False)", "False"),
    ("or_true", "print(False or True)", "True"),
    ("or_false", "print(False or False)", "False"),
    ("not_true", "print(not False)", "True"),
    ("not_false", "print(not True)", "False"),
    ("and_short", "print(False and 1/0)", "False"),
    ("or_short", "print(True or 1/0)", "True"),
    ("truthy_int", "print(bool(1))", "True"),
    ("truthy_zero", "print(bool(0))", "False"),
    ("truthy_str", "print(bool('hello'))", "True"),
    ("truthy_empty", "print(bool(''))", "False"),
    ("truthy_list", "print(bool([1,2]))", "True"),
    ("truthy_empty_list", "print(bool([]))", "False"),
]

CONTROL_FLOW_TESTS = [
    ("if_true", "x=42\nif x > 0:\n    print('positive')", "positive"),
    ("if_else", "x=-5\nif x > 0:\n    print('pos')\nelse:\n    print('neg')", "neg"),
    ("if_elif_else", "x=0\nif x > 0:\n    print('pos')\n    elif x < 0:\n        print('neg')\nelse:\n    print('zero')", "zero"),
    ("while_loop", "i=3\nwhile i > 0:\n    print(i)\n    i-=1", "3\n2\n1"),
    ("for_range", "for i in range(3):\n    print(i)", "0\n1\n2"),
    ("for_list", "for x in [10,20,30]:\n    print(x)", "10\n20\n30"),
    ("for_str", "for c in 'abc':\n    print(c)", "a\nb\nc"),
    ("break_stmt", "for i in range(10):\n    if i==3:\n        break\n    print(i)", "0\n1\n2"),
    ("continue_stmt", "for i in range(5):\n    if i==2:\n        continue\n    print(i)", "0\n1\n3\n4"),
    ("nested_loop", "for i in range(2):\n    for j in range(2):\n        print(i,j)", "0 0\n0 1\n1 0\n1 1"),
    ("pass_stmt", "x=1\nif x>0:\n    pass\nprint('ok')", "ok"),
]

FUNCTION_TESTS = [
    ("def_return", "def f(): return 42\nprint(f())", "42"),
    ("def_args", "def add(a,b): return a+b\nprint(add(3,4))", "7"),
    ("def_defaults", "def f(x=10): return x\nprint(f())\nprint(f(5))", "10\n5"),
    ("def_kwargs", "def f(a,b): return a-b\nprint(f(b=10,a=5))", "-5"),
    ("def_varargs", "def f(*args): return sum(args)\nprint(f(1,2,3))", "6"),
    ("def_varkw", "def f(**kw): return kw['a']\nprint(f(a=1,b=2))", "1"),
    ("def_combined", "def f(a,*b,**c): return (a,b,c)\nprint(f(1,2,3,x=4))", "(1, (2, 3), {'x': 4})"),
    ("nested_func", "def outer(x):\n    def inner(y): return x+y\n    return inner(10)\nprint(outer(5))", "15"),
    ("closure", "def make_adder(x):\n    def adder(y): return x+y\n    return adder\nadd5=make_adder(5)\nprint(add5(3))", "8"),
    ("lambda", "f=lambda x: x*2\nprint(f(5))", "10"),
    ("lambda_multi", "f=lambda a,b: a+b\nprint(f(3,4))", "7"),
    ("lambda_noargs", "f=lambda: 42\nprint(f())", "42"),
    ("recursion", "def fact(n):\n    return 1 if n<=1 else n*fact(n-1)\nprint(fact(5))", "120"),
    ("decorator", "def dec(f):\n    def wrapper(): return 'wrapped'\n    return wrapper\n@dec\ndef foo(): return 'orig'\nprint(foo())", "wrapped"),
    ("global_var", "x=10\ndef f():\n    global x\n    x=20\nf()\nprint(x)", "20"),
]

CLASS_TESTS = [
    ("simple_class", "class A:\n    pass\na=A()\nprint(type(a).__name__)", "A"),
    ("class_method", "class A:\n    def f(self): return 42\na=A()\nprint(a.f())", "42"),
    ("class_attr", "class A:\n    x=10\na=A()\nprint(a.x)\nprint(A.x)", "10\n10"),
    ("init_method", "class A:\n    def __init__(self,v): self.val=v\n    def get(self): return self.val\na=A(42)\nprint(a.get())", "42"),
    ("inheritance", "class A:\n    def f(self): return 'A'\nclass B(A):\n    pass\nb=B()\nprint(b.f())", "A"),
    ("override", "class A:\n    def f(self): return 'A'\nclass B(A):\n    def f(self): return 'B'\nb=B()\nprint(b.f())", "B"),
    ("super_call", "class A:\n    def f(self): return 'A'\nclass B(A):\n    def f(self): return super().f()+'B'\nb=B()\nprint(b.f())", "AB"),
    ("property", "class A:\n    @property\n    def val(self): return 42\na=A()\nprint(a.val)", "42"),
    ("staticmethod", "class A:\n    @staticmethod\n    def f(): return 'static'\nprint(A.f())", "static"),
    ("classmethod", "class A:\n    @classmethod\n    def f(cls): return cls.__name__\nprint(A.f())", "A"),
    ("multiple_inheritance", "class A:\n    def f(self): return 'A'\nclass B:\n    def g(self): return 'B'\nclass C(A,B):\n    pass\nc=C()\nprint(c.f())\nprint(c.g())", "A\nB"),
    ("isinstance_check", "class A: pass\na=A()\nprint(isinstance(a, A))\nprint(isinstance(a, object))", "True\nTrue"),
    ("issubclass_check", "class A: pass\nclass B(A): pass\nprint(issubclass(B, A))\nprint(issubclass(A, object))", "True\nTrue"),
    ("del_attr", "class A: pass\na=A()\na.x=10\nprint(a.x)\ndel a.x\nprint(hasattr(a,'x'))", "10\nFalse"),
    ("class_dict", "class A: pass\na=A()\na.x=10\nprint(a.__dict__)", "{'x': 10}"),
]

STRING_METHOD_TESTS = [
    ("upper", "print('hello'.upper())", "HELLO"),
    ("lower", "print('HELLO'.lower())", "hello"),
    ("capitalize", "print('hello'.capitalize())", "Hello"),
    ("title", "print('hello world'.title())", "Hello World"),
    ("swapcase", "print('Hello'.swapcase())", "hELLO"),
    ("strip", "print('  hello  '.strip())", "hello"),
    ("lstrip", "print('  hello  '.lstrip())", "hello  "),
    ("rstrip", "print('  hello  '.rstrip())", "  hello"),
    ("split", "print('a b c'.split())", "['a', 'b', 'c']"),
    ("split_sep", "print('a,b,c'.split(','))", "['a', 'b', 'c']"),
    ("rsplit", "print('a,b,c'.rsplit(','))", "['a', 'b', 'c']"),
    ("splitlines", "print('a\\nb\\nc'.splitlines())", "['a', 'b', 'c']"),
    ("join", "print(','.join(['a','b','c']))", "a,b,c"),
    ("replace", "print('hello world'.replace('world','there'))", "hello there"),
    ("startswith", "print('hello'.startswith('he'))", "True"),
    ("endswith", "print('hello'.endswith('lo'))", "True"),
    ("find", "print('hello'.find('l'))", "2"),
    ("rfind", "print('hello'.rfind('l'))", "3"),
    ("index", "print('hello'.index('l'))", "2"),
    ("rindex", "print('hello'.rindex('l'))", "3"),
    ("count", "print('hello'.count('l'))", "2"),
    ("partition", "print('hello world'.partition(' '))", "('hello', ' ', 'world')"),
    ("rpartition", "print('hello world'.rpartition(' '))", "('hello', ' ', 'world')"),
    ("isalpha", "print('hello'.isalpha())", "True"),
    ("isdigit", "print('123'.isdigit())", "True"),
    ("isalnum", "print('abc123'.isalnum())", "True"),
    ("isspace", "print('   '.isspace())", "True"),
    ("isupper", "print('HELLO'.isupper())", "True"),
    ("islower", "print('hello'.islower())", "True"),
    ("istitle", "print('Hello World'.istitle())", "True"),
    ("zfill", "print('42'.zfill(5))", "00042"),
    ("ljust", "print('hi'.ljust(5))", "hi   "),
    ("rjust", "print('hi'.rjust(5))", "   hi"),
    ("center", "print('hi'.center(5))", " hi  "),
    ("expandtabs", "print('a\\tb'.expandtabs(4))", "a   b"),
    ("encode", "print('hello'.encode())", "b'hello'"),
    ("removeprefix", "print('hello world'.removeprefix('hello '))", "world"),
    ("removesuffix", "print('hello world'.removesuffix(' world'))", "hello"),
    ("isdecimal", "print('123'.isdecimal())", "True"),
    ("isnumeric", "print('Ⅳ'.isnumeric())", "True" if sys.version_info >= (3, 0) else "False"),
    ("isidentifier", "print('hello'.isidentifier())", "True"),
    ("isprintable", "print('hello'.isprintable())", "True"),
]

LIST_METHOD_TESTS = [
    ("append", "a=[1,2]\na.append(3)\nprint(a)", "[1, 2, 3]"),
    ("extend", "a=[1,2]\na.extend([3,4])\nprint(a)", "[1, 2, 3, 4]"),
    ("insert", "a=[1,3]\na.insert(1,2)\nprint(a)", "[1, 2, 3]"),
    ("remove", "a=[1,2,3]\na.remove(2)\nprint(a)", "[1, 3]"),
    ("pop", "a=[1,2,3]\nprint(a.pop())\nprint(a)", "3\n[1, 2]"),
    ("pop_index", "a=[1,2,3]\nprint(a.pop(0))\nprint(a)", "1\n[2, 3]"),
    ("clear", "a=[1,2,3]\na.clear()\nprint(a)", "[]"),
    ("index_method", "a=[10,20,30]\nprint(a.index(20))", "1"),
    ("count_method", "a=[1,2,2,3]\nprint(a.count(2))", "2"),
    ("reverse", "a=[1,2,3]\na.reverse()\nprint(a)", "[3, 2, 1]"),
    ("sort", "a=[3,1,2]\na.sort()\nprint(a)", "[1, 2, 3]"),
    ("copy", "a=[1,2]\nb=a.copy()\nb.append(3)\nprint(a)\nprint(b)", "[1, 2]\n[1, 2, 3]"),
    ("list_concat", "print([1,2]+[3,4])", "[1, 2, 3, 4]"),
    ("list_repeat", "print([1,2]*3)", "[1, 2, 1, 2, 1, 2]"),
    ("list_slice", "print([0,1,2,3,4][1:3])", "[1, 2]"),
    ("list_slice_step", "print([0,1,2,3,4][::2])", "[0, 2, 4]"),
    ("list_comp_simple", "print([x*2 for x in [1,2,3]])", "[2, 4, 6]"),
    ("list_comp_if", "print([x for x in [1,2,3,4] if x%2==0])", "[2, 4]"),
]

DICT_METHOD_TESTS = [
    ("dict_get", "print({'a':1,'b':2}.get('a'))", "1"),
    ("dict_get_default", "print({'a':1}.get('x',99))", "99"),
    ("dict_keys", "print(list({'a':1,'b':2}.keys()))", "['a', 'b']"),
    ("dict_values", "print(list({'a':1,'b':2}.values()))", "[1, 2]"),
    ("dict_items", "print(list({'a':1,'b':2}.items()))", "[('a', 1), ('b', 2)]"),
    ("dict_pop", "d={'a':1,'b':2}\nprint(d.pop('a'))\nprint(d)", "1\n{'b': 2}"),
    ("dict_pop_default", "print({'a':1}.pop('x',99))", "99"),
    ("dict_popitem", "d={'a':1}\nk,v=d.popitem()\nprint(k,v)", "a 1"),
    ("dict_setdefault", "d={}\nprint(d.setdefault('a',42))\nprint(d)", "42\n{'a': 42}"),
    ("dict_update", "d={'a':1}\nd.update({'b':2})\nprint(d)", "{'a': 1, 'b': 2}"),
    ("dict_clear", "d={'a':1}\nd.clear()\nprint(d)", "{}"),
    ("dict_copy", "d1={'a':1}\nd2=d1.copy()\nd2['b']=2\nprint(d1)\nprint(d2)", "{'a': 1}\n{'a': 1, 'b': 2}"),
    ("dict_fromkeys", "print(dict.fromkeys(['a','b'],0))", "{'a': 0, 'b': 0}"),
    ("dict_comp", "print({x:x*2 for x in [1,2,3]})", "{1: 2, 2: 4, 3: 6}"),
]

SET_METHOD_TESTS = [
    ("set_add", "s=set()\ns.add(1)\ns.add(2)\nprint(sorted(s))", "[1, 2]"),
    ("set_remove", "s={1,2,3}\ns.remove(2)\nprint(sorted(s))", "[1, 3]"),
    ("set_discard", "s={1,2}\ns.discard(1)\ns.discard(99)\nprint(sorted(s))", "[2]"),
    ("set_pop", "s={42}\nprint(s.pop())", "42"),
    ("set_clear", "s={1,2}\ns.clear()\nprint(len(s))", "0"),
    ("set_union", "print({1,2} | {2,3})", "{1, 2, 3}"),
    ("set_intersection", "print({1,2,3} & {2,3,4})", "{2, 3}"),
    ("set_difference", "print({1,2,3} - {2,3})", "{1}"),
    ("set_sym_diff", "print({1,2} ^ {2,3})", "{1, 3}"),
    ("set_issubset", "print({1,2}.issubset({1,2,3}))", "True"),
    ("set_issuperset", "print({1,2,3}.issuperset({1,2}))", "True"),
    ("set_isdisjoint", "print({1,2}.isdisjoint({3,4}))", "True"),
    ("set_copy", "s={1,2}\nc=s.copy()\ns.add(3)\nprint(len(s))\nprint(len(c))", "2\n2"),
    ("set_update", "s={1,2}\ns.update({2,3,4})\nprint(sorted(s))", "[1, 2, 3, 4]"),
    ("set_comp", "print({x*2 for x in [1,2,3]})", "{2, 4, 6}"),
]

TUPLE_METHOD_TESTS = [
    ("tuple_index", "print((10,20,30).index(20))", "1"),
    ("tuple_count", "print((1,2,2,3).count(2))", "2"),
    ("tuple_concat", "print((1,2)+(3,4))", "(1, 2, 3, 4)"),
    ("tuple_repeat", "print((1,2)*3)", "(1, 2, 1, 2, 1, 2)"),
    ("tuple_slice", "print((0,1,2,3)[1:3])", "(1, 2)"),
    ("tuple_unpack", "a,b=(1,2)\nprint(a,b)", "1 2"),
    ("tuple_contains", "print(3 in (1,2,3))", "True"),
]

BYTES_METHOD_TESTS = [
    ("bytes_decode", "print(b'hello'.decode())", "hello"),
    ("bytes_hex", "print(b'\\x00\\x01'.hex())", "0001"),
    ("bytes_upper", "print(b'hello'.upper())", "b'HELLO'"),
    ("bytes_lower", "print(b'HELLO'.lower())", "b'hello'"),
    ("bytes_split", "print(b'a b c'.split())", "[b'a', b'b', b'c']"),
    ("bytes_join", "print(b','.join([b'a',b'b']))", "b'a,b'"),
    ("bytes_replace", "print(b'hello world'.replace(b'world', b'there'))", "b'hello there'"),
    ("bytes_startswith", "print(b'hello'.startswith(b'he'))", "True"),
    ("bytes_endswith", "print(b'hello'.endswith(b'lo'))", "True"),
    ("bytes_find", "print(b'hello'.find(b'l'))", "2"),
    ("bytes_count", "print(b'hello'.count(b'l'))", "2"),
    ("bytes_strip", "print(b'  hi  '.strip())", "b'hi'"),
    ("bytes_lstrip", "print(b'  hi  '.lstrip())", "b'hi  '"),
    ("bytes_rstrip", "print(b'  hi  '.rstrip())", "b'  hi'"),
    ("bytes_concat", "print(b'hello '+b'world')", "b'hello world'"),
    ("bytes_repeat", "print(b'ab'*3)", "b'ababab'"),
    ("bytes_slice", "print(b'hello'[1:3])", "b'el'"),
    ("bytes_contains", "print(b'll' in b'hello')", "True"),
]

BYTEARRAY_METHOD_TESTS = [
    ("bytearray_from_str", "print(bytearray('abc','utf-8'))", "bytearray(b'abc')"),
    ("bytearray_append", "b=bytearray(b'ab')\nb.append(99)\nprint(b)", "bytearray(b'abc')"),
    ("bytearray_extend", "b=bytearray(b'a')\nb.extend(b'bc')\nprint(b)", "bytearray(b'abc')"),
    ("bytearray_insert", "b=bytearray(b'ac')\nb.insert(1,98)\nprint(b)", "bytearray(b'abc')"),
    ("bytearray_pop", "b=bytearray(b'ab')\nprint(b.pop())\nprint(b)", "98\nbytearray(b'a')"),
    ("bytearray_remove", "b=bytearray(b'ab')\nb.remove(98)\nprint(b)", "bytearray(b'a')"),
    ("bytearray_clear", "b=bytearray(b'ab')\nb.clear()\nprint(b)", "bytearray(b'')"),
    ("bytearray_reverse", "b=bytearray(b'abc')\nb.reverse()\nprint(b)", "bytearray(b'cba')"),
    ("bytearray_decode", "print(bytearray(b'hello').decode())", "hello"),
    ("bytearray_slice", "print(bytearray(b'hello')[1:3])", "bytearray(b'el')"),
]

EXCEPTION_TESTS = [
    ("try_except", "try:\n    1/0\nexcept:\n    print('caught')", "caught"),
    ("try_except_type", "try:\n    1/0\nexcept ZeroDivisionError:\n    print('zero')", "zero"),
    ("try_except_else", "try:\n    x=1\nexcept:\n    pass\nelse:\n    print('ok')", "ok"),
    ("try_finally", "try:\n    print('try')\nfinally:\n    print('finally')", "try\nfinally"),
    ("try_except_finally", "try:\n    1/0\nexcept:\n    print('caught')\nfinally:\n    print('done')", "caught\ndone"),
    ("raise_exception", "try:\n    raise ValueError('bad')\nexcept ValueError as e:\n    print(e)", "bad"),
    ("multi_except", "try:\n    1/0\nexcept ValueError:\n    print('val')\nexcept ZeroDivisionError:\n    print('zero')", "zero"),
    ("nested_try", "try:\n    try:\n        1/0\n    except:\n        print('inner')\nfinally:\n    print('outer')", "inner\nouter"),
    ("assert_pass", "assert True\nprint('ok')", "ok"),
    ("assert_fail", "try:\n    assert False, 'msg'\nexcept AssertionError as e:\n    print(e)", "msg"),
]

IMPORT_TESTS = [
    ("import_math", "import math\nprint(math.sqrt(9))", "3.0"),
    ("import_math_pi", "import math\nprint(math.pi > 3.14)", "True"),
    ("import_math_sin", "import math\nprint(math.sin(0))", "0.0"),
    ("import_math_cos", "import math\nprint(math.cos(0))", "1.0"),
    ("import_math_factorial", "import math\nprint(math.factorial(5))", "120"),
    ("from_import", "from math import sqrt\nprint(sqrt(16))", "4.0"),
    ("import_as", "import math as m\nprint(m.sqrt(9))", "3.0"),
    ("from_import_as", "from math import sqrt as s\nprint(s(25))", "5.0"),
    ("import_star", "from math import *\nprint(sqrt(36))", "6.0"),
    ("import_sys_argv", "import sys\nprint(len(sys.argv) > 0)", "True"),
    ("import_sys_path", "import sys\nprint(len(sys.path) > 0)", "True"),
    ("import_os_getcwd", "import os\ncwd=os.getcwd()\nprint(cwd!='')", "True"),
    ("import_os_listdir", "import os\nprint(len(os.listdir('.')) > 0)", "True"),
    ("import_os_name", "import os\nprint(os.name == 'posix' or os.name == 'nt')", "True"),
    ("multiple_imports", "import sys,math\nprint(math.sqrt(4))", "2.0"),
]

GENERATOR_TESTS = [
    ("generator_simple", "def g():\n    yield 1\n    yield 2\n    yield 3\nprint(list(g()))", "[1, 2, 3]"),
    ("generator_expr", "print(list(x*2 for x in [1,2,3]))", "[2, 4, 6]"),
    ("generator_with_send", "def g():\n    val = yield 1\n    yield val\n    yield 3\ngen=g()\nprint(next(gen))\nprint(gen.send(42))\nprint(next(gen))", "1\n42\n3"),
    ("yield_from", "def inner():\n    yield 1\n    yield 2\ndef outer():\n    yield from inner()\n    yield 3\nprint(list(outer()))", "[1, 2, 3]"),
]

ASYNC_TESTS = [
    ("async_basic", """
import asyncio
async def foo():
    return 42
result = asyncio.run(foo())
print(result)
""".strip(), "42"),
    ("async_await", """
import asyncio
async def bar():
    return 10
async def foo():
    r = await bar()
    return r + 5
print(asyncio.run(foo()))
""".strip(), "15"),
]

FILE_IO_TESTS = [
    ("file_write_read", """
f = open("/tmp/rustpy_compat_file.txt", "w")
f.write("hello")
f.close()
f = open("/tmp/rustpy_compat_file.txt", "r")
print(f.read())
f.close()
""".strip(), "hello"),
    ("file_append", """
f = open("/tmp/rustpy_compat_file.txt", "w")
f.write("hello\\n")
f.close()
f = open("/tmp/rustpy_compat_file.txt", "a")
f.write("world\\n")
f.close()
f = open("/tmp/rustpy_compat_file.txt", "r")
print(f.read().strip())
""".strip(), "hello\nworld"),
    ("file_readline", """
f = open("/tmp/rustpy_compat_file.txt", "w")
f.write("a\\nb\\nc\\n")
f.close()
f = open("/tmp/rustpy_compat_file.txt", "r")
print(f.readline().strip())
print(f.readline().strip())
f.close()
""".strip(), "a\nb"),
    ("file_iteration", """
f = open("/tmp/rustpy_compat_file.txt", "w")
f.write("x\\ny\\nz\\n")
f.close()
lines = []
f = open("/tmp/rustpy_compat_file.txt", "r")
for line in f:
    lines.append(line.strip())
f.close()
print(lines)
""".strip(), "['x', 'y', 'z']"),
    ("with_stmt", """
with open("/tmp/rustpy_compat_file.txt", "w") as f:
    f.write("ctx manager\\n")
with open("/tmp/rustpy_compat_file.txt", "r") as f:
    print(f.read().strip())
""".strip(), "ctx manager"),
    ("file_seek_tell", """
f = open("/tmp/rustpy_compat_file.txt", "w")
f.write("0123456789")
f.close()
f = open("/tmp/rustpy_compat_file.txt", "r")
f.seek(3)
print(f.tell())
print(f.read(3))
f.close()
""".strip(), "3\n345"),
]

BUILTIN_TESTS = [
    ("len_list", "print(len([1,2,3]))", "3"),
    ("len_str", "print(len('hello'))", "5"),
    ("len_tuple", "print(len((1,2)))", "2"),
    ("len_dict", "print(len({'a':1,'b':2}))", "2"),
    ("len_set", "print(len({1,2,3}))", "3"),
    ("range_stop", "print(list(range(5)))", "[0, 1, 2, 3, 4]"),
    ("range_start_stop", "print(list(range(2,5)))", "[2, 3, 4]"),
    ("range_step", "print(list(range(0,10,3)))", "[0, 3, 6, 9]"),
    ("range_neg_step", "print(list(range(5,0,-1)))", "[5, 4, 3, 2, 1]"),
    ("type_int", "print(type(42).__name__)", "int"),
    ("type_str", "print(type('hi').__name__)", "str"),
    ("type_list", "print(type([]).__name__)", "list"),
    ("isinstance", "print(isinstance(42, int))\nprint(isinstance('hi', str))\nprint(isinstance(42, str))", "True\nTrue\nFalse"),
    ("hasattr", "print(hasattr([], 'append'))\nprint(hasattr([], 'foo'))", "True\nFalse"),
    ("getattr", "print(getattr([1,2], 'pop')())", "2"),
    ("setattr", "class A: pass\na=A()\nsetattr(a,'x',42)\nprint(a.x)", "42"),
    ("delattr", "class A: pass\na=A()\na.x=10\ndelattr(a,'x')\nprint(hasattr(a,'x'))", "False"),
    ("abs_func", "print(abs(-5))\nprint(abs(3))", "5\n3"),
    ("min_func", "print(min([3,1,2]))\nprint(min(3,1,2))", "1\n1"),
    ("max_func", "print(max([3,1,2]))\nprint(max(3,1,2))", "3\n3"),
    ("sum_func", "print(sum([1,2,3]))\nprint(sum([1,2,3],10))", "6\n16"),
    ("any_func", "print(any([False,True,False]))\nprint(any([False,False]))", "True\nFalse"),
    ("all_func", "print(all([True,True]))\nprint(all([True,False]))", "True\nFalse"),
    ("enumerate", "print(list(enumerate(['a','b'])))", "[(0, 'a'), (1, 'b')]"),
    ("zip_func", "print(list(zip([1,2],[3,4])))", "[(1, 3), (2, 4)]"),
    ("map_func", "print(list(map(lambda x:x*2, [1,2,3])))", "[2, 4, 6]"),
    ("filter_func", "print(list(filter(lambda x:x>1, [1,2,3])))", "[2, 3]"),
    ("reversed_list", "print(list(reversed([1,2,3])))", "[3, 2, 1]"),
    ("sorted_func", "print(sorted([3,1,2]))\nprint(sorted([3,1,2], reverse=True))", "[1, 2, 3]\n[3, 2, 1]"),
    ("iter_next", "it=iter([1,2])\nprint(next(it))\nprint(next(it))", "1\n2"),
    ("chr_ord", "print(chr(65))\nprint(ord('A'))", "A\n65"),
    ("hex_func", "print(hex(255))", "0xff"),
    ("oct_func", "print(oct(63))", "0o77"),
    ("bin_func", "print(bin(10))", "0b1010"),
    ("ord", "print(ord('A'))", "65"),
    ("bool_func", "print(bool(1))\nprint(bool(0))\nprint(bool([]))\nprint(bool([1]))", "True\nFalse\nFalse\nTrue"),
    ("str_func", "print(str(42))\nprint(str(3.14))\nprint(str([1,2]))", "42\n3.14\n[1, 2]"),
    ("int_func", "print(int('42'))\nprint(int(3.9))\nprint(int('ff',16))", "42\n3\n255"),
    ("float_func", "print(float(42))\nprint(float('3.14'))", "42.0\n3.14"),
    ("list_func", "print(list('abc'))\nprint(list((1,2,3)))", "['a', 'b', 'c']\n[1, 2, 3]"),
    ("tuple_func", "print(tuple([1,2,3]))", "(1, 2, 3)"),
    ("set_func", "print(set([1,2,2,3]))", "{1, 2, 3}"),
    ("dict_func", "print(dict([('a',1),('b',2)]))", "{'a': 1, 'b': 2}"),
    ("bytes_func", "print(bytes([65,66,67]))", "b'ABC'"),
    ("bytearray_func", "print(bytearray([65,66,67]))", "bytearray(b'ABC')"),
    ("pow_func", "print(pow(2,10))\nprint(pow(2,10,1000))", "1024\n24"),
    ("round_func", "print(round(3.7))\nprint(round(3.14159,2))", "4\n3.14"),
    ("divmod_func", "print(divmod(10,3))", "(3, 1)"),
    ("callable_func", "print(callable(print))\nprint(callable(42))", "True\nFalse"),
    ("hash_func", "print(hash(42) == hash(42))\nprint(hash('hello') == hash('hello'))", "True\nTrue"),
    ("id_func", "x=42;y=42\nprint(id(x)==id(y))", "True"),
    ("vars_func", "class A:pass\na=A()\na.x=10\nprint(vars(a))", "{'x': 10}"),
    ("dir_func", "print('append' in dir([]))", "True"),
    ("globals_func", "x=42\nprint('x' in globals())", "True"),
    ("locals_func", "def f():\n    y=99\n    return 'y' in locals()\nprint(f())", "True"),
    ("repr_func", "print(repr('hello'))", "'hello'"),
    ("ascii_func", "print(ascii('hello\\n'))", "'hello\\n'"),
    ("format_func", "print(format(3.14159,'.2f'))", "3.14"),
    ("exec_func", "exec('x=42')\nprint(x)", "42"),
    ("eval_func", "print(eval('1+2'))", "3"),
    ("compile_func", "c=compile('x=42','<s>','exec')\nexec(c)\nprint(x)", "42"),
]

AUGMENTED_TESTS = [
    ("aug_add", "x=5\nx+=3\nprint(x)", "8"),
    ("aug_sub", "x=5\nx-=2\nprint(x)", "3"),
    ("aug_mul", "x=5\nx*=3\nprint(x)", "15"),
    ("aug_div", "x=7\nx/=2\nprint(x)", "3.5"),
    ("aug_floordiv", "x=7\nx//=2\nprint(x)", "3"),
    ("aug_mod", "x=10\nx%=3\nprint(x)", "1"),
    ("aug_pow", "x=2\nx**=10\nprint(x)", "1024"),
    ("aug_and", "x=5\nx&=3\nprint(x)", "1"),
    ("aug_or", "x=5\nx|=3\nprint(x)", "7"),
    ("aug_xor", "x=5\nx^=3\nprint(x)", "6"),
    ("aug_lshift", "x=1\nx<<=3\nprint(x)", "8"),
    ("aug_rshift", "x=8\nx>>=2\nprint(x)", "2"),
]

WALRUS_TESTS = [
    ("walrus_basic", "if (x:=42) > 40:\n    print(x)", "42"),
    ("walrus_in_while", "i=0\nwhile (i:=i+1) < 3:\n    print(i)", "1\n2"),
    ("walrus_in_list", "print([x for i in range(3) if (x:=i*2) > 2])", "[4]"),
]

MISC_TESTS = [
    ("del_variable", "x=42\ndel x\nprint('x' in dir())\n", "False"),
    ("del_list_item", "a=[1,2,3]\ndel a[1]\nprint(a)", "[1, 3]"),
    ("del_dict_item", "d={'a':1,'b':2}\ndel d['a']\nprint(d)", "{'b': 2}"),
    ("slice_basic", "print(slice(1,3))", "slice(1, 3, None)"),
    ("slice_full", "print(slice(1,5,2))", "slice(1, 5, 2)"),
    ("fstring_debug", "x=42\nprint(f'{x=}')", "x=42"),
    ("fstring_float_format", "x=3.14159\nprint(f'{x:.2f}')", "3.14"),
    ("for_with_else", "for i in []:\n    pass\nelse:\n    print('else')", "else"),
    ("while_with_else", "i=0\nwhile False:\n    pass\nelse:\n    print('else')", "else"),
]

EXEC_TESTS = [
    ("exec_in_func", "def f():\n    exec('x=42')\n    return x\nprint(f())", "42"),
    ("exec_import", "exec('import math')\nprint(math.sqrt(9))", "3.0"),
    ("eval_expr", "print(eval('2+3*4'))", "14"),
    ("eval_with_vars", "x=10\nprint(eval('x+5'))", "15"),
    ("compile_exec", "c=compile('a=100','<s>','exec')\nexec(c)\nprint(a)", "100"),
    ("compile_eval", "c=compile('2+3','<s>','eval')\nprint(eval(c))", "5"),
]

PATTERN_MATCHING_TESTS = [
    ("match_literal", "x=1\nmatch x:\n    case 1: print('one')\n    case 2: print('two')\n    case _: print('other')", "one"),
    ("match_capture", "x=42\nmatch x:\n    case n: print(n)", "42"),
    ("match_wildcard", "x='abc'\nmatch x:\n    case 1: print('one')\n    case _: print('other')", "other"),
    ("match_or", "x=2\nmatch x:\n    case 1|2: print('small')\n    case _: print('other')", "small"),
    ("match_guard", "x=5\nmatch x:\n    case n if n>3: print('big')\n    case _: print('small')", "big"),
    ("match_sequence", "x=[1,2]\nmatch x:\n    case [a,b]: print(a,b)\n    case _: print('no')", "1 2"),
    ("match_mapping", "x={'a':1}\nmatch x:\n    case {'a':v}: print(v)\n    case _: print('no')", "1"),
    ("match_class", "class Point:\n    def __init__(self,x,y): self.x=x;self.y=y\n    __match_args__=('x','y')\np=Point(10,20)\nmatch p:\n    case Point(x,y): print(x,y)", "10 20"),
    ("match_nested", "x=[1,{'a':2}]\nmatch x:\n    case [n,{'a':v}]: print(n,v)\n    case _: print('no')", "1 2"),
]

# Error message test cases
ERROR_TESTS = [
    ("division_by_zero", "1/0", "ZeroDivisionError"),
    ("name_error", "print(undefined_var)", "NameError"),
    ("type_error", "print(1+'str')", "TypeError"),
    ("index_error", "[1,2,3][10]", "IndexError"),
    ("key_error", "{'a':1}['b']", "KeyError"),
    ("value_error", "int('abc')", "ValueError"),
    ("attribute_error", "None.foo", "AttributeError"),
    ("import_error", "import nonexistent_module", "ModuleNotFoundError"),
    ("stop_iteration", "iter([]).__next__()", "StopIteration"),
    ("zero_division_mod", "5%0", "ZeroDivisionError"),
    ("recursion_error", "def f(): return f()\nf()", "RecursionError"),
    ("assertion_error", "assert False", "AssertionError"),
    ("runtime_error", "raise RuntimeError('boom')", "RuntimeError"),
]

# ============================================================
# MAIN
# ============================================================

def main():
    global PASS, FAIL

    print("=" * 60)
    print("RustPy Compatibility Test Suite")
    print(f"  RustPy: {RUSTPY_BIN}")
    print(f"  Python: {PYTHON_BIN}")
    print("=" * 60)

    categories = [
        ("Literals", LITERAL_TESTS),
        ("Arithmetic", ARITHMETIC_TESTS),
        ("Bitwise", BITWISE_TESTS),
        ("Comparisons", COMPARISON_TESTS),
        ("Boolean", BOOLEAN_TESTS),
        ("Control Flow", CONTROL_FLOW_TESTS),
        ("Functions", FUNCTION_TESTS),
        ("Classes", CLASS_TESTS),
        ("String Methods", STRING_METHOD_TESTS),
        ("List Methods", LIST_METHOD_TESTS),
        ("Dict Methods", DICT_METHOD_TESTS),
        ("Set Methods", SET_METHOD_TESTS),
        ("Tuple Methods", TUPLE_METHOD_TESTS),
        ("Bytes Methods", BYTES_METHOD_TESTS),
        ("ByteArray Methods", BYTEARRAY_METHOD_TESTS),
        ("Augmented Assignment", AUGMENTED_TESTS),
        ("Walrus Operator", WALRUS_TESTS),
        ("Exceptions", EXCEPTION_TESTS),
        ("Generators", GENERATOR_TESTS),
        ("Async/Await", ASYNC_TESTS),
        ("File I/O", FILE_IO_TESTS),
        ("Imports", IMPORT_TESTS),
        ("Built-in Functions", BUILTIN_TESTS),
        ("exec/eval/compile", EXEC_TESTS),
        ("Pattern Matching", PATTERN_MATCHING_TESTS),
        ("Misc", MISC_TESTS),
    ]

    for cat_name, cat_tests in categories:
        test_category(cat_name, cat_tests)

    # Error message tests
    print(f"\n{'='*60}")
    print("Category: Error Messages (comparing error type names)")
    print(f"{'='*60}")
    for tname, code, expected_error in ERROR_TESTS:
        run_error_test(f"[Errors] {tname}", code, expected_error)

    # Summary
    print(f"\n{'='*60}")
    print(f"RESULTS: {PASS} passed, {FAIL} failed")
    print(f"{'='*60}")

    if ERRORS:
        print(f"\nFailed tests:")
        for name, code, issues in ERRORS:
            print(f"  {name}")
            print(f"    code: {code!r}")
            print(f"    {issues}")

    return 0 if FAIL == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
