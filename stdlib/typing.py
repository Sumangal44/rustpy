class _Final:
    def __repr__(self):
        return self._name

class _SpecialForm:
    def __init__(self, name):
        self._name = name
    def __repr__(self):
        return self._name
    def __getitem__(self, params):
        return _GenericAlias(self, params)

class _GenericAlias:
    def __init__(self, origin, params):
        self.origin = origin
        self.params = params
    def __repr__(self):
        return f"{self.origin._name}[{self.params}]"
    def __eq__(self, other):
        if isinstance(other, _GenericAlias):
            return self.origin == other.origin and self.params == other.params
        return False

Any = _SpecialForm('Any')
Union = _SpecialForm('Union')
Optional = _SpecialForm('Optional')
Callable = _SpecialForm('Callable')
ClassVar = _SpecialForm('ClassVar')
Tuple = _SpecialForm('Tuple')
Type = _SpecialForm('Type')
List = _SpecialForm('List')
Dict = _SpecialForm('Dict')
Set = _SpecialForm('Set')
FrozenSet = _SpecialForm('FrozenSet')
Sequence = _SpecialForm('Sequence')
MutableSequence = _SpecialForm('MutableSequence')
Mapping = _SpecialForm('Mapping')
MutableMapping = _SpecialForm('MutableMapping')
Iterable = _SpecialForm('Iterable')
Iterator = _SpecialForm('Iterator')
Generator = _SpecialForm('Generator')
AsyncGenerator = _SpecialForm('AsyncGenerator')
ContextManager = _SpecialForm('ContextManager')
AsyncContextManager = _SpecialForm('AsyncContextManager')
Literal = _SpecialForm('Literal')
Final = _SpecialForm('Final')
TypeVar = _SpecialForm('TypeVar')

def cast(typ, val):
    return val

_type_checking = False
