def ss(ttFont, vhb):
    """Return number of ssXX features."""
    return sum([ttFont.has_feature(f"ss{str(i).zfill(2)}") for i in range(1, 21)])


def salt(ttFont, vhb):
    """Return True if the font has a functioning salt feature."""
    return ttFont.has_feature("salt")


def calt(ttFont, vhb):
    """Return True if the font has a functioning calt feature."""
    return ttFont.has_feature("calt")
