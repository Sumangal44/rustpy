# Comprehensive loop structures test

# 1. Basic for loop
print("Basic for loop:")
for i in range(3):
    print(i)

# 2. Basic while loop
print("\nBasic while loop:")
count = 0
while count < 3:
    print(count)
    count += 1

# 3. for loop with break
print("\nfor loop with break:")
for i in range(5):
    if i == 3:
        break
    print(i)

# 4. while loop with break
print("\nwhile loop with break:")
count = 0
while True:
    if count == 3:
        break
    print(count)
    count += 1

# 5. for loop with continue
print("\nfor loop with continue:")
for i in range(5):
    if i == 2:
        continue
    print(i)

# 6. while loop with continue
print("\nwhile loop with continue:")
count = 0
while count < 5:
    count += 1
    if count == 3:
        continue
    print(count)

# 7. for-else clause
print("\nfor-else (no break):")
for i in range(3):
    print(i)
else:
    print("for-else executed!")

print("\nfor-else (with break):")
for i in range(3):
    if i == 1:
        break
    print(i)
else:
    print("for-else executed (should not see this)!")

# 8. while-else clause
print("\nwhile-else (no break):")
count = 0
while count < 3:
    print(count)
    count += 1
else:
    print("while-else executed!")

print("\nwhile-else (with break):")
count = 0
while count < 3:
    if count == 1:
        break
    print(count)
    count += 1
else:
    print("while-else executed (should not see this)!")

# 9. Nested loops
print("\nNested loops:")
for x in range(2):
    for y in range(2):
        print(x, y)
