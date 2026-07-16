# Dedicated map, filter, all, and any function parity tests

# 1. Map Function Tests
print("--- map tests ---")
# Eager map returning lists in RustPy, compatible with CPython list wrapping
print(list(map(lambda x: x * 3, [1, 2, 3, 4])))
print(list(map(str, [10, 20, 30])))
print(list(map(lambda x: x + 1, [])))

def add_five(x):
    return x + 5
print(list(map(add_five, (1, 2, 3))))

# 2. Filter Function Tests
print("--- filter tests ---")
print(list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5, 6])))
print(list(filter(lambda x: x > 10, [1, 15, 2, 20])))
print(list(filter(lambda x: x, [True, False, 1, 0, "", "hello"])))
print(list(filter(lambda x: True, [])))

def is_even(x):
    return x % 2 == 0
print(list(filter(is_even, (1, 2, 3, 4))))

# 3. All Function Tests
print("--- all tests ---")
print(all([True, True, True]))
print(all([True, False, True]))
print(all([]))  # Empty iterable should return True
print(all([1, 2, "hello", [1]]))
print(all([1, 0, 3]))
print(all(range(1, 5)))
print(all(range(0, 5)))

# 4. Any Function Tests
print("--- any tests ---")
print(any([False, False, False]))
print(any([False, True, False]))
print(any([]))  # Empty iterable should return False
print(any([0, "", [], None]))
print(any([0, "hello", None]))
print(any(range(0, 5)))
