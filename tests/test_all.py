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
