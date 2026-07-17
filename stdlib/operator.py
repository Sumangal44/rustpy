def add(a, b): return a + b
def sub(a, b): return a - b
def mul(a, b): return a * b
def truediv(a, b): return a / b
def floordiv(a, b): return a // b
def mod(a, b): return a % b
def pow(a, b): return a ** b
def matmul(a, b): return a @ b
def lshift(a, b): return a << b
def rshift(a, b): return a >> b
def and_(a, b): return a & b
def or_(a, b): return a | b
def xor(a, b): return a ^ b
def neg(a): return -a
def pos(a): return +a
def invert(a): return ~a
def abs_(a): return abs(a)
def not_(a): return not a
def truth(a): return bool(a)
def is_(a, b): return a is b
def is_not(a, b): return a is not b
def eq(a, b): return a == b
def ne(a, b): return a != b
def lt(a, b): return a < b
def le(a, b): return a <= b
def gt(a, b): return a > b
def ge(a, b): return a >= b
def contains(a, b): return b in a
def indexOf(a, b):
    for i, v in enumerate(a):
        if v == b:
            return i
    raise ValueError("sequence.index(x): x not in sequence")
def countOf(a, b):
    count = 0
    for v in a:
        if v == b:
            count += 1
    return count
def getitem(a, b): return a[b]
def setitem(a, b, c): a[b] = c
def delitem(a, b): del a[b]
def getattr(a, b): return getattr(a, b)
def setattr(a, b, c): setattr(a, b, c)
def delattr(a, b): delattr(a, b)
def length_hint(obj, default=0):
    try:
        return len(obj)
    except TypeError:
        return default
attrgetter = property
itemgetter = property
methodcaller = property
