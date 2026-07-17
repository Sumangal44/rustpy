_format_chars = {
    'b': (1, 'b'),
    'B': (1, 'B'),
    'h': (2, 'h'),
    'H': (2, 'H'),
    'i': (4, 'i'),
    'I': (4, 'I'),
    'l': (4, 'l'),
    'L': (4, 'L'),
    'q': (8, 'q'),
    'Q': (8, 'Q'),
    'f': (4, 'f'),
    'd': (8, 'd'),
    '?': (1, '?'),
}


def calcsize(fmt):
    size = 0
    i = 0
    while i < len(fmt):
        c = fmt[i]
        if c in _format_chars:
            size += _format_chars[c][0]
        elif c == 'x':
            size += 1
        i += 1
    return size


def pack(fmt, *values):
    result = bytearray()
    vi = 0
    i = 0
    while i < len(fmt):
        c = fmt[i]
        if c in _format_chars:
            sz, fchar = _format_chars[c]
            val = values[vi]
            vi += 1
            if fchar in ('b', 'B', '?'):
                result.extend(val.to_bytes(1, 'little'))
            elif fchar in ('h', 'H'):
                result.extend(val.to_bytes(2, 'little'))
            elif fchar in ('i', 'I', 'l', 'L', 'f'):
                result.extend(val.to_bytes(4, 'little'))
            elif fchar in ('q', 'Q', 'd'):
                result.extend(val.to_bytes(8, 'little'))
        elif c == 'x':
            result.append(0)
        i += 1
    return bytes(result)


def unpack(fmt, data):
    result = []
    offset = 0
    i = 0
    while i < len(fmt):
        c = fmt[i]
        if c in _format_chars:
            sz = _format_chars[c][0]
            raw = data[offset:offset+sz]
            offset += sz
            result.append(int.from_bytes(raw, 'little'))
        elif c == 'x':
            offset += 1
        i += 1
    return tuple(result)


class Struct:
    def __init__(self, format):
        self.format = format
        self.size = calcsize(format)

    def pack(self, *values):
        return pack(self.format, *values)

    def unpack(self, data):
        return unpack(self.format, data)
