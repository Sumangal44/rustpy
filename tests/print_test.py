import sys

# Test sep
print(1, 2, 3, sep="-")

# Test end
print("Hello", end="!!!\n")

# Test combination of sep and end
print("a", "b", "c", sep=", ", end=".\n")

# Test flush (truthy/falsy)
print("Flush test", flush=True)

# Test file (using sys.stdout, sys.stderr)
# sys.stderr is a file/stream-like object
print("StdOut print", file=sys.stdout)
print("StdErr print", file=sys.stderr)
