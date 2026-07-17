def pformat(obj, indent=1, width=80, depth=None, compact=False):
    return _format(obj, indent, width, depth, compact)


def pprint(obj, stream=None, indent=1, width=80, depth=None, compact=False):
    if stream is None:
        import sys as _sys
        stream = _sys.stdout
    output = pformat(obj, indent, width, depth, compact)
    stream.write(output)
    stream.write('\n')


def _format(obj, indent, width, depth, compact, level=0):
    if depth is not None and level >= depth:
        return '...'
    if isinstance(obj, str):
        return repr(obj)
    if isinstance(obj, bytes):
        return repr(obj)
    if isinstance(obj, int) or isinstance(obj, float) or isinstance(obj, bool) or obj is None:
        return repr(obj)
    if isinstance(obj, list):
        if not obj:
            return '[]'
        if len(obj) == 1:
            return '[' + _format(obj[0], indent, width, depth, compact, level) + ']'
        items = []
        for item in obj:
            items.append(_format(item, indent, width, depth, compact, level + 1))
        inner = ',\n'.join(' ' * (indent * (level + 1)) + i for i in items)
        return '[\n' + inner + '\n' + ' ' * (indent * level) + ']'
    if isinstance(obj, tuple):
        if not obj:
            return '()'
        items = []
        for item in obj:
            items.append(_format(item, indent, width, depth, compact, level + 1))
        inner = ',\n'.join(' ' * (indent * (level + 1)) + i for i in items)
        return '(\n' + inner + '\n' + ' ' * (indent * level) + ')'
    if isinstance(obj, dict):
        if not obj:
            return '{}'
        items = []
        for k, v in obj.items():
            k_str = _format(k, indent, width, depth, compact, level + 1)
            v_str = _format(v, indent, width, depth, compact, level + 1)
            items.append(k_str + ': ' + v_str)
        inner = ',\n'.join(' ' * (indent * (level + 1)) + i for i in items)
        return '{\n' + inner + '\n' + ' ' * (indent * level) + '}'
    if isinstance(obj, set):
        if not obj:
            return 'set()'
        items = []
        for item in obj:
            items.append(_format(item, indent, width, depth, compact, level + 1))
        inner = ',\n'.join(' ' * (indent * (level + 1)) + i for i in items)
        return '{\n' + inner + '\n' + ' ' * (indent * level) + '}'
    if isinstance(obj, frozenset):
        if not obj:
            return 'frozenset()'
        items = []
        for item in obj:
            items.append(_format(item, indent, width, depth, compact, level + 1))
        inner = ',\n'.join(' ' * (indent * (level + 1)) + i for i in items)
        return 'frozenset({\n' + inner + '\n' + ' ' * (indent * level) + '})'
    return repr(obj)
