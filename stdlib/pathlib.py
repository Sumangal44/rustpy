# pathlib mock
import os

class Path:
    def __init__(self, path="."):
        self._path = path

    def __repr__(self):
        return f"PosixPath('{self._path}')"

    def __str__(self):
        return self._path

    def iterdir(self):
        files = os.listdir(self._path)
        for f in files:
            yield Path(self._path + "/" + f)

    def write_text(self, text):
        with open(self._path, "w") as f:
            f.write(text)

    def read_text(self):
        with open(self._path, "r") as f:
            return f.read()
