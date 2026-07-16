# datetime mock

class datetime:
    def __init__(self, year, month, day, hour=0, minute=0, second=0):
        self.year = year
        self.month = month
        self.day = day
        self.hour = hour
        self.minute = minute
        self.second = second

    def __repr__(self):
        # Format string nicely, padding with zeros
        y = str(self.year)
        m = str(self.month)
        if len(m) == 1: m = "0" + m
        d = str(self.day)
        if len(d) == 1: d = "0" + d
        h = str(self.hour)
        if len(h) == 1: h = "0" + h
        mi = str(self.minute)
        if len(mi) == 1: mi = "0" + mi
        s = str(self.second)
        if len(s) == 1: s = "0" + s
        return f"{y}-{m}-{d} {h}:{mi}:{s}"

    def __str__(self):
        return self.__repr__()

    @staticmethod
    def now():
        return datetime(2026, 7, 16, 17, 17, 11)

    @staticmethod
    def today():
        return datetime(2026, 7, 16, 17, 17, 11)
