# collections mock

class Counter:
    def __init__(self, iterable=None):
        self._dict = {}
        if iterable:
            for item in iterable:
                self._dict[item] = self._dict.get(item, 0) + 1

    def items(self):
        return self._dict.items()
