B64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
B64_URLSAFE_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"

def b64encode(data, altchars=None):
    if isinstance(data, str):
        data = data.encode()
    chars = bytearray(altchars.encode()) if altchars else bytearray(B64_CHARS.encode())
    result = bytearray()
    i = 0
    while i < len(data):
        chunk = data[i:i+3]
        pad = 3 - len(chunk)
        chunk = chunk + b'\x00' * pad
        val = chunk[0] << 16 | chunk[1] << 8 | chunk[2]
        n = 4 - pad
        result.append(chars[(val >> 18) & 0x3F])
        if n > 1:
            result.append(chars[(val >> 12) & 0x3F])
        if n > 2:
            result.append(chars[(val >> 6) & 0x3F])
        if n > 3:
            result.append(chars[val & 0x3F])
        result.extend(b'=' * pad)
        i += 3
    return bytes(result)


def b64decode(data, altchars=None, validate=False):
    if isinstance(data, str):
        data = data.encode()
    data = data.rstrip(b'\n').rstrip()
    pad = data.count(b'=')
    if altchars:
        data = data.replace(altchars.encode()[0:1], b'A')
        data = data.replace(altchars.encode()[1:2], b'/')
    data = data.replace(b'=', b'A')
    if validate:
        for c in data:
            if c not in B64_CHARS.encode() and c != ord('='):
                raise ValueError("Invalid base64 character")
    result = bytearray()
    i = 0
    while i < len(data):
        chunk = data[i:i+4]
        if len(chunk) < 4:
            chunk = chunk + b'A' * (4 - len(chunk))
        val = 0
        for c in chunk:
            idx = B64_CHARS.find(chr(c))
            if idx < 0:
                idx = 0
            val = (val << 6) | idx
        result.append((val >> 16) & 0xFF)
        result.append((val >> 8) & 0xFF)
        result.append(val & 0xFF)
        i += 4
    if pad:
        result = result[:-pad]
    return bytes(result)


def urlsafe_b64encode(data):
    return b64encode(data, altchars='-_')


def urlsafe_b64decode(data):
    return b64decode(data, altchars='-_')


def b32encode(data):
    B32_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    if isinstance(data, str):
        data = data.encode()
    result = bytearray()
    i = 0
    while i < len(data):
        chunk = data[i:i+5]
        pad = 5 - len(chunk)
        chunk = chunk + b'\x00' * pad
        val = chunk[0] << 32 | chunk[1] << 24 | chunk[2] << 16 | chunk[3] << 8 | chunk[4]
        n = 8 - pad
        shifts = [35, 30, 25, 20, 15, 10, 5, 0]
        for idx in range(n):
            result.append(ord(B32_CHARS[(val >> shifts[idx]) & 0x1F]))
        result.extend(b'=' * pad)
        i += 5
    return bytes(result)


def b32decode(data):
    B32_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    if isinstance(data, str):
        data = data.encode()
    data = data.rstrip(b'=')
    result = bytearray()
    i = 0
    while i < len(data):
        chunk = data[i:i+8]
        if len(chunk) < 8:
            chunk = chunk + b'A' * (8 - len(chunk))
        val = 0
        for c in chunk:
            idx = B32_CHARS.find(chr(c).upper())
            if idx < 0:
                idx = 0
            val = (val << 5) | idx
        for shift in [32, 24, 16, 8, 0]:
            result.append((val >> shift) & 0xFF)
        i += 8
    # Trim padding
    orig_pad = (8 - (len(data) % 8)) % 8
    if orig_pad:
        result = result[:-orig_pad]
    return bytes(result)


def b16encode(data):
    if isinstance(data, str):
        data = data.encode()
    return data.hex().upper().encode()


def b16decode(data):
    if isinstance(data, str):
        data = data.encode()
    return bytes.fromhex(data.decode())


def encodebytes(s):
    if isinstance(s, str):
        s = s.encode()
    return b64encode(s) + b'\n'


def decodebytes(s):
    return b64decode(s)
