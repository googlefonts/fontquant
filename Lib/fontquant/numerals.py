from fontquant import Check

# Settings:

# How much of the height of an "x" must the upper and lower bbox variance of numerals
# be to be considered PON?
PON_THRESHOLD = 0.1

# How much may the width variance of table figures be to be considered TLN_MATRIX?
# Measured in variance / UPM.
TLN_THRESHOLD = 0.005

# Constants:
NUMERALS = "0123456789"
ENCODED_FRACTIONS = (
    "1/4",
    "1/2",
    "3/4",
    "1/7",
    "1/9",
    "1/10",
    "1/3",
    "2/3",
    "1/5",
    "2/5",
    "3/5",
    "4/5",
    "1/6",
    "5/6",
    "1/8",
    "3/8",
    "5/8",
    "7/8",
    "1/",
    "0/3",
)

# Numeral sets:
PON = "proportional_oldstyle"
TON = "tabular_oldstyle"
PLN = "proportional_lining"
TLN = "tabular_lining"

# Activation matrix
PON_MATRIX = ("onum", "pnum")
TON_MATRIX = ("onum", "tnum")
PLN_MATRIX = ("lnum", "pnum")
TLN_MATRIX = ("lnum", "tnum")


def vertical_variance(ttFont, vhb, features):
    """Determine numerals' upper and lower bbox variance"""
    upper = []
    lower = []
    feature_dict = {}
    for feature in features or []:
        feature_dict[feature] = True
    for numeral in NUMERALS:
        x_min, x_max, y_min, y_max = vhb.bbox(numeral, {"features": feature_dict})
        upper.append(y_max)
        lower.append(y_min)

    return max(upper) - min(upper), max(lower) - min(lower)


def width(vhb, string):
    """Determine numerals' width"""
    x_min, x_max, y_min, y_max = vhb.bbox(string)
    return x_max + x_min


def horizontal_variance(ttFont, vhb, features):
    """Determine numerals' horizontal bbox variance"""
    widths = []
    feature_dict = {}
    for feature in features or []:
        feature_dict[feature] = True
    for numeral in NUMERALS:
        widths.append(vhb.buf_to_width(vhb.shape(numeral, {"features": feature_dict})))
    return max(widths) - min(widths)


def differs(vhb, string, features1, features2):
    """Return True if the two featuresets differ."""
    feature_dict_1 = {}
    feature_dict_2 = {}
    for feature in features1:
        feature_dict_1[feature] = True
    for feature in features2:
        feature_dict_2[feature] = True
    return vhb.str(string, {"features": feature_dict_1}) != vhb.str(string, {"features": feature_dict_2})


def same(vhb, string, features1, features2):
    return not differs(vhb, string, features1, features2)


def pon_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, PON_MATRIX, [])


class PON_CHECK(Check):
    """\
    Returns a boolean of whether or not the font has functioning set of _proportional oldstyle_ numerals,
    either by default or activatable by the `onum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Proportional Oldstyle Numerals"
    keyword = PON

    def JSON(self):
        return (
            pon_matrix(self.ttFont, self.vhb)
            and numeral_style_heuristics(self.ttFont, self.vhb, PON_MATRIX) == PON
            or default_numerals(self.ttFont, self.vhb) == PON
        )


def ton_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TON_MATRIX, [])


class TON_CHECK(Check):
    """\
    Returns a boolean of whether or not the font has functioning set of _tabular oldstyle_ numerals,
    either by default or activatable by the `onum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Tabular Oldstyle Numerals"
    keyword = TON

    def JSON(self):
        return (
            ton_matrix(self.ttFont, self.vhb)
            and numeral_style_heuristics(self.ttFont, self.vhb, TON_MATRIX) == TON
            or default_numerals(self.ttFont, self.vhb) == TON
        )


def pln_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, PLN_MATRIX, [])


class PLN_CHECK(Check):
    """\
    Returns a boolean of whether or not the font has functioning set of _proportional lining_ numerals,
    either by default or activatable by the `lnum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Proportional Lining Numerals"
    keyword = PLN

    def JSON(self):
        return (
            pln_matrix(self.ttFont, self.vhb)
            and numeral_style_heuristics(self.ttFont, self.vhb, PLN_MATRIX) == PLN
            or default_numerals(self.ttFont, self.vhb) == PLN
        )


def tln_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TLN_MATRIX, [])


class TLN_CHECK(Check):
    """\
    Returns a boolean of whether or not the font has functioning set of _tabular lining_ numerals,
    either by default or activatable by the `lnum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Tabular Lining Numerals"
    keyword = TLN

    def JSON(self):
        return (
            tln_matrix(self.ttFont, self.vhb)
            and numeral_style_heuristics(self.ttFont, self.vhb, TLN_MATRIX) == TLN
            or default_numerals(self.ttFont, self.vhb) == TLN
        )


# def sc(ttFont, vhb):
#     return vhb.str(NUMERALS) != vhb.str(NUMERALS, {"features": {"smcp": True}})


def numeral_style_heuristics(ttFont, vhb, features):
    x_bbox = vhb.bbox("x")
    x_height = x_bbox[3] - x_bbox[2]
    upper_variance, lower_variance = vertical_variance(ttFont, vhb, features)

    # Vertical:
    # Compare upper and lower bbox variance to x-height
    if upper_variance > x_height * PON_THRESHOLD and lower_variance > x_height * PON_THRESHOLD:
        vertical = "onum"
    else:
        vertical = "lnum"

    # Allow some variance in width
    if horizontal_variance(ttFont, vhb, features) / ttFont["head"].unitsPerEm < TLN_THRESHOLD:
        horizontal = "tnum"
    else:
        horizontal = "pnum"

    if (vertical, horizontal) == PON_MATRIX:
        return PON
    elif (vertical, horizontal) == TON_MATRIX:
        return TON
    elif (vertical, horizontal) == PLN_MATRIX:
        return PLN
    elif (vertical, horizontal) == TLN_MATRIX:
        return TLN


def default_numerals(ttFont, vhb):
    numeralsets = [
        PON,
        TON,
        PLN,
        TLN,
    ]

    # Remove numeralsets from the full list that are available
    if pon_matrix(ttFont, vhb):
        numeralsets.remove(PON)
    if ton_matrix(ttFont, vhb):
        numeralsets.remove(TON)
    if pln_matrix(ttFont, vhb):
        numeralsets.remove(PLN)
    if tln_matrix(ttFont, vhb):
        numeralsets.remove(TLN)

    # If only one numeral set remains, return it
    if len(numeralsets) == 1:
        return numeralsets[0]

    # otherwise use heuristics
    else:
        return numeral_style_heuristics(ttFont, vhb)


class DEFAULT_NUMERALS(Check):
    """\
    Returns the default numeral set
    (out of `proportional_oldstyle`, `tabular_oldstyle`, `proportional_lining`, `tabular_lining`).
    """

    name = "Default Numerals"
    keyword = "default_numerals"

    def JSON(self):
        return default_numerals(self.ttFont, self.vhb)


class SLASHED_ZERO(Check):
    """\
    Returns percentage (as float 0—1) of feature combinations that shape the slashed zero.
    Here, the `zero` feature is used alone and in combination with other numeral-related features,
    currently `subs` and `sinf`.
    """

    name = "Slashed Zero"
    keyword = "slashed_zero"

    def JSON(self):
        # TODO:
        # These aren't all useful combinations yet.
        # Add more here in the future.
        features = [
            [["zero"], []],
            [["zero", "subs"], ["zero"]],
            [["zero", "sinf"], ["zero"]],
        ]
        return sum([differs(self.vhb, "0", off, on) for off, on in features]) / len(features)


class ENCODED_FRACTIONS_CHECK(Check):
    """\
    Returns percentage (as float 0—1) of encoded default fractions (e.g. ½) that are shaped by the `frac` feature.
    """

    name = "Encoded Fractions"
    keyword = "encoded_fractions"
    interpretation_hint = """\
        Consider encoded fractions to be inferior to arbitrary fractions
        as checked by the `numerals/arbitrary_fractions` check."""

    def JSON(self):
        return self.ttFont.has_feature("frac") and sum(
            [
                self.vhb.str(string) != self.vhb.str(string, {"features": {"frac": True}})
                for string in ENCODED_FRACTIONS
            ]
        ) / len(ENCODED_FRACTIONS)


class EXTENDED_FRACTIONS(Check):
    """\
    Returns boolean of whether or not arbitrary fractions (e.g. 12/99) can be shaped by the `frac` feature.
    """

    name = "Arbitrary Fractions"
    keyword = "arbitrary_fractions"
    interpretation_hint = """\
        Consider arbitrary fractions to be superior to encoded fractions
        as checked by the `numerals/encoded_fractions` check."""

    def JSON(self):
        return all(
            [
                self.vhb.str(string) != self.vhb.str(string, {"features": {"frac": True}})
                for string in ("12/99", "1/3", "1234/9876", "1/1234567890")
            ]
        )


class SINF(Check):
    """\
    Returns the percentage (as float 0—1) of numerals that get shaped by the `sinf` feature.
    """

    name = "Inferior Numerals"
    keyword = "inferiors"
    interpretation_hint = """\
        Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%).
        A partial support is useless in practice."""

    def JSON(self):
        covered = 0
        for numeral in NUMERALS:
            if self.vhb.str(numeral) != self.vhb.str(numeral, {"features": {"sinf": True}}):
                covered += 1
        return covered / len(NUMERALS)


class SUPS(Check):
    """\
    Returns the percentage (as float 0—1) of numerals that get shaped by the `sups` feature.
    """

    name = "Superior Numerals"
    keyword = "superiors"
    interpretation_hint = """\
        Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%).
        A partial support is useless in practice."""

    def JSON(self):
        covered = 0
        for numeral in NUMERALS:
            if self.vhb.str(numeral) != self.vhb.str(numeral, {"features": {"sups": True}}):
                covered += 1
        return covered / len(NUMERALS)


class Numerals(Check):
    name = "Numerals"
    keyword = "numerals"

    children = [
        PON_CHECK,
        TON_CHECK,
        PLN_CHECK,
        TLN_CHECK,
        DEFAULT_NUMERALS,
        SUPS,
        SINF,
        ENCODED_FRACTIONS_CHECK,
        EXTENDED_FRACTIONS,
        SLASHED_ZERO,
    ]
