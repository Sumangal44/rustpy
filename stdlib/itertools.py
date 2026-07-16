# itertools mock

def permutations(iterable, r=None):
    pool = tuple(iterable)
    n = len(pool)
    if r is None:
        r = n
    if r > n:
        return
    indices = list(range(n))
    cycles = list(range(n, n-r, -1))
    yield tuple(pool[i] for i in indices[:r])
    while n:
        found = False
        for i in reversed(range(r)):
            cycles[i] = cycles[i] - 1
            if cycles[i] == 0:
                val = indices.pop(i)
                indices.append(val)
                cycles[i] = n - i
            else:
                j = cycles[i]
                tmp = indices[-j]
                indices[-j] = indices[i]
                indices[i] = tmp
                yield tuple(pool[k] for k in indices[:r])
                found = True
                break
        if not found:
            return
