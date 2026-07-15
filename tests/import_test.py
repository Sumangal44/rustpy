# Comprehensive import behavior tests
import sys
# Make sure we add the tests directory to sys.path so RustPy can resolve helper_module
if "tests" not in sys.path:
    sys.path.append("tests")

# 1. Built-in math module import
import math
print("math.sqrt(16):", math.sqrt(16.0))

# 2. Built-in sys module import
print("sys.path is list:", type(sys.path) is list)

# 3. Import local file as alias
import helper_module as helper
print("helper.value:", helper.value)
print("helper.func(5):", helper.func(5))

# 4. From-import specific items
from helper_module import value, func
print("from-imported value:", value)
print("from-imported func(5):", func(5))

# 5. From-import with alias
from helper_module import func as my_func
print("aliased func(5):", my_func(5))

# 6. Star import
from helper_module import *
print("star value:", value)
print("star func(10):", func(10))
h = HelperClass(100)
print("star HelperClass val:", h.val)
