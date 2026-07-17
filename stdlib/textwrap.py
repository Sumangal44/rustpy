def wrap(text, width=70, **kwargs):
    paragraphs = text.split('\n')
    result = []
    for para in paragraphs:
        words = para.split()
        lines = []
        current = []
        current_len = 0
        for word in words:
            if current_len + len(word) + (1 if current else 0) <= width:
                current.append(word)
                current_len += len(word) + (1 if current_len > 0 else 0)
            else:
                if current:
                    lines.append(' '.join(current))
                current = [word]
                current_len = len(word)
        if current:
            lines.append(' '.join(current))
        result.extend(lines)
    return result


def fill(text, width=70, **kwargs):
    return '\n'.join(wrap(text, width, **kwargs))


def dedent(text):
    lines = text.split('\n')
    indent = None
    for line in lines:
        stripped = line.strip()
        if stripped:
            line_indent = len(line) - len(line.lstrip())
            if indent is None or line_indent < indent:
                indent = line_indent
    if indent is None or indent == 0:
        return text
    return '\n'.join(line[indent:] if line.strip() else line for line in lines)


def indent(text, prefix, predicate=None):
    if predicate is None:
        predicate = lambda line: True
    lines = text.split('\n')
    return '\n'.join(prefix + line if predicate(line) else line for line in lines)


class TextWrapper:
    def __init__(self, width=70, **kwargs):
        self.width = width

    def wrap(self, text):
        return wrap(text, self.width)

    def fill(self, text):
        return fill(text, self.width)
