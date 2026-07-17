_filters = []
_showwarning = None


class WarningMessage:
    def __init__(self, message, category, filename, lineno, file=None, line=None):
        self.message = message
        self.category = category
        self.filename = filename
        self.lineno = lineno
        self.file = file
        self.line = line


def warn(message, category=None, stacklevel=1):
    if category is None:
        category = UserWarning
    if isinstance(message, str):
        message = category(message)
    import sys as _sys
    _sys.stderr.write(f"{category.__name__}: {message}\n")


def warn_explicit(message, category, filename, lineno, module=None, registry=None, module_globals=None):
    import sys as _sys
    _sys.stderr.write(f"{filename}:{lineno}: {category.__name__}: {message}\n")


def simplefilter(action, category=None, lineno=0, append=False):
    if category is None:
        category = Warning
    _filters.insert(0, (action, category))


def filterwarnings(action, message="", category=None, module="", lineno=0, append=False):
    if category is None:
        category = Warning
    _filters.append((action, message, category, module, lineno))


def resetwarnings():
    _filters.clear()


class catch_warnings:
    def __init__(self, record=False, module=None):
        self._record = record
        self._module = module

    def __enter__(self):
        return self

    def __exit__(self, *exc_info):
        pass
