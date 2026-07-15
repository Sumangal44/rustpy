# Comprehensive if-elif-else conditional testing

# 1. Basic if-else
x = 10
if x > 5:
    print("x is greater than 5")
else:
    print("x is not greater than 5")

# 2. Multiple elif blocks
score = 85
if score >= 90:
    print("Grade: A")
elif score >= 80:
    print("Grade: B")
elif score >= 70:
    print("Grade: C")
else:
    print("Grade: F")

# 3. Nested conditionals
y = 15
if y > 10:
    if y % 2 == 0:
        print("y is even and greater than 10")
    else:
        print("y is odd and greater than 10")
else:
    print("y is less than or equal to 10")

# 4. Truthiness / Falsiness in conditionals
falsy_values = [None, False, 0, 0.0, "", [], {}, ()]
truthy_values = [True, 1, 1.5, "hello", [1], {"a": 1}, (1,)]

for val in falsy_values:
    if val:
        print("Falsy evaluated as Truthy:", repr(val))
    else:
        print("Falsy evaluated as Falsy:", repr(val))

for val in truthy_values:
    if val:
        print("Truthy evaluated as Truthy:", repr(val))
    else:
        print("Truthy evaluated as Falsy:", repr(val))

# 5. Ternary operator (inline if-else)
status = "Even" if y % 2 == 0 else "Odd"
print("Status of y:", status)
