import os
from fontquant import quantify, CustomTTFont


def get_font_path(filename):
    return os.path.dirname(os.path.realpath(__file__)) + "/fonts/" + filename


def get_result(filename, includes=None, excludes=None, variable=None):
    font_path = get_font_path(filename)

    return quantify(
        font_path,
        includes=includes,
        excludes=excludes,
        locations=variable,
    )


def test_quantify():
    bigshouldersstencil = get_result("BigShouldersStencilText[wght].ttf")
    assert bigshouldersstencil == {
        "appearance": {
            "XOFI": {"value": 41},
            "XOLC": {"value": 41},
            "XOPQ": {"value": 40},
            "XTFI": {"value": 158},
            "XTLC": {"value": 152},
            "XTRA": {"value": 160},
            "YOFI": {"value": None},
            "YOLC": {"value": 34},
            "YOPQ": {"value": 38},
            "ascender": {"value": 800.0},
            "cap_height": {"value": 800.0},
            "descender": {"value": -201.0},
            "lowercase_a_style": {"value": None},
            "lowercase_g_style": {"value": None},
            "slant": {"value": -0.447},
            "stencil": {"value": True},
            "stroke_contrast_angle": {"value": 0.0},
            "stroke_contrast_ratio": {"value": 0.86},
            "weight": {"value": 0.188},
            "width": {"value": 0.319},
            "x_height": {"value": 600.0},
        },
        "casing": {
            "caps-to-smallcaps": {"failed": ["İ", "Ǎ", "Ǐ", "Ǒ", "Ǔ", "Ǖ", "Ǘ", "Ǚ", "Ǜ", "Ủ"], "value": 0.958},
            "case_sensitive_punctuation": {"value": 0.385},
            "lowercase_shapes": {"value": "lowercase"},
            "smallcaps": {
                "failed": ["ǎ", "ǐ", "ǒ", "ǔ", "ǖ", "ǘ", "ǚ", "ǜ", "ǝ", "ȷ", "π", "ủ", "ﬁ", "ﬂ"],
                "value": 0.943,
            },
            "unicase": {"value": False},
        },
        "numerals": {
            "arbitrary_fractions": {"value": True},
            "default_numerals": {"value": "proportional_lining"},
            "encoded_fractions": {"value": 1.0},
            "inferior_numerals": {"value": 1.0},
            "proportional_lining": {"value": True},
            "proportional_oldstyle": {"value": False},
            "slashed_zero": {"checked_additional_features": ["sups", "sinf", "frac"], "value": 0.0},
            "superior_numerals": {"value": 1.0},
            "tabular_lining": {"value": False},
            "tabular_oldstyle": {"value": False},
        },
    }


def test_casing():
    farro = get_result("Farro-Regular.ttf", includes=["casing"])
    assert farro["casing"]["unicase"]["value"] is False
    assert farro["casing"]["lowercase_shapes"]["value"] == "lowercase"
    youngserif = get_result("YoungSerif-Regular.ttf", includes=["casing"])
    assert youngserif["casing"]["unicase"]["value"] is False
    unica = get_result("UnicaOne-Regular.ttf", includes=["casing"])
    assert unica["casing"]["unicase"]["value"] is True
    delius = get_result("DeliusUnicase-Regular.ttf", includes=["casing"])
    assert delius["casing"]["unicase"]["value"] is True
    ysabeau = get_result("Ysabeau[wght].ttf", includes=["casing"])
    assert ysabeau["casing"]["lowercase_shapes"]["value"] == "lowercase"
    ysabeau_sc = get_result("YsabeauSC[wght].ttf", includes=["casing"])
    assert ysabeau_sc["casing"]["lowercase_shapes"]["value"] == "smallcaps"
    castorotitling = get_result("CastoroTitling-Regular.ttf", includes=["casing"])
    assert castorotitling["casing"]["lowercase_shapes"]["value"] == "uppercase"


def test_numerals():
    farro = get_result("Farro-Regular.ttf", includes=["numerals"])
    assert farro["numerals"]["proportional_oldstyle"]["value"] is False
    assert farro["numerals"]["tabular_oldstyle"]["value"] is False
    assert farro["numerals"]["proportional_lining"]["value"] is True
    assert farro["numerals"]["tabular_lining"]["value"] is True
    assert farro["numerals"]["default_numerals"]["value"] == "proportional_lining"

    foldit = get_result("Foldit-VariableFont_wght.ttf", includes=["numerals"])
    assert foldit["numerals"]["proportional_oldstyle"]["value"] is False
    assert foldit["numerals"]["tabular_oldstyle"]["value"] is False
    # Foldit has tabular_lining numerals by default and an additional .lf set
    # but they look identical to the tabular_lining set, so False is reported here:
    assert foldit["numerals"]["proportional_lining"]["value"] is False
    assert foldit["numerals"]["tabular_lining"]["value"] is True
    assert foldit["numerals"]["default_numerals"]["value"] == "tabular_lining"


def test_appearance():
    # TODO:
    # Test for Foldit, which errors out
    farro = get_result("Farro-Regular.ttf", includes=["appearance"])
    assert farro["appearance"]["stroke_contrast_ratio"]["value"] == 0.94
    assert farro["appearance"]["stroke_contrast_angle"]["value"] == 0.0
    assert farro["appearance"]["weight"]["value"] == 0.296
    assert farro["appearance"]["width"]["value"] == 0.561
    assert farro["appearance"]["slant"]["value"] == -0.099
    assert farro["appearance"]["lowercase_a_style"]["value"] == "double_story"
    assert farro["appearance"]["lowercase_g_style"]["value"] == "single_story"
    assert farro["appearance"]["stencil"]["value"] is False
    assert farro["appearance"]["x_height"]["value"] == 600.0
    assert farro["appearance"]["cap_height"]["value"] == 751.0
    assert farro["appearance"]["ascender"]["value"] == 800.0
    assert farro["appearance"]["descender"]["value"] == -216.0

    youngserif = get_result("YoungSerif-Regular.ttf", includes=["appearance"])
    assert youngserif["appearance"]["stroke_contrast_ratio"]["value"] == 0.55
    assert youngserif["appearance"]["stroke_contrast_angle"]["value"] == 24.06
    assert youngserif["appearance"]["stencil"]["value"] is False

    bodonimoda = get_result("BodoniModa_18pt-Italic.ttf", includes=["appearance"])
    assert bodonimoda["appearance"]["stroke_contrast_ratio"]["value"] == 0.15
    assert bodonimoda["appearance"]["stroke_contrast_angle"]["value"] == -17.14
    assert bodonimoda["appearance"]["lowercase_a_style"]["value"] == "single_story"
    assert bodonimoda["appearance"]["lowercase_g_style"]["value"] == "double_story"
    assert bodonimoda["appearance"]["stencil"]["value"] is False
    assert bodonimoda["appearance"]["slant"]["value"] == -11.821

    allertastencil = get_result("AllertaStencil-Regular.ttf", includes=["appearance/stencil"])
    assert allertastencil["appearance"]["stencil"]["value"] is True

    bigshouldersstencil = get_result("BigShouldersStencilText[wght].ttf", includes=["appearance/stencil"])
    assert bigshouldersstencil["appearance"]["stencil"]["value"] is True

    robotoflex = get_result("RobotoFlex-Var.ttf", includes=["appearance"])
    assert robotoflex["appearance"]["XOPQ"]["value"] == 94
    assert robotoflex["appearance"]["XOLC"]["value"] == 91
    assert robotoflex["appearance"]["XOFI"]["value"] == 94
    assert robotoflex["appearance"]["XTRA"]["value"] == 358
    assert robotoflex["appearance"]["XTLC"]["value"] == 234
    assert robotoflex["appearance"]["XTFI"]["value"] == 268
    assert robotoflex["appearance"]["YOPQ"]["value"] == 77
    assert robotoflex["appearance"]["YOLC"]["value"] == 69
    assert robotoflex["appearance"]["YOFI"]["value"] == 77


def test_variable():
    font = "Foldit-VariableFont_wght.ttf"
    assert (
        get_result(
            font,
            includes=["appearance/weight"],
            variable="fvar",
        )
        == get_result(
            font,
            includes=["appearance/weight"],
            variable="stat",
        )
        == get_result(
            font,
            includes=["appearance/weight"],
            variable="all",
        )
        == {
            "appearance": {
                "weight": {
                    "value": {
                        "wght=100.0": 0.137,
                        "wght=200.0": 0.178,
                        "wght=300.0": 0.238,
                        "wght=400.0": 0.305,
                        "wght=500.0": 0.358,
                        "wght=600.0": 0.378,
                        "wght=700.0": 0.419,
                        "wght=800.0": 0.474,
                        "wght=900.0": 0.515,
                    }
                }
            }
        }
    )


def test_helpers():
    ttFont = CustomTTFont(get_font_path("Farro-Regular.ttf"))
    assert ttFont.get_glyphs_for_primary_script() == [
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "g",
        "h",
        "i",
        "j",
        "k",
        "l",
        "m",
        "n",
        "o",
        "p",
        "q",
        "r",
        "s",
        "t",
        "u",
        "v",
        "w",
        "x",
        "y",
        "z",
        "ordfeminine",
        "ordmasculine",
        "Agrave",
        "Aacute",
        "Acircumflex",
        "Atilde",
        "Adieresis",
        "Aring",
        "AE",
        "Ccedilla",
        "Egrave",
        "Eacute",
        "Ecircumflex",
        "Edieresis",
        "Igrave",
        "Iacute",
        "Icircumflex",
        "Idieresis",
        "Eth",
        "Ntilde",
        "Ograve",
        "Oacute",
        "Ocircumflex",
        "Otilde",
        "Odieresis",
        "Oslash",
        "Ugrave",
        "Uacute",
        "Ucircumflex",
        "Udieresis",
        "Yacute",
        "Thorn",
        "germandbls",
        "agrave",
        "aacute",
        "acircumflex",
        "atilde",
        "adieresis",
        "aring",
        "ae",
        "ccedilla",
        "egrave",
        "eacute",
        "ecircumflex",
        "edieresis",
        "igrave",
        "iacute",
        "icircumflex",
        "idieresis",
        "eth",
        "ntilde",
        "ograve",
        "oacute",
        "ocircumflex",
        "otilde",
        "odieresis",
        "oslash",
        "ugrave",
        "uacute",
        "ucircumflex",
        "udieresis",
        "yacute",
        "thorn",
        "ydieresis",
        "Amacron",
        "amacron",
        "Abreve",
        "abreve",
        "Aogonek",
        "aogonek",
        "Cacute",
        "cacute",
        "Cdotaccent",
        "cdotaccent",
        "Ccaron",
        "ccaron",
        "Dcaron",
        "dcaron",
        "Dcroat",
        "dcroat",
        "Emacron",
        "emacron",
        "Edotaccent",
        "edotaccent",
        "Eogonek",
        "eogonek",
        "Ecaron",
        "ecaron",
        "Gbreve",
        "gbreve",
        "Gdotaccent",
        "gdotaccent",
        "uni0122",
        "uni0123",
        "Hbar",
        "hbar",
        "Imacron",
        "imacron",
        "Iogonek",
        "iogonek",
        "Idotaccent",
        "dotlessi",
        "IJ",
        "ij",
        "uni0136",
        "uni0137",
        "Lacute",
        "lacute",
        "uni013B",
        "uni013C",
        "Lcaron",
        "lcaron",
        "Ldot",
        "ldot",
        "Lslash",
        "lslash",
        "Nacute",
        "nacute",
        "uni0145",
        "uni0146",
        "Ncaron",
        "ncaron",
        "napostrophe",
        "Eng",
        "eng",
        "Omacron",
        "omacron",
        "Ohungarumlaut",
        "ohungarumlaut",
        "OE",
        "oe",
        "Racute",
        "racute",
        "uni0156",
        "uni0157",
        "Rcaron",
        "rcaron",
        "Sacute",
        "sacute",
        "Scedilla",
        "scedilla",
        "Scaron",
        "scaron",
        "uni0162",
        "uni0163",
        "Tcaron",
        "tcaron",
        "Tbar",
        "tbar",
        "Umacron",
        "umacron",
        "Uring",
        "uring",
        "Uhungarumlaut",
        "uhungarumlaut",
        "Uogonek",
        "uogonek",
        "Wcircumflex",
        "wcircumflex",
        "Ycircumflex",
        "ycircumflex",
        "Ydieresis",
        "Zacute",
        "zacute",
        "Zdotaccent",
        "zdotaccent",
        "Zcaron",
        "zcaron",
        "uni0218",
        "uni0219",
        "uni021A",
        "uni021B",
        "uni0237",
        "caron",
        "dotaccent",
        "gravecomb",
        "acutecomb",
        "uni0302",
        "tildecomb",
        "uni0304",
        "uni0306",
        "uni0307",
        "uni0308",
        "uni030A",
        "uni030B",
        "uni030C",
        "Wgrave",
        "wgrave",
        "Wacute",
        "wacute",
        "Wdieresis",
        "wdieresis",
        "uni1E9E",
        "Ygrave",
        "ygrave",
        "fi",
    ]
