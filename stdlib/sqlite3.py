# sqlite3 mock

class Cursor:
    def __init__(self):
        self.results = [(1,)]
    def execute(self, sql):
        return self.results

class Connection:
    def cursor(self):
        return Cursor()

def connect(database):
    return Connection()
