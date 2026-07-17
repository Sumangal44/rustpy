def copy(x):
    t = type(x)
    if t in (int, float, complex, bool, str, bytes, type(None)):
        return x
    if t is list:
        return list(x)
    if t is tuple:
        return tuple(x)
    if t is dict:
        return dict(x)
    if t is set:
        return set(x)
    if t is frozenset:
        return frozenset(x)
    return x


def deepcopy(x, memo=None):
    if memo is None:
        memo = {}
    t = type(x)
    if t in (int, float, complex, bool, str, bytes, type(None)):
        return x
    x_id = id(x)
    if x_id in memo:
        return memo[x_id]
    if t is list:
        result = []
        memo[x_id] = result
        for item in x:
            result.append(deepcopy(item, memo))
        return result
    if t is tuple:
        result = tuple(deepcopy(item, memo) for item in x)
        memo[x_id] = result
        return result
    if t is dict:
        result = {}
        memo[x_id] = result
        for k, v in x.items():
            result[deepcopy(k, memo)] = deepcopy(v, memo)
        return result
    if t is set:
        result = set()
        memo[x_id] = result
        for item in x:
            result.add(deepcopy(item, memo))
        return result
    if t is frozenset:
        result = frozenset(deepcopy(item, memo) for item in x)
        memo[x_id] = result
        return result
    return x
