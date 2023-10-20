import os
from fontquant import CustomHarfbuzz, CustomTTFont, Base


def get_font_path(filename):
    return os.path.dirname(os.path.realpath(__file__)) + "/fonts/" + filename


def get_result(filename):
    font_path = get_font_path(filename)

    ttFont = CustomTTFont(font_path)
    vhb = CustomHarfbuzz(font_path)

    base = Base(ttFont, vhb)
    return base.value()


farro = get_result("Farro-Regular.ttf")
foldit = get_result("Foldit-VariableFont_wght.ttf")
youngserif = get_result("YoungSerif-Regular.ttf")
robotoflex = get_result("RobotoFlex-Var.ttf")
bodonimoda = get_result("BodoniModa_18pt-Italic.ttf")


def test_numerals():
    assert farro["numerals"]["proportional_oldstyle"]["value"] is False
    assert farro["numerals"]["tabular_oldstyle"]["value"] is False
    assert farro["numerals"]["proportional_lining"]["value"] is True
    assert farro["numerals"]["tabular_lining"]["value"] is True
    assert farro["numerals"]["default_numerals"]["value"] == "proportional_lining"

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
    assert farro["appearance"]["stroke_contrast_ratio"]["value"] == 0.94
    assert farro["appearance"]["stroke_contrast_angle"]["value"] == 0
    assert youngserif["appearance"]["stroke_contrast_ratio"]["value"] == 0.55
    assert youngserif["appearance"]["stroke_contrast_angle"]["value"] == 25
    assert robotoflex["appearance"]["stroke_contrast_ratio"]["value"] == 0.8
    assert robotoflex["appearance"]["stroke_contrast_angle"]["value"] == 0
    assert bodonimoda["appearance"]["stroke_contrast_ratio"]["value"] == 0.15
    assert bodonimoda["appearance"]["stroke_contrast_angle"]["value"] == -18

    assert farro["appearance"]["weight"]["value"] == 0.295
    assert farro["appearance"]["width"]["value"] == 0.589


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
