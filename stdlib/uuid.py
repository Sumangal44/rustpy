import os
import time


def _int_from_bytes(bytes_val, byteorder='big'):
    return int.from_bytes(bytes_val, byteorder)


def _int_to_bytes(val, length, byteorder='big'):
    return val.to_bytes(length, byteorder)


class UUID:
    def __init__(self, hex=None, bytes_val=None, fields=None, int_val=None, version=None):
        if hex is not None:
            self._str = hex.lower()
            self._int = int(hex.replace('-', ''), 16)
        elif int_val is not None:
            self._int = int_val
            self._str = format(int_val, '032x')
        elif bytes_val is not None:
            self._int = _int_from_bytes(bytes_val, 'big')
            self._str = format(self._int, '032x')
        elif fields is not None:
            self._int = (fields[0] << 96) | (fields[1] << 80) | (fields[2] << 64) | (fields[3] << 48) | (fields[4] << 32) | fields[5]
            self._str = format(self._int, '032x')
        else:
            self._int = 0
            self._str = '00000000000000000000000000000000'

    def __str__(self):
        s = self._str
        return s[:8] + '-' + s[8:12] + '-' + s[12:16] + '-' + s[16:20] + '-' + s[20:]

    def __repr__(self):
        return "UUID('" + str(self) + "')"

    @property
    def hex(self):
        return self._str

    @property
    def int_val(self):
        return self._int

    @property
    def bytes(self):
        return _int_to_bytes(self._int, 16, 'big')

    @property
    def version(self):
        return (self._int >> 76) & 0xf

    def __eq__(self, other):
        if isinstance(other, UUID):
            return self._int == other._int
        return False

    def __hash__(self):
        return hash(self._int)


def uuid1():
    node = _int_from_bytes(os.urandom(6), 'big') | 0x010000000000
    timestamp = int(time.time() * 10000000) + 0x01b21dd213814000
    clock_seq = _int_from_bytes(os.urandom(2), 'big') & 0x3fff
    time_low = timestamp & 0xffffffff
    time_mid = (timestamp >> 32) & 0xffff
    time_hi_version = ((timestamp >> 48) & 0x0fff) | 0x1000
    clock_seq_hi = ((clock_seq >> 8) & 0x3f) | 0x80
    clock_seq_low = clock_seq & 0xff
    fields = (time_low, time_mid, time_hi_version, clock_seq_hi, clock_seq_low, node)
    return UUID(fields=fields)


def uuid4():
    return UUID(bytes_val=os.urandom(16))


UUID4 = uuid4
