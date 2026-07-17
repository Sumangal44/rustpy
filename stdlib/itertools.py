class chain:
    def __init__(self, *iterables):
        self.iters = []
        for it in iterables:
            self.iters.append(iter(it))
        self.idx = 0

    def __iter__(self):
        return self

    def __next__(self):
        while self.idx < len(self.iters):
            cur = self.iters[self.idx]
            try:
                return next(cur)
            except StopIteration:
                self.idx = self.idx + 1
        raise StopIteration()


class count:
    def __init__(self, start=0, step=1):
        self.n = start
        self.step = step

    def __next__(self):
        val = self.n
        self.n = self.n + self.step
        return val

    def __iter__(self):
        return self


def permutations(iterable, r=None):
    pool = list(iterable)
    n = len(pool)
    if r is None:
        r = n
    if r > n:
        return iter([])

    result = []
    indices = list(range(n))
    cycles = list(range(n, n - r, -1))

    perm = []
    for i in range(r):
        perm.append(pool[indices[i]])
    result.append(tuple(perm))

    while True:
        found = False
        i = r - 1
        while i >= 0:
            cycles[i] = cycles[i] - 1
            if cycles[i] == 0:
                val = indices.pop(i)
                indices.append(val)
                cycles[i] = n - i
                i = i - 1
            else:
                j = cycles[i]
                indices[i], indices[n - j] = indices[n - j], indices[i]
                perm = []
                for k in range(r):
                    perm.append(pool[indices[k]])
                result.append(tuple(perm))
                found = True
                break
        if not found:
            break

    return iter(result)
