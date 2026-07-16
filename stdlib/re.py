# re mock

class Match:
    def __init__(self, s):
        self._s = s
    def __repr__(self):
        return f"<re.Match object; span=(0, {len(self._s)}), match='{self._s}'>"

def findall(pattern, text):
    if pattern == "\\d+":
        digits = ""
        for c in text:
            if c.isdigit():
                digits += c
        return [digits] if digits else []
    return []

def search(pattern, text):
    if pattern in text:
        return Match(pattern)
    return None
