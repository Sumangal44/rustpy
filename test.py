def greet(name):
    print("Hello,", name)
    print("Your name has length:", len(name))

greet("RustPy")

a = 10
b = 20
print("10 + 20 =", a + b)

my_list = [1, 2, "three", [4, 5]]
print("List:", my_list)
print("List[2]:", my_list[2])

my_dict = {"name": "RustPy", "version": 1}
print("Dict:", my_dict)
print("Dict['name']:", my_dict["name"])
