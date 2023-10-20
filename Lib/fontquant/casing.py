from fontquant import Metric, Percentage
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


class SMCP(Metric):
    """\
    Returns the percentage of characters that are lowercase letters (`Ll`)
    and get shaped by the `smcp` feature.
    """

    name = "SmallCaps"
    keyword = "smallcaps"
    data_type = Percentage
    interpretation_hint = """\
    Consider fonts to have a functioning `smcp` feature if the value is above `0.95` (95%),
    as there are some characters that are lowercase letters but don't get shaped by the `smcp` feature, e.g. `florin`.
    Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.py` to see your
    values rise."""

    def value(self, includes=None, excludes=None):
        cmap = self.ttFont.getBestCmap()
        eligible_glyphs = 0
        covered_glyphs = 0
        for unicode in cmap:
            char = chr(unicode)
            if unicodedata.category(char) == "Ll" and unicode not in exceptions_smcp:
                eligible_glyphs += 1
                if self.vhb.str(char) != self.vhb.str(char, {"features": {"smcp": True}}):
                    covered_glyphs += 1

        dictionary = {"value": self.shape_value(covered_glyphs / eligible_glyphs)}
        return dictionary


class C2SC(Metric):
    """\
    Returns the percentage of characters that are uppercase letters (`Lu`)
    and get shaped by the `c2sc` feature.
    """

    name = "Caps-To-SmallCaps"
    keyword = "caps-to-smallcaps"
    data_type = Percentage
    interpretation_hint = """\
    Consider fonts to have a functioning `c2sc` feature if the value is above `0.95` (95%),
    as there are some characters that are uppercase letters but don't typically get shaped by the `c2sc` feature,
    e.g. `Ohm`.
    Alternatively, consider contributing exceptions to the `exceptions_c2sc` variable in `casing.py` to see your
    values rise."""

    def value(self, includes=None, excludes=None):
        cmap = self.ttFont.getBestCmap()
        eligible_glyphs = 0
        covered_glyphs = 0
        for unicode in cmap:
            char = chr(unicode)
            if unicodedata.category(char) == "Lu" and unicode not in exceptions_c2sc:
                eligible_glyphs += 1
                if self.vhb.str(char) != self.vhb.str(char, {"features": {"c2sc": True}}):
                    covered_glyphs += 1

        dictionary = {"value": self.shape_value(covered_glyphs / eligible_glyphs)}
        return dictionary


class CASE(Metric):
    """\
    Returns the percentage of characters that are punctuation (`P*`)
    and get shaped by the `case` feature.
    """

    name = "Case-Sensitive Punctuation"
    keyword = "case_sensitive_punctuation"
    data_type = Percentage

    def value(self, includes=None, excludes=None):
        cmap = self.ttFont.getBestCmap()
        eligible_glyphs = 0
        covered_glyphs = 0
        for unicode in cmap:
            char = chr(unicode)
            if unicodedata.category(char).startswith("P"):  # Punctuation
                eligible_glyphs += 1

                buf1 = self.vhb.shape(char, {"features": {}})
                buf2 = self.vhb.shape(char, {"features": {"case": True}})

                if self.vhb.serialize_buf(buf1) != self.vhb.serialize_buf(buf2):
                    covered_glyphs += 1

        dictionary = {"value": self.shape_value(covered_glyphs / eligible_glyphs)}
        return dictionary


class Casing(Metric):
    name = "Casing"
    keyword = "casing"
    children = [SMCP, C2SC, CASE]
