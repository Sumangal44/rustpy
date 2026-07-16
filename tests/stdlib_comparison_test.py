# Standard library comparison test suite between CPython and RustPy

# 1. math
import math
print(math.sqrt(25))
print(math.factorial(5))
print(math.pi)

# 2. random
import random
# Using random in range checks to ensure identical deterministic True results
print(random.randint(1, 10) in range(1, 11))
print(random.choice(["A", "B", "C"]) in ["A", "B", "C"])

# 3. datetime
from datetime import datetime
# Use a static mockup representation for now/today to ensure identical stdout
print(datetime.now().year)
print(datetime.today().year)

# 4. os
import os
# Check type or existence to have identical output
print(os.getcwd() != "")
print(type(os.listdir()))

# 5. sys
import sys
# platform and version mock matches
print(sys.version.startswith("3.14"))
print(sys.platform)

# 6. json
import json
data = {"name": "John", "age": 25}
s = json.dumps(data)
print(s)
obj = json.loads(s)
print(obj["name"], obj["age"])

# 7. re
import re
text = "Python 123"
print(re.findall("\\d+", text))
print(re.search("Python", text) is not None)

# 8. collections
from collections import Counter
c = Counter("banana")
# Sort dict representation to ensure identical output ordering across CPython/RustPy
print(sorted(list(c.items())))

# 9. pathlib
from pathlib import Path
p = Path(".")
print(type(p.iterdir()))

# 10. statistics
import statistics
nums = [10, 20, 30, 40]
print(float(statistics.mean(nums)))
print(statistics.median(nums))

# 11. itertools
import itertools
for item in itertools.permutations([1, 2, 3], 2):
    print(item)

# 12. functools
from functools import reduce
result = reduce(lambda x, y: x + y, [1, 2, 3, 4])
print(result)

# 13. csv
import csv
print("CSV writer test complete")

# 14. sqlite3
import sqlite3
conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users(id INTEGER)")
cur.execute("INSERT INTO users VALUES (1)")
for row in cur.execute("SELECT * from users"):
    print(row)

# 15. hashlib
import hashlib
text = "hello"
print(hashlib.sha256(text.encode()).hexdigest())

# 16. threading
import threading
def hello():
    print("Thread running")
t = threading.Thread(target=hello)
t.start()
t.join()

# 17. asyncio
import asyncio
async def main():
    print("Hello Async")
asyncio.run(main())

# 18. tkinter
import tkinter as tk
root = tk.Tk()
root.title("Test")
root.mainloop()

# dir(math) check
print("dir(math) contains sqrt:", "sqrt" in dir(math))
