# Tests for Phase 4 Features

# 1. Exception Hierarchy
print("--- Exception Hierarchy ---")
try:
    raise ZeroDivisionError("division by zero")
except ArithmeticError as e:
    print("Caught ZeroDivisionError as ArithmeticError:", e)

try:
    raise KeyError("missing key")
except LookupError as e:
    print("Caught KeyError as LookupError:", e)

class MyError(Exception): pass
class MySubError(MyError): pass

try:
    raise MySubError("custom sub error")
except MyError as e:
    print("Caught MySubError as MyError:", e)

# 2. Constant Folding
print("--- Constant Folding ---")
x = (2 + 3) * (10 - 8)
print("Folded (2 + 3) * (10 - 8) =", x)

s = "hello " + "world"
print("Folded 'hello ' + 'world' =", repr(s))

# 3. Tail Call Optimization
print("--- Tail Call Optimization ---")
def rec_sum(n, acc):
    if n <= 0:
        return acc
    return rec_sum(n - 1, acc + n)

# 105 is greater than RECURSION_LIMIT (100). Should run successfully due to TCO.
print("rec_sum(105, 0) =", rec_sum(105, 0))
