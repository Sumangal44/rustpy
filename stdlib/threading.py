# threading mock

class Thread:
    def __init__(self, target, args=None):
        self.target = target
        if args is None:
            self.args = []
        else:
            self.args = args
    def start(self):
        self.target(*self.args)
    def join(self):
        pass
