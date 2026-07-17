def _make_enum(name, members):
    cls_dict = {}
    member_map = {}
    value_map = {}
    for key, value in members.items():
        member = object.__new__(_EnumInstance)
        member._name_ = key
        member._value_ = value
        member_map[key] = member
        value_map[value] = member
        cls_dict[key] = value
    cls = type(name, (object,), cls_dict)
    cls._member_names_ = list(member_map.keys())
    cls._member_map_ = member_map
    cls._value2member_map_ = value_map
    return cls


class _EnumInstance:
    _name_ = ''
    _value_ = None

    def __repr__(self):
        return "<%s.%s: %s>" % (self.__class__.__name__, self._name_, repr(self._value_))

    def __str__(self):
        return "%s.%s" % (self.__class__.__name__, self._name_)

    @property
    def name(self):
        return self._name_

    @property
    def value(self):
        return self._value_

    def __eq__(self, other):
        if isinstance(other, _EnumInstance):
            return self._value_ == other._value_
        return self._value_ == other

    def __hash__(self):
        return hash(self._value_)

    def __bool__(self):
        return True


class EnumMeta(type):
    pass


class Enum:
    pass


def _missing_(cls, value):
    return None


_auto_counter = {}
def auto():
    global _auto_counter
    import sys as _sys
    name = 'auto'
    count = _auto_counter.get(name, 0) + 1
    _auto_counter[name] = count
    return count


def unique(enum_class):
    values = {}
    for name in enum_class._member_names_:
        member = enum_class._member_map_[name]
        if member._value_ in values:
            raise ValueError("duplicate value %r in %s" % (member._value_, enum_class.__name__))
        values[member._value_] = member
    return enum_class
