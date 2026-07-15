# Test combining loops and conditionals

# 1. FizzBuzz classification loop
print("FizzBuzz 1 to 16:")
for i in range(1, 17):
    if i % 3 == 0 and i % 5 == 0:
        print(i, "FizzBuzz")
    elif i % 3 == 0:
        print(i, "Fizz")
    elif i % 5 == 0:
        print(i, "Buzz")
    else:
        print(i, "Number")

# 2. Prime Finder using nested loops & conditionals with break/else
print("\nPrimes up to 20:")
for n in range(2, 20):
    for x in range(2, n):
        if n % x == 0:
            break
    else:
        print(n, "is prime")

# 3. Conditional loops matching values in multi-dimensional list
matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]
print("\nMatrix elements matching conditions:")
for row_idx, row in enumerate(matrix):
    for col_idx, val in enumerate(row):
        if val % 2 == 0:
            print("Even at", row_idx, col_idx, "val:", val)
        else:
            if val == 5:
                print("Five at", row_idx, col_idx)
            else:
                continue
