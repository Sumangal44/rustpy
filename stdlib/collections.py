class OrderedDict(dict):
    pass


def namedtuple(typename, field_names):
    if isinstance(field_names, str):
        field_names = field_names.replace(',', ' ').split()
    fields = tuple(field_names)

    def __init__(self, *args, **kwargs):
        for i in range(len(fields)):
            if i < len(args):
                setattr(self, fields[i], args[i])
            elif fields[i] in kwargs:
                setattr(self, fields[i], kwargs[fields[i]])
            else:
                raise TypeError('missing argument: ' + fields[i])

    def __repr__(self):
        parts = []
        for f in fields:
            parts.append(str(getattr(self, f)))
        return typename + '(' + ', '.join(parts) + ')'

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        for f in fields:
            if getattr(self, f) != getattr(other, f):
                return False
        return True

    def __hash__(self):
        vals = [getattr(self, f) for f in fields]
        return hash(tuple(vals))

    def _asdict(self):
        d = {}
        for f in fields:
            d[f] = getattr(self, f)
        return d

    def _replace(self, **kwargs):
        vals = {}
        for f in fields:
            vals[f] = getattr(self, f)
        for k, v in kwargs.items():
            vals[k] = v
        return self.__class__(**vals)

    methods = {
        '__init__': __init__,
        '__repr__': __repr__,
        '__eq__': __eq__,
        '__hash__': __hash__,
        '_asdict': _asdict,
        '_replace': _replace,
    }
    cls = type(typename, (), methods)
    cls.__qualname__ = typename
    return cls


class Counter:
    def __init__(self, iterable=None):
        self._dict = {}
        if iterable:
            for item in iterable:
                self._dict[item] = self._dict.get(item, 0) + 1

    def items(self):
        return self._dict.items()

    def __getitem__(self, key):
        return self._dict[key]

    def __repr__(self):
        pairs = []
        for k, v in self._dict.items():
            pairs.append(repr(k) + ': ' + repr(v))
        return 'Counter({' + ', '.join(pairs) + '})'
