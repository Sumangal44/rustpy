# json mock

def dumps(obj):
    if isinstance(obj, dict):
        items = []
        for k, v in obj.items():
            val_str = dumps(v)
            items.append('"' + str(k) + '": ' + val_str)
        return "{" + ", ".join(items) + "}"
    elif isinstance(obj, list):
        return "[" + ", ".join(dumps(x) for x in obj) + "]"
    elif isinstance(obj, str):
        return '"' + obj + '"'
    else:
        return str(obj)

def loads(s):
    py_str = s.replace("true", "True").replace("false", "False").replace("null", "None")
    return eval(py_str)
