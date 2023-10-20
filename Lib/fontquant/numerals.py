from fontquant import Metric, Percentage, Boolean, String

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


def vertical_variance(ttFont, vhb, features=None):
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


def horizontal_variance(ttFont, vhb, features=None):
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


class PON_CHECK(Metric):
    """\
    Returns a boolean of whether or not the font has functioning set of _proportional oldstyle_ numerals,
    either by default or activatable by the `onum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Proportional Oldstyle Numerals"
    keyword = PON
    data_type = Boolean

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": (
                pon_matrix(self.ttFont, self.vhb)
                and numeral_style_heuristics(self.ttFont, self.vhb, PON_MATRIX) == PON
                or default_numerals(self.ttFont, self.vhb) == PON
            )
        }

        return dictionary


def ton_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TON_MATRIX, [])


class TON_CHECK(Metric):
    """\
    Returns a boolean of whether or not the font has functioning set of _tabular oldstyle_ numerals,
    either by default or activatable by the `onum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Tabular Oldstyle Numerals"
    keyword = TON
    data_type = Boolean

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": (
                ton_matrix(self.ttFont, self.vhb)
                and numeral_style_heuristics(self.ttFont, self.vhb, TON_MATRIX) == TON
                or default_numerals(self.ttFont, self.vhb) == TON
            )
        }
        return dictionary


def pln_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, PLN_MATRIX, [])


class PLN_CHECK(Metric):
    """\
    Returns a boolean of whether or not the font has functioning set of _proportional lining_ numerals,
    either by default or activatable by the `lnum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Proportional Lining Numerals"
    keyword = PLN
    data_type = Boolean

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": (
                pln_matrix(self.ttFont, self.vhb)
                and numeral_style_heuristics(self.ttFont, self.vhb, PLN_MATRIX) == PLN
                or default_numerals(self.ttFont, self.vhb) == PLN
            )
        }
        return dictionary


def tln_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TLN_MATRIX, [])


class TLN_CHECK(Metric):
    """\
    Returns a boolean of whether or not the font has functioning set of _tabular lining_ numerals,
    either by default or activatable by the `lnum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't.
    """

    name = "Tabular Lining Numerals"
    keyword = TLN
    data_type = Boolean

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": (
                tln_matrix(self.ttFont, self.vhb)
                and numeral_style_heuristics(self.ttFont, self.vhb, TLN_MATRIX) == TLN
                or default_numerals(self.ttFont, self.vhb) == TLN
            )
        }
        return dictionary


# def sc(ttFont, vhb):
#     return vhb.str(NUMERALS) != vhb.str(NUMERALS, {"features": {"smcp": True}})


def numeral_style_heuristics(ttFont, vhb, features=None):
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


class DEFAULT_NUMERALS(Metric):
    """\
    Returns the default numeral set
    (out of `proportional_oldstyle`, `tabular_oldstyle`, `proportional_lining`, `tabular_lining`).
    """

    name = "Default Numerals"
    keyword = "default_numerals"
    data_type = String
    example_value = "proportional_lining"

    def value(self, includes=None, excludes=None):
        dictionary = {"value": self.shape_value(default_numerals(self.ttFont, self.vhb))}
        return dictionary


class SLASHED_ZERO(Metric):
    """\
    Returns percentage of feature combinations that shape the slashed zero.
    Here, the `zero` feature is used alone and in combination with other numeral-related features,
    if supported by the font, currently `sups`, `sinf`, `frac`. If so, the additional features are listed
    in the `checked_additional_features` key.
    """

    name = "Slashed Zero"
    keyword = "slashed_zero"
    data_type = Percentage
    interpretation_hint = """\
        A professional font should reach a value of `1.0` here."""

    def value(self, includes=None, excludes=None):
        # TODO:
        # These aren't all useful combinations yet.
        # Add more here in the future.
        features = [
            ["0", ["zero"], []],
        ]
        checked_additional_features = []
        # Add sups if supported
        if differs(self.vhb, "0", ["sups"], []):
            features.append(["0", ["zero", "sups"], ["sups"]])
            checked_additional_features.append("sups")
        # Add sinf if supported
        if differs(self.vhb, "0", ["sinf"], []):
            features.append(["0", ["zero", "sinf"], ["sinf"]])
            checked_additional_features.append("sinf")
        # Add arbitrary_fractions if supported
        arbitrary_fractions_check = self.base().find_check("numerals/arbitrary_fractions")
        if arbitrary_fractions_check.value()["value"]:
            features.append(["0/1", ["zero", "frac"], ["frac"]])
            features.append(["1/0", ["zero", "frac"], ["frac"]])
            checked_additional_features.append("frac")

        dictionary = {
            "value": self.shape_value(
                sum([differs(self.vhb, string, off, on) for string, on, off in features]) / len(features)
            )
        }
        if checked_additional_features:
            dictionary["checked_additional_features"] = checked_additional_features
        return dictionary


class ENCODED_FRACTIONS_CHECK(Metric):
    """\
    Returns percentage of encoded default fractions (e.g. ½) that are shaped by the `frac` feature.
    """

    name = "Encoded Fractions"
    keyword = "encoded_fractions"
    data_type = Percentage
    interpretation_hint = """\
        Consider encoded fractions to be _inferior_ to arbitrary fractions
        as checked by the `numerals/arbitrary_fractions` check.
        For a professional font, ignore this check."""

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": self.shape_value(
                sum(
                    [
                        self.vhb.str(string) != self.vhb.str(string, {"features": {"frac": True}})
                        for string in ENCODED_FRACTIONS
                    ]
                )
                / len(ENCODED_FRACTIONS)
            )
        }
        return dictionary


class EXTENDED_FRACTIONS(Metric):
    """\
    Returns boolean of whether or not arbitrary fractions (e.g. 12/99) can be shaped by the `frac` feature.
    """

    name = "Arbitrary Fractions"
    keyword = "arbitrary_fractions"
    data_type = Boolean
    interpretation_hint = """\
        Consider arbitrary fractions to be _superior_ to encoded fractions
        as checked by the `numerals/encoded_fractions` check."""

    def value(self, includes=None, excludes=None):
        dictionary = {
            "value": all(
                [
                    self.vhb.str(string) != self.vhb.str(string, {"features": {"frac": True}})
                    for string in ("12/99", "1/3", "1234/9876", "1/1234567890")
                ]
            )
        }
        return dictionary


class SINF(Metric):
    """\
    Returns the percentage of numerals that get shaped by the `sinf` feature.
    """

    name = "Inferior Numerals"
    keyword = "inferior_numerals"
    data_type = Percentage
    interpretation_hint = """\
        Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%).
        _A partial support is useless in practice._"""

    def value(self, includes=None, excludes=None):
        covered = 0
        for numeral in NUMERALS:
            if self.vhb.str(numeral) != self.vhb.str(numeral, {"features": {"sinf": True}}):
                covered += 1
        dictionary = {"value": self.shape_value(covered / len(NUMERALS))}
        return dictionary


class SUPS(Metric):
    """\
    Returns the percentage of numerals that get shaped by the `sups` feature.
    """

    name = "Superior Numerals"
    keyword = "superior_numerals"
    data_type = Percentage
    interpretation_hint = """\
        Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%).
        _A partial support is useless in practice._"""

    def value(self, includes=None, excludes=None):
        covered = 0
        for numeral in NUMERALS:
            if self.vhb.str(numeral) != self.vhb.str(numeral, {"features": {"sups": True}}):
                covered += 1
        dictionary = {"value": self.shape_value(covered / len(NUMERALS))}
        return dictionary


class Numerals(Metric):
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
