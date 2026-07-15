# Comprehensive function features test

# 1. Lambdas
add = lambda x, y: x + y
print(add(5, 10))
print(add(5, 20))

# 2. Closures & Lexical Scoping
def make_counter(start):
    count = start
    def incr():
        nonlocal count
        count += 1
        return count
    return incr

c = make_counter(10)
print(c())
print(c())

# 3. High-order Functions
def apply(func, val):
    return func(val)

print(apply(lambda x: x * 3, 5))

# 4. Nested functions capturing read-only variables
def outer(x):
    def inner(y):
        return x + y
    return inner

f = outer(100)
print(f(50))
