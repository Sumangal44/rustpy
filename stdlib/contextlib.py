class _GeneratorContextManager:
    def __init__(self, func, args, kwargs):
        self.gen = func(*args, **kwargs)

    def __enter__(self):
        try:
            return next(self.gen)
        except StopIteration:
            raise RuntimeError("generator didn't yield")

    def __exit__(self, typ, value, traceback):
        if typ is None:
            try:
                next(self.gen)
            except StopIteration:
                return False
            else:
                raise RuntimeError("generator didn't stop")
        else:
            if value is None:
                value = typ()
            try:
                self.gen.throw(typ, value, traceback)
            except StopIteration as exc:
                return exc is not value
            raise RuntimeError("generator didn't stop after throw()")


def contextmanager(func):
    def helper(*args, **kwargs):
        return _GeneratorContextManager(func, args, kwargs)
    return helper


class closing:
    def __init__(self, thing):
        self.thing = thing

    def __enter__(self):
        return self.thing

    def __exit__(self, *exc_info):
        self.thing.close()


class nullcontext:
    def __init__(self, enter_result=None):
        self.enter_result = enter_result

    def __enter__(self):
        return self.enter_result

    def __exit__(self, *excinfo):
        pass


class suppress:
    def __init__(self, *exceptions):
        self._exceptions = exceptions

    def __enter__(self):
        pass

    def __exit__(self, exctype, excval, traceback):
        if exctype is not None and issubclass(exctype, self._exceptions):
            return True
        return False


class ExitStack:
    def __init__(self):
        self._exit_callbacks = []

    def push(self, exit_func):
        self._exit_callbacks.append(exit_func)
        return exit_func

    def callback(self, callback, *args, **kwargs):
        def _exit(*exc):
            if exc[0] is None:
                callback(*args, **kwargs)
            return False
        self._exit_callbacks.append(_exit)
        return callback

    def enter_context(self, cm):
        result = cm.__enter__()
        self._exit_callbacks.append(cm.__exit__)
        return result

    def pop_all(self):
        stack = ExitStack()
        stack._exit_callbacks = self._exit_callbacks
        self._exit_callbacks = []
        return stack

    def close(self):
        self.__exit__(None, None, None)

    def __enter__(self):
        return self

    def __exit__(self, *exc_details):
        received_exc = exc_details[0] is not None
        for callback in reversed(self._exit_callbacks):
            try:
                if callback(*exc_details):
                    received_exc = False
                    exc_details = (None, None, None)
            except BaseException:
                received_exc = True
                if exc_details[0] is None:
                    exc_details = (type(None)(), type(None)(), type(None)())
        self._exit_callbacks.clear()
        if received_exc:
            raise exc_details[0](str(exc_details[1]) if exc_details[1] else "").with_traceback(exc_details[2])
