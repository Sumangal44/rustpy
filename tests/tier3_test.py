# Tier 3 feature tests
import subprocess, sys, os, tempfile

rustpy = "./target/debug/rustpy"

def run(code):
    with tempfile.NamedTemporaryFile(suffix=".py", mode="w", delete=False) as f:
        f.write(code)
        fname = f.name
    try:
        r = subprocess.run([rustpy, fname], capture_output=True, text=True, timeout=10)
        return r.stdout.strip(), r.stderr.strip()
    finally:
        os.unlink(fname)

tests = []
def test(name, code, expected_out="", expected_err=""):
    out, err = run(code)
    passed = (out == expected_out.strip()) and (expected_err in err)
    tests.append((name, passed, out, err, expected_out.strip()))
    return passed

# ── 1. str.format() ──────────────────────────────────────────────────────────
test("format_basic_positional",
     "print('Hello {}!'.format('world'))",
     "Hello world!")

test("format_indexed",
     "print('{0} and {1}'.format('a', 'b'))",
     "a and b")

test("format_keyword",
     "print('{name} is {age}'.format(name='Alice', age=30))",
     "Alice is 30")

test("format_float_spec",
     "print('{:.3f}'.format(3.14159))",
     "3.142")

test("format_conversion_r",
     "print('{!r}'.format('hi'))",
     "'hi'")

test("format_align_left",
     "print('{:<10}'.format('hi'))",
     "hi        ")

test("format_align_right",
     "print('{:>10}'.format('hi'))",
     "        hi")

test("format_align_center",
     "print('{:^10}'.format('hi'))",
     "    hi    ")

test("format_escaped_braces",
     "print('{{literal}} {}'.format('braces'))",
     "{literal} braces")

# ── 2. Generator .throw() and .close() ───────────────────────────────────────
test("generator_throw",
     """
def g():
    try:
        yield 1
    except ValueError:
        yield 'caught'
gen = g()
print(next(gen))
print(gen.throw(ValueError('bad')))
""",
     "1\ncaught")

test("generator_close",
     """
def g():
    yield 1
    yield 2
gen = g()
print(next(gen))
gen.close()
try:
    next(gen)
except StopIteration:
    print('closed')
""",
     "1\nclosed")

# ── 3. __slots__ ─────────────────────────────────────────────────────────────
test("slots_basic",
     """
class Point:
    __slots__ = ['x', 'y']
    def __init__(self, x, y):
        self.x = x
        self.y = y
p = Point(1, 2)
print(p.x, p.y)
""",
     "1 2")

test("slots_deny_extra",
     """
class Point:
    __slots__ = ['x', 'y']
    def __init__(self):
        self.x = 0
p = Point()
try:
    p.z = 99
    print('no error')
except AttributeError:
    print('blocked')
""",
     "blocked")

# ── 4. Relative imports ───────────────────────────────────────────────────────
# Write sibling modules and test relative import
import tempfile, os
tmpdir = tempfile.mkdtemp()
pkg_dir = os.path.join(tmpdir, "mypkg")
os.makedirs(pkg_dir)

with open(os.path.join(pkg_dir, "__init__.py"), "w") as f:
    f.write("")
with open(os.path.join(pkg_dir, "utils.py"), "w") as f:
    f.write("VALUE = 42\n")
with open(os.path.join(pkg_dir, "main.py"), "w") as f:
    f.write("from . import utils\nprint(utils.VALUE)\n")

r = subprocess.run([rustpy, os.path.join(pkg_dir, "main.py")], capture_output=True, text=True)
out = r.stdout.strip()
tests.append(("relative_import_dot", out == "42", out, r.stderr.strip(), "42"))

# ── 5. memoryview ─────────────────────────────────────────────────────────────
test("memoryview_basic",
     """
b = b'hello'
mv = memoryview(b)
print(mv[0])
print(mv[1])
""",
     "104\n101")

test("memoryview_tobytes",
     """
b = b'abc'
mv = memoryview(b)
print(mv.tobytes())
""",
     "b'abc'")

test("memoryview_tolist",
     """
b = b'\\x01\\x02\\x03'
mv = memoryview(b)
print(mv.tolist())
""",
     "[1, 2, 3]")

test("memoryview_nbytes",
     """
b = b'hello'
mv = memoryview(b)
print(mv.nbytes)
""",
     "5")

test("memoryview_readonly",
     """
b = b'hi'
mv = memoryview(b)
print(mv.readonly)
""",
     "True")

test("memoryview_len",
     """
b = b'hello'
mv = memoryview(b)
print(len(mv))
""",
     "5")

# ── 6. Match keyword patterns ─────────────────────────────────────────────────
test("match_keyword_pattern",
     """
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
p = Point(1, 2)
match p:
    case Point(x=1, y=2):
        print('matched 1,2')
    case _:
        print('no match')
""",
     "matched 1,2")

test("match_keyword_pattern_no_match",
     """
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
p = Point(3, 4)
match p:
    case Point(x=1, y=2):
        print('matched 1,2')
    case _:
        print('no match')
""",
     "no match")

# ─────────────────────────────────────────────────────────────────────────────
# Report
passed = sum(1 for _, p, *_ in tests if p)
failed = sum(1 for _, p, *_ in tests if not p)
print(f"\n{'='*60}")
print(f"Tier 3 Tests: {passed} passed, {failed} failed")
print(f"{'='*60}")
for name, p, out, err, exp in tests:
    status = "PASS" if p else "FAIL"
    print(f"  [{status}] {name}")
    if not p:
        print(f"    expected: {repr(exp)}")
        print(f"    got:      {repr(out)}")
        if err:
            print(f"    stderr:   {repr(err[:200])}")
sys.exit(0 if failed == 0 else 1)
