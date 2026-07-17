import math


class Fraction:
    def __init__(self, numerator=0, denominator=None):
        if denominator is None:
            if isinstance(numerator, str):
                if '/' in numerator:
                    parts = numerator.split('/')
                    numerator = int(parts[0])
                    denominator = int(parts[1])
                else:
                    numerator = int(numerator)
                    denominator = 1
            elif isinstance(numerator, float):
                denom = 10 ** 15
                num = int(round(numerator * denom))
                return self._init_from_numer_denom(num, denom)
            elif isinstance(numerator, Fraction):
                self._num = numerator._num
                self._den = numerator._den
                return
            else:
                numerator = int(numerator)
                denominator = 1

        if denominator is None:
            denominator = 1

        if denominator < 0:
            numerator = -numerator
            denominator = -denominator

        g = math.gcd(abs(numerator), abs(denominator))
        self._num = numerator // g
        self._den = denominator // g

    def _init_from_numer_denom(self, num, den):
        if den < 0:
            num = -num
            den = -den
        g = math.gcd(abs(num), abs(den))
        self._num = num // g
        self._den = den // g

    @property
    def numerator(self):
        return self._num

    @property
    def denominator(self):
        return self._den

    def __repr__(self):
        return f'Fraction({self._num}, {self._den})'

    def __str__(self):
        if self._den == 1:
            return str(self._num)
        return f'{self._num}/{self._den}'

    def __add__(self, other):
        if isinstance(other, int):
            other = Fraction(other)
        if isinstance(other, Fraction):
            num = self._num * other._den + other._num * self._den
            den = self._den * other._den
            return Fraction(num, den)
        return NotImplemented

    def __sub__(self, other):
        if isinstance(other, int):
            other = Fraction(other)
        if isinstance(other, Fraction):
            num = self._num * other._den - other._num * self._den
            den = self._den * other._den
            return Fraction(num, den)
        return NotImplemented

    def __mul__(self, other):
        if isinstance(other, int):
            other = Fraction(other)
        if isinstance(other, Fraction):
            return Fraction(self._num * other._num, self._den * other._den)
        return NotImplemented

    def __truediv__(self, other):
        if isinstance(other, int):
            other = Fraction(other)
        if isinstance(other, Fraction):
            return Fraction(self._num * other._den, self._den * other._num)
        return NotImplemented

    def __eq__(self, other):
        if isinstance(other, int):
            return self._den == 1 and self._num == other
        if isinstance(other, Fraction):
            return self._num == other._num and self._den == other._den
        return False

    def __lt__(self, other):
        if isinstance(other, int):
            other = Fraction(other)
        if isinstance(other, Fraction):
            return self._num * other._den < other._num * self._den
        return NotImplemented

    def __le__(self, other):
        return self == other or self < other

    def __gt__(self, other):
        return not (self <= other)

    def __ge__(self, other):
        return not (self < other)

    def __neg__(self):
        return Fraction(-self._num, self._den)

    def __pos__(self):
        return Fraction(self._num, self._den)

    def __abs__(self):
        return Fraction(abs(self._num), self._den)

    def __float__(self):
        return self._num / self._den

    def __hash__(self):
        return hash(self._num * 31 + self._den)

    def __bool__(self):
        return self._num != 0
