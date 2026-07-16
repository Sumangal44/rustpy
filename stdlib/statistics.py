# statistics mock

def mean(nums):
    return sum(nums) / len(nums)

def median(nums):
    s = sorted(nums)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    else:
        return (s[n // 2 - 1] + s[n // 2]) / 2
