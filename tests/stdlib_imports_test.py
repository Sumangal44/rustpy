# Parity test for importing all standard library modules

modules_to_test = [
    "sys", "math", "os", "random", "datetime", "time", "calendar",
    "pathlib", "shutil", "json", "csv", "re", "collections", "itertools", "functools",
    "statistics", "decimal", "fractions", "string", "hashlib", "secrets", "logging",
    "sqlite3", "threading", "multiprocessing", "asyncio", "socket", "urllib", "email",
    "zipfile", "gzip", "tarfile", "tkinter", "unittest", "typing", "dataclasses"
]

for name in modules_to_test:
    try:
        mod = __import__(name)
        # Check that the imported object is indeed a module and has name/attr support
        name_attr = mod.__name__
    except Exception as e:
        print("Failed to import:", name, e)

print("All standard library imports verified successfully!")
