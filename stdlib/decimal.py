class Decimal:
    def __init__(self, value='0', context=None):
        if isinstance(value, str):
            self._value = value
        elif isinstance(value, int):
            self._value = str(value)
        elif isinstance(value, float):
            self._value = str(value)
        elif isinstance(value, Decimal):
            self._value = value._value
        else:
            self._value = str(value)

    def __repr__(self):
        return f"Decimal('{self._value}')"

    def __str__(self):
        return self._value

    def __add__(self, other):
        if isinstance(other, int):
            other = Decimal(other)
        if isinstance(other, Decimal):
            a = float(self._value)
            b = float(other._value)
            return Decimal(str(a + b))
        return NotImplemented

    def __sub__(self, other):
        if isinstance(other, int):
            other = Decimal(other)
        if isinstance(other, Decimal):
            a = float(self._value)
            b = float(other._value)
            return Decimal(str(a - b))
        return NotImplemented

    def __mul__(self, other):
        if isinstance(other, int):
            other = Decimal(other)
        if isinstance(other, Decimal):
            a = float(self._value)
            b = float(other._value)
            return Decimal(str(a * b))
        return NotImplemented

    def __truediv__(self, other):
        if isinstance(other, int):
            other = Decimal(other)
        if isinstance(other, Decimal):
            a = float(self._value)
            b = float(other._value)
            return Decimal(str(a / b))
        return NotImplemented

    def __eq__(self, other):
        if isinstance(other, int):
            return float(self._value) == other
        if isinstance(other, Decimal):
            return self._value == other._value
        return False

    def __lt__(self, other):
        if isinstance(other, int):
            return float(self._value) < other
        if isinstance(other, Decimal):
            return float(self._value) < float(other._value)
        return NotImplemented

    def __le__(self, other):
        return self == other or self < other

    def __gt__(self, other):
        return not (self <= other)

    def __ge__(self, other):
        return not (self < other)

    def __neg__(self):
        return Decimal('-' + self._value)

    def __pos__(self):
        return Decimal(self._value)

    def __abs__(self):
        v = self._value
        if v.startswith('-'):
            return Decimal(v[1:])
        return Decimal(v)

    def __float__(self):
        return float(self._value)

    def __int__(self):
        return int(float(self._value))

    def __hash__(self):
        return hash(float(self._value))

    def __bool__(self):
        return float(self._value) != 0.0


def getcontext():
    return _Context()


class _Context:
    def __init__(self):
        self.prec = 28
        self.rounding = 'ROUND_HALF_EVEN'

    def create_decimal(self, value):
        return Decimal(value)


class ROUND_HALF_EVEN:
    pass


ROUND_DOWN = 'ROUND_DOWN'
ROUND_UP = 'ROUND_UP'
