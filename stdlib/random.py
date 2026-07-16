# LCG Pseudo-random number generator mock
_seed = 42

def seed(a):
    global _seed
    _seed = hash(a) & 0x7fffffff

def randint(a, b):
    global _seed
    _seed = (_seed * 1103515245 + 12345) & 0x7fffffff
    val = _seed % (b - a + 1)
    return a + val

def choice(seq):
    idx = randint(0, len(seq) - 1)
    return seq[idx]
