import os
from fontquant import CustomHarfbuzz, CustomTTFont, Base


def get_font_path(filename):
    return os.path.dirname(os.path.realpath(__file__)) + "/fonts/" + filename


def get_result(filename, includes=None, excludes=None, variable=None):
    font_path = get_font_path(filename)

    ttFont = CustomTTFont(font_path)
    vhb = CustomHarfbuzz(font_path)

    base = Base(ttFont, vhb, variable)
    return base.value(includes, excludes)


def test_casing():
    farro = get_result("Farro-Regular.ttf", includes=["casing"])
    assert farro["casing"]["unicase"]["value"] is False
    youngserif = get_result("YoungSerif-Regular.ttf", includes=["casing"])
    assert youngserif["casing"]["unicase"]["value"] is False
    unica = get_result("UnicaOne-Regular.ttf", includes=["casing"])
    assert unica["casing"]["unicase"]["value"] is True
    delius = get_result("DeliusUnicase-Regular.ttf", includes=["casing"])
    assert delius["casing"]["unicase"]["value"] is True


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
    assert farro["appearance"]["stroke_contrast_angle"]["value"] == 1.37
    assert farro["appearance"]["weight"]["value"] == 0.295
    assert farro["appearance"]["width"]["value"] == 0.589
    assert farro["appearance"]["slant"]["value"] == 0.142
    assert farro["appearance"]["lowercase_a_style"]["value"] == "double_story"
    assert farro["appearance"]["lowercase_g_style"]["value"] == "single_story"
    assert farro["appearance"]["stencil"]["value"] is False

    youngserif = get_result("YoungSerif-Regular.ttf", includes=["appearance"])
    assert youngserif["appearance"]["stroke_contrast_ratio"]["value"] == 0.55
    assert youngserif["appearance"]["stroke_contrast_angle"]["value"] == 25.42
    assert youngserif["appearance"]["stencil"]["value"] is False

    bodonimoda = get_result("BodoniModa_18pt-Italic.ttf", includes=["appearance"])
    assert bodonimoda["appearance"]["stroke_contrast_ratio"]["value"] == 0.15
    assert bodonimoda["appearance"]["stroke_contrast_angle"]["value"] == -17.88
    assert bodonimoda["appearance"]["lowercase_a_style"]["value"] == "single_story"
    assert bodonimoda["appearance"]["lowercase_g_style"]["value"] == "double_story"
    assert bodonimoda["appearance"]["stencil"]["value"] is False
    assert bodonimoda["appearance"]["slant"]["value"] == -12.23

    allertastencil = get_result("AllertaStencil-Regular.ttf", includes=["appearance/stencil"])
    assert allertastencil["appearance"]["stencil"]["value"] is True

    bigshouldersstencil = get_result("BigShouldersStencilText[wght].ttf", includes=["appearance/stencil"])
    assert bigshouldersstencil["appearance"]["stencil"]["value"] is True


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
                        "wght=100.0": 0.139,
                        "wght=200.0": 0.179,
                        "wght=300.0": 0.239,
                        "wght=400.0": 0.306,
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
