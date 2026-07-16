# hashlib mock

class SHA256:
    def __init__(self, data):
        self.data = data
    def hexdigest(self):
        return "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

def sha256(data=b""):
    return SHA256(data)
