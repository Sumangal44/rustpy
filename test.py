print("Starting try block")
try:
    print("Inside try")
    raise "CustomError"
    print("This should not run")
except:
    print("Caught an error!")
print("Finished")
