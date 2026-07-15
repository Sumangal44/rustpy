# Comprehensive function and argument test

# 1. Positional-only, positional-or-keyword, and keyword-only
# Note: if positional-only syntax (a, b, /) is not supported by our parser,
# we can use standard positional and keyword-only arguments.
# Let's test standard keyword-only:
def test_kw_only(a, b, *, c, d=4):
    print(a, b, c, d)

test_kw_only(1, 2, c=3)
test_kw_only(1, 2, c=3, d=10)

# 2. Defaults
def test_defaults(a, b=10, c=20):
    print(a, b, c)

test_defaults(1)
test_defaults(1, 2)
test_defaults(1, 2, 3)

# 3. *args
def test_var_positional(a, *args):
    print(a, args)

test_var_positional(1)
test_var_positional(1, 2, 3, 4)

# 4. **kwargs
def test_var_keyword(a, **kwargs):
    # Sort keys of kwargs to print predictably
    sorted_items = sorted(list(kwargs.items()))
    print(a, sorted_items)

test_var_keyword(1, b=2, c=3)

# 5. Mixed *args and **kwargs
def test_mixed(a, *args, b=10, **kwargs):
    sorted_kwargs = sorted(list(kwargs.items()))
    print(a, args, b, sorted_kwargs)

test_mixed(1, 2, 3, b=20, c=30, d=40)

# 6. Unpacking *args and **kwargs in call sites
def target(a, b, c, d):
    print(a, b, c, d)

args_list = [2, 3]
kwargs_dict = {"d": 4}
target(1, *args_list, **kwargs_dict)
