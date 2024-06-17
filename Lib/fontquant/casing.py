from fontquant import Metric, Percentage, Boolean, String
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
    0x3BC,  # mu
]


class Unicase(Metric):
    """\
    Reports whether or not a font is unicase (lowercase and uppercase letters being of the same height).
    To check for different shapes of lowercase letters compared to uppercase, use the `Lowercase Shapes` metric.
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
        failed_glyphs = []
        for unicode in cmap:
            char = chr(unicode)
            if unicodedata.category(char) == "Ll" and unicode not in exceptions_smcp:
                eligible_glyphs += 1
                if self.vhb.str(char) != self.vhb.str(char, {"features": {"smcp": True}}):
                    covered_glyphs += 1
                else:
                    failed_glyphs.append(char)

        dictionary = {"value": self.shape_value(covered_glyphs / eligible_glyphs), "failed": failed_glyphs}
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
        failed_glyphs = []
        for unicode in cmap:
            char = chr(unicode)
            if unicodedata.category(char) == "Lu" and unicode not in exceptions_c2sc:
                eligible_glyphs += 1
                if self.vhb.str(char) != self.vhb.str(char, {"features": {"c2sc": True}}):
                    covered_glyphs += 1
                else:
                    failed_glyphs.append(char)

        dictionary = {"value": self.shape_value(covered_glyphs / eligible_glyphs), "failed": failed_glyphs}
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


class LowercaseShapes(Metric):
    """\
    Returns the shapes of lowercase-codepoint characters. Possible values are `uppercase`, `lowercase`, and `smallcaps`.
    This check compares the contour count (and the average height) of uppercase and lowercase letters,
    so it compares actual outline construction. In that sense it's different from the `Unicase` metric which only looks
    at dimensions and allows upper/lowercase shapes to be different as long as they are of similar height.
    """

    name = "Lowercase Shapes"
    keyword = "lowercase_shapes"
    data_type = String

    def value(self, includes=None, excludes=None):

        from fontquant.helpers.bezier import removeOverlaps

        UC_upperbounds = []
        UC_lowerbounds = []
        LC_upperbounds = []
        LC_lowerbounds = []
        UC_contours_in_lowercase = []

        for character in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            UC = removeOverlaps(self.vhb.buf_to_bezierpaths(self.vhb.shape(character, {"features": {}})))
            lowercase = removeOverlaps(
                self.vhb.buf_to_bezierpaths(self.vhb.shape(character.lower(), {"features": {}}))
            )

            if len(UC) == len(lowercase):
                UC_contours_in_lowercase.append(1)
            else:
                UC_contours_in_lowercase.append(0)

            # Add bounds to lists
            UC_upperbound = max([path.bounds().top for path in UC])
            UC_lowerbound = min([path.bounds().bottom for path in UC])
            LC_upperbound = max([path.bounds().top for path in lowercase])
            LC_lowerbound = min([path.bounds().bottom for path in lowercase])

            UC_upperbounds.append(UC_upperbound)
            UC_lowerbounds.append(UC_lowerbound)
            LC_upperbounds.append(LC_upperbound)
            LC_lowerbounds.append(LC_lowerbound)

            UC_average_height = (sum(UC_upperbounds) - sum(UC_lowerbounds)) / (2 * len(UC_upperbounds))
            LC_average_height = (sum(LC_upperbounds) - sum(LC_lowerbounds)) / (2 * len(LC_upperbounds))
            LC_to_UC_height_ratio = LC_average_height / UC_average_height

        UC_LC_outline_fidelity = sum(UC_contours_in_lowercase) / len(UC_contours_in_lowercase)

        # LC shapes adhere to UC outlines over 90% of the time
        # (2 off out of 26 is 92.3%)
        if UC_LC_outline_fidelity > 0.9:
            # Average height of LC shapes is less than 80% of UC shapes
            # (real LC actually have a higher average height than SC because of ascenders and descenders)
            if LC_to_UC_height_ratio < 0.8:
                LC = "smallcaps"
            else:
                LC = "uppercase"
        else:
            LC = "lowercase"

        dictionary = {"value": LC}
        return dictionary


class Casing(Metric):
    name = "Casing"
    keyword = "casing"
    children = [SMCP, C2SC, CASE, Unicase, LowercaseShapes]
