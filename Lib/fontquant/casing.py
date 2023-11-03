from fontquant import Metric, Percentage, Boolean
from beziers.path import BezierPath
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


class Unicase(Metric):
    """\
    Reports whether or not a font is unicase (lowercase and uppercase letters being of the same height).
    """

    name = "Unicase"
    keyword = "unicase"
    data_type = Boolean
    exclude_uppercase = ["Q", "J", "Ŋ"]
    exclude_lowercase = ["μ", "ŋ", "ƒ"]

    def value(self, includes=None, excludes=None):
        cmap = self.ttFont.getBestCmap()

        highest_list = []
        lowest_list = []

        unicase = []
        height_threshold = self.ttFont["head"].unitsPerEm * 0.1
        chars = []

        for unicode in cmap:
            lowest = self.ttFont["head"].unitsPerEm * 2
            highest = -self.ttFont["head"].unitsPerEm
            char = chr(unicode)

            if (
                unicodedata.category(char) == "Lu"
                and not unicodedata.decomposition(char)
                and char not in self.exclude_uppercase
            ):
                # Go through all characters and find the highest and lowest point
                try:
                    paths = BezierPath.fromFonttoolsGlyph(self.ttFont, cmap[ord(char)])
                    for path in paths:
                        bounds = path.bounds()
                        if bounds and bounds.bl and bounds.tr:
                            lowest = min(lowest, bounds.bl.y)
                            highest = max(highest, bounds.tr.y)

                    highest_list.append(highest)
                    lowest_list.append(lowest)
                except IndexError:
                    pass

        # Averages
        if highest_list and lowest_list:
            highest_average = sum(highest_list) / len(highest_list)
            lowest_average = sum(lowest_list) / len(lowest_list)

            # Go through all characters again to see if the are within the average
            for unicode in cmap:
                char = chr(unicode)
                if (
                    unicodedata.category(char) in ("Lu", "Ll")
                    and not unicodedata.decomposition(char)
                    and char not in self.exclude_uppercase
                    and char not in self.exclude_lowercase
                ):
                    chars.append(char)

                    try:
                        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, cmap[ord(char)])

                        in_bounds = []
                        lowest = self.ttFont["head"].unitsPerEm * 2
                        highest = -self.ttFont["head"].unitsPerEm
                        for path in paths:
                            if path.length:
                                bounds = path.bounds()
                                if bounds and bounds.bl and bounds.tr:
                                    lowest = min(lowest, bounds.bl.y)
                                    highest = max(highest, bounds.tr.y)
                        if (
                            abs(lowest - lowest_average) < height_threshold
                            and abs(highest - highest_average) < height_threshold
                        ):
                            in_bounds.append(True)
                        else:
                            in_bounds.append(False)
                        if all(in_bounds):
                            unicase.append(char)
                    except IndexError:
                        pass

            return {"value": len(unicase) / len(chars) > 0.95}

        return {"value": False}


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
    children = [SMCP, C2SC, CASE, Unicase]
