class writer:
    def __init__(self, f):
        self._f = f

    def writerow(self, row):
        parts = []
        for v in row:
            parts.append(str(v))
        line = ','.join(parts) + '\n'
        fout = self._f
        fout.write(line)


class reader:
    def __init__(self, lines):
        self._lines = lines
        self._iter = iter(lines)

    def __iter__(self):
        return self

    def __next__(self):
        itobj = self._iter
        line = next(itobj)
        return line.rstrip('\n').split(',')
