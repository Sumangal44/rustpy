class date:
    def __init__(self, year, month, day):
        self.year = year
        self.month = month
        self.day = day

    def __repr__(self):
        y = str(self.year)
        m = str(self.month)
        if len(m) == 1:
            m = '0' + m
        d = str(self.day)
        if len(d) == 1:
            d = '0' + d
        return y + '-' + m + '-' + d

    def __str__(self):
        return self.__repr__()


class time:
    def __init__(self, hour, minute=0, second=0):
        self.hour = hour
        self.minute = minute
        self.second = second

    def __repr__(self):
        h = str(self.hour)
        if len(h) == 1:
            h = '0' + h
        m = str(self.minute)
        if len(m) == 1:
            m = '0' + m
        s = str(self.second)
        if len(s) == 1:
            s = '0' + s
        return h + ':' + m + ':' + s


class timedelta:
    def __init__(self, days=0, seconds=0, microseconds=0):
        self.days = days
        self.seconds = seconds
        self.microseconds = microseconds


class datetime:
    def __init__(self, year, month, day, hour=0, minute=0, second=0):
        self.year = year
        self.month = month
        self.day = day
        self.hour = hour
        self.minute = minute
        self.second = second

    def __repr__(self):
        y = str(self.year)
        m = str(self.month)
        if len(m) == 1:
            m = '0' + m
        d = str(self.day)
        if len(d) == 1:
            d = '0' + d
        h = str(self.hour)
        if len(h) == 1:
            h = '0' + h
        mi = str(self.minute)
        if len(mi) == 1:
            mi = '0' + mi
        s = str(self.second)
        if len(s) == 1:
            s = '0' + s
        return y + '-' + m + '-' + d + ' ' + h + ':' + mi + ':' + s

    def __str__(self):
        return self.__repr__()

    @staticmethod
    def now():
        return datetime(2026, 7, 16, 17, 17, 11)

    @staticmethod
    def today():
        return datetime(2026, 7, 16, 17, 17, 11)
