# Dedicated F-String parity tests

# 1. Simple variables and literals
name = "world"
print(f"hello {name}")

# 2. Arithmetic expressions
x = 10
y = 20
print(f"sum: {x} + {y} = {x + y}")

# 3. Float formatting with precision
pi = 3.14159265
print(f"pi to 2 decimal places: {pi:.2f}")
print(f"pi to 4 decimal places: {pi:.4f}")

# 4. Width and alignment formatting
val = 123
print(f"right: {val:>8}")
print(f"left: {val:<8}")
print(f"center: {val:^8}")

# 5. Escaped braces (literal '{' and '}')
val = 42
print(f"braces: {{{val}}}")

# 6. Debug format (name=value)
x = 42
y = 10
print(f"{x=}")
print(f"{x+y=}")

# 7. Empty f-strings
print(f"")

# 8. Single and double quotes inside f-strings
name = "Gemini"
print(f"hello {'there'} {name}")
print(f'hello {"there"} {name}')

# 9. Escape sequences inside f-strings
val = "test"
print(f"escape: newline\nand {val}")
