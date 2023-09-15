# Settings:

# How much of the height of an "x" must the upper and lower bbox variance of numerals
# be to be considered OSF?
OSF_THRESHOLD = 0.1

# How much may the width variance of table figures be to be considered TF?
# Measured in variance / UPM.
TF_THRESHOLD = 0.005

# Constants:
NUMERALS = "0123456789"

# Activation matrix
OSF = ("onum", "pnum")
TOSF = ("onum", "tnum")
LF = ("lnum", "pnum")
TF = ("lnum", "tnum")


def vertical_variance(ttFont, vhb, features=[]):
    """Determine numerals' upper and lower bbox variance"""
    upper = []
    lower = []
    feature_dict = {}
    for feature in features:
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


def horizontal_variance(ttFont, vhb, features=[]):
    """Determine numerals' horizontal bbox variance"""
    widths = []
    feature_dict = {}
    for feature in features:
        feature_dict[feature] = True
    for numeral in NUMERALS:
        widths.append(vhb.buf_to_width(vhb.shape(numeral, {"features": feature_dict})))
    return max(widths) - min(widths)


# Problem:
#
# The below code doesn not (yet) check whether numerals shaped by certain features
# are actually different from numerals shaped by other features.
# So it doesn't compare tnum and tosf to see of they're different.
# It only checks whether numerals activated by one combination of features
# are different from the default numerals.
# So it's not 100% accurate for non-standard fonts, but should work for standard fonts.


def differs(vhb, string, features1, features2):
    """Return True if the two featuresets differ."""
    feature_dict_1 = {}
    feature_dict_2 = {}
    for feature in features1:
        feature_dict_1[feature] = True
    for feature in features2:
        feature_dict_2[feature] = True
    return vhb.str(string, {"features": feature_dict_1}) != vhb.str(
        string, {"features": feature_dict_2}
    )


def same(vhb, string, features1, features2):
    return not differs(vhb, string, features1, features2)


def osf_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, OSF, [])


def osf(ttFont, vhb):
    return (
        osf_matrix(ttFont, vhb)
        and numeral_style_heuristics(ttFont, vhb, OSF) == "osf"
        or default_numerals(ttFont, vhb) == "osf"
    )


def tosf_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TOSF, [])


def tosf(ttFont, vhb):
    return (
        tosf_matrix(ttFont, vhb)
        and numeral_style_heuristics(ttFont, vhb, TOSF) == "tosf"
        or default_numerals(ttFont, vhb) == "tosf"
    )


def lf_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, LF, [])


def lf(ttFont, vhb):
    return (
        lf_matrix(ttFont, vhb)
        and numeral_style_heuristics(ttFont, vhb, LF) == "lf"
        or default_numerals(ttFont, vhb) == "lf"
    )


def tf_matrix(ttFont, vhb):
    return differs(vhb, NUMERALS, TF, [])


def tf(ttFont, vhb):
    return (
        tf_matrix(ttFont, vhb)
        and numeral_style_heuristics(ttFont, vhb, TF) == "tf"
        or default_numerals(ttFont, vhb) == "tf"
    )


def sc(ttFont, vhb):
    return vhb.str(NUMERALS) != vhb.str(NUMERALS, {"features": {"smcp": True}})


def numeral_style_heuristics(ttFont, vhb, features=[]):
    x_bbox = vhb.bbox("x")
    x_height = x_bbox[3] - x_bbox[2]
    upper_variance, lower_variance = vertical_variance(ttFont, vhb, features)

    # Vertical:
    # Compare upper and lower bbox variance to x-height
    if (
        upper_variance > x_height * OSF_THRESHOLD
        and lower_variance > x_height * OSF_THRESHOLD
    ):
        vertical = "onum"
    else:
        vertical = "lnum"

    # Allow some variance in width
    if (
        horizontal_variance(ttFont, vhb, features) / ttFont["head"].unitsPerEm
        < TF_THRESHOLD
    ):
        horizontal = "tnum"
    else:
        horizontal = "pnum"

    if (vertical, horizontal) == OSF:
        return "osf"
    elif (vertical, horizontal) == TOSF:
        return "tosf"
    elif (vertical, horizontal) == LF:
        return "lf"
    elif (vertical, horizontal) == TF:
        return "tf"


def default_numerals(ttFont, vhb):
    """Determine font's default numerals"""
    numeralsets = ["osf", "tosf", "lf", "tf"]

    # Remove numeralsets from the full list that are available
    if osf_matrix(ttFont, vhb):
        numeralsets.remove("osf")
    if tosf_matrix(ttFont, vhb):
        numeralsets.remove("tosf")
    if lf_matrix(ttFont, vhb):
        numeralsets.remove("lf")
    if tf_matrix(ttFont, vhb):
        numeralsets.remove("tf")

    # If only one numeral set remains, return it
    if len(numeralsets) == 1:
        return numeralsets[0]

    # otherwise use heuristics
    else:
        return numeral_style_heuristics(ttFont, vhb)


def slashed_zero(ttFont, vhb):
    return ttFont.has_feature("zero") and vhb.str("0") != vhb.str(
        "0", {"features": {"zero": True}}
    )


def default_fractions(ttFont, vhb):
    return ttFont.has_feature("frac") and vhb.str("1/4 1/2 3/4") != vhb.str(
        "1/4 1/2 3/4", {"features": {"frac": True}}
    )


def extended_fractions(ttFont, vhb):
    return ttFont.has_feature("frac") and vhb.str("1234/5678") != vhb.str(
        "1234/5678", {"features": {"frac": True}}
    )


def inferiors(ttFont, vhb):
    covered = 0
    for numeral in NUMERALS:
        if vhb.str(numeral) != vhb.str(numeral, {"features": {"sinf": True}}):
            covered += 1
    return covered / len(NUMERALS)


def superiors(ttFont, vhb):
    covered = 0
    for numeral in NUMERALS:
        if vhb.str(numeral) != vhb.str(numeral, {"features": {"sups": True}}):
            covered += 1
    return covered / len(NUMERALS)
