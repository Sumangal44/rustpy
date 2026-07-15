class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    __match_args__ = ('x', 'y')

p = Point(10, 20)
match p:
    case Point(x, y):
        print(x, y)
    case _:
        print("no match")
