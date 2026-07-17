_FIELD = object()
_FIELDS = '__dataclasses_fields__'


def _is_dataclass_instance(obj):
    return hasattr(obj, _FIELDS)


def _get_fields(cls):
    fields = {}
    if _FIELDS in cls.__dict__:
        for name, field in cls.__dict__[_FIELDS].items():
            fields[name] = field
    for base in cls.__class__.__mro__:
        if _FIELDS in base.__dict__:
            for name, field in base.__dict__[_FIELDS].items():
                if name not in fields:
                    fields[name] = field
    return fields


def field(*, default=_FIELD, default_factory=_FIELD, init=True, repr=True, hash=None, compare=True, metadata=None):
    return (default, default_factory, init, repr, hash, compare, metadata)


def dataclass(cls=None, *, init=True, repr=True, eq=True, order=False, frozen=False, unsafe_hash=False):
    def wrap(cls):
        annotations = cls.__annotations__ if hasattr(cls, '__annotations__') else {}
        fields = {}

        if hasattr(cls, _FIELDS):
            fields.update(getattr(cls, _FIELDS))

        field_defaults = {}
        field_has_default = {}

        field_specs = {}
        for name, ann_type in annotations.items():
            if name.startswith('_'):
                continue
            spec = {}
            if name in cls.__dict__:
                val = cls.__dict__[name]
                if isinstance(val, tuple) and len(val) == 7:
                    default, default_factory, finit, frepr, fhash, fcompare, fmetadata = val
                    spec['default'] = default
                    spec['default_factory'] = default_factory
                    spec['init'] = finit
                    spec['repr'] = frepr
                    spec['hash'] = fhash
                    spec['compare'] = fcompare
                    spec['metadata'] = fmetadata
                    if default is not _FIELD:
                        field_defaults[name] = default
                    if default_factory is not _FIELD:
                        field_defaults[name] = default_factory
                else:
                    field_defaults[name] = val
            spec['name'] = name
            spec['type'] = ann_type
            fields[name] = spec

        setattr(cls, _FIELDS, fields)

        if init:
            init_params = []
            for name, spec in fields.items():
                if spec.get('init', True):
                    if name in field_defaults:
                        init_params.append((name, field_defaults[name]))
                    else:
                        init_params.append((name, None))

            def make_init(params):
                def __init__(self, *args, **kwargs):
                    param_values = {}
                    for i, (name, default) enumerate(params):
                        if i < len(args):
                            param_values[name] = args[i]
                        elif name in kwargs:
                            param_values[name] = kwargs[name]
                        elif default is not None:
                            param_values[name] = default
                        else:
                            raise TypeError(f"__init__() missing required argument: '{name}'")
                    for name, val in param_values.items():
                        setattr(self, name, val)
                return __init__

            cls.__init__ = make_init(init_params)

        if repr:
            def __repr__(self):
                field_strs = []
                for name, spec in fields.items():
                    if spec.get('repr', True):
                        field_strs.append(f"{name}={getattr(self, name, None)!r}")
                return f"{cls.__name__}({', '.join(field_strs)})"
            cls.__repr__ = __repr__

        if eq:
            def __eq__(self, other):
                if not isinstance(other, cls):
                    return NotImplemented
                for name, spec in fields.items():
                    if spec.get('compare', True):
                        if getattr(self, name) != getattr(other, name):
                            return False
                return True
            cls.__eq__ = __eq__

        if order:
            def __lt__(self, other):
                if not isinstance(other, cls):
                    return NotImplemented
                for name, spec in fields.items():
                    if spec.get('compare', True):
                        a = getattr(self, name)
                        b = getattr(other, name)
                        if a < b:
                            return True
                        if a > b:
                            return False
                return False
            cls.__lt__ = __lt__

            def __le__(self, other):
                if not isinstance(other, cls):
                    return NotImplemented
                return self == other or self < other

            def __gt__(self, other):
                if not isinstance(other, cls):
                    return NotImplemented
                return not (self <= other)

            def __ge__(self, other):
                if not isinstance(other, cls):
                    return NotImplemented
                return not (self < other)

            cls.__le__ = __le__
            cls.__gt__ = __gt__
            cls.__ge__ = __ge__

        if frozen:
            def __setattr__(self, name, value):
                raise TypeError(f"cannot assign to field '{name}' in frozen dataclass '{cls.__name__}'")
            cls.__setattr__ = __setattr__

        return cls

    if cls is None:
        return wrap

    if callable(cls):
        return wrap(cls)
    return cls
