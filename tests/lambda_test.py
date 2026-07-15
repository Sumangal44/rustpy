# Dedicated Lambda function tests

# 1. Standard multiple arguments lambda
multiply = lambda x, y: x * y
print(multiply(6, 7))

# 2. Currying (lambda returning another lambda)
add_n = lambda n: lambda x: x + n
add_five = add_n(5)
print(add_five(10))
print(add_five(20))

# 3. Inline self-invocation
val = (lambda x: x * x)(9)
print(val)

# 4. Lambda in list comprehensions using a closure wrapper
def make_lambda(i):
    return lambda x: x + i

funcs = [make_lambda(i) for i in range(3)]
for f in funcs:
    print(f(10))

# 5. Capturing outer variables
y = 100
add_y = lambda x: x + y
print(add_y(50))
y = 200
print(add_y(50))
