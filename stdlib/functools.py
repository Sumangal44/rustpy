def reduce(function, iterable, initializer=None):
    it = iter(iterable)
    if initializer is None:
        try:
            value = next(it)
        except StopIteration:
            raise TypeError('reduce() of empty sequence with no initial value')
    else:
        value = initializer
    for x in it:
        value = function(value, x)
    return value


class partial:
    def __init__(self, func, *args, **kwargs):
        self.func = func
        self.args = args
        self.kwargs = kwargs

    def __call__(self, *args, **kwargs):
        merged = {}
        for k, v in self.kwargs.items():
            merged[k] = v
        for k, v in kwargs.items():
            merged[k] = v
        return self.func(*self.args, *args, **merged)
