# csv mock

class writer:
    def __init__(self, f):
        self.f = f
    def writerow(self, row):
        line = ",".join(str(x) for x in row) + "\r\n"
        self.f.write(line)
