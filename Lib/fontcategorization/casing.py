import unicodedata

exceptions_c2sc = [
    0x2126,  # Ohm
]
exceptions_smcp = [
    0x00B5,  # micro
    0x2113,  # liter
    0x1FBE,  # prosgegrammeni
    0x192,  # florin
]


def _smcp_coverage(ttFont, vhb):
    """Return percentage of actual smcp coverage."""
    cmap = ttFont.getBestCmap()
    eligible_glyphs = 0
    covered_glyphs = 0
    for unicode in cmap:
        char = chr(unicode)
        if unicodedata.category(char) == "Ll" and not unicode in exceptions_smcp:
            eligible_glyphs += 1
            if vhb.str(char) != vhb.str(char, {"features": {"smcp": True}}):
                covered_glyphs += 1

    return covered_glyphs / eligible_glyphs


def smcp(ttFont, vhb):
    """Return True if the font has a functioning smcp feature."""
    return _smcp_coverage(ttFont, vhb)


def _c2sc_coverage(ttFont, vhb):
    """Return percentage of actual c2sc coverage."""
    cmap = ttFont.getBestCmap()
    eligible_glyphs = 0
    covered_glyphs = 0
    for unicode in cmap:
        char = chr(unicode)
        if unicodedata.category(char) == "Lu" and not unicode in exceptions_c2sc:
            eligible_glyphs += 1
            if vhb.str(char) != vhb.str(char, {"features": {"c2sc": True}}):
                covered_glyphs += 1

    return covered_glyphs / eligible_glyphs


def c2sc(ttFont, vhb):
    """Return True if the font has a functioning c2sc feature."""
    return _c2sc_coverage(ttFont, vhb)


def _case_coverage(ttFont, vhb):
    """Return percentage of glyphs of actual case-sensitive punctuation coverage."""
    cmap = ttFont.getBestCmap()
    eligible_glyphs = 0
    covered_glyphs = 0
    for unicode in cmap:
        char = chr(unicode)
        if unicodedata.category(char).startswith("P"):  # Punctuation
            eligible_glyphs += 1

            buf1 = vhb.shape(char, {"features": {}})
            buf2 = vhb.shape(char, {"features": {"case": True}})

            if vhb.serialize_buf(buf1) != vhb.serialize_buf(buf2):
                covered_glyphs += 1

    return covered_glyphs / eligible_glyphs


def case(ttFont, vhb):
    """Return True if the font has a functioning case sensitive punctuation feature."""
    return _case_coverage(ttFont, vhb)
