# Comprehensive built-in functions test

# 1. Primitives & Collections casting
print(int("123"), float("12.3"), bool(1), bool(0))
print(str(100), repr("hello"), ascii("hello\nworld"))
print(list((1, 2)), tuple([3, 4]), set([1, 2, 2]), dict([("a", 1)]))
print(bytes([65, 66]))

# 2. Math & computational
print(abs(-5), abs(-5.5))
print(round(5.4), round(5.6))
print(min(5, 3, 8), max(5, 3, 8))
print(sum([1, 2, 3]), pow(2, 3), divmod(10, 3))

# 3. Conversion
print(chr(65), ord("A"))
print(hex(255), oct(8), bin(5))

# 4. Iterators & High-order
print(all([True, True]), all([True, False]))
print(any([False, True]), any([False, False]))

for idx, val in enumerate(["a", "b"]):
    print(idx, val)

for x, y in zip([1, 2], [3, 4]):
    print(x, y)

print(list(map(lambda x: x * 2, [1, 2, 3])))
print(list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4])))
print(list(reversed([1, 2, 3])))
print(sorted([3, 1, 2]))

# 5. Meta & reflection
class Foo:
    def __init__(self):
        self.x = 10

f = Foo()
print(isinstance(f, Foo))
print(callable(Foo), callable(f))
print(hasattr(f, "x"), hasattr(f, "y"))
setattr(f, "y", 20)
print(getattr(f, "x"), getattr(f, "y"))

# Check type of id and hash (value might differ, but type is always int)
print(type(id(f)) is int)
print(type(hash("test")) is int)
