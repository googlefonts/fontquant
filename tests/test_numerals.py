import sys
import os
from fontquant import CustomHarfbuzz, CustomTTFont, Base


def get_font_path(filename):
    return os.path.dirname(os.path.realpath(__file__)) + "/fonts/" + filename


def get_result(filename):
    font_path = get_font_path(filename)

    ttFont = CustomTTFont(font_path)
    vhb = CustomHarfbuzz(font_path)

    base = Base(ttFont, vhb)
    return base.JSON()


def test_numerals():
    results = get_result("Farro-Regular.ttf")
    assert results["numerals"]["proportional_oldstyle"] == False
    assert results["numerals"]["tabular_oldstyle"] == False
    assert results["numerals"]["proportional_lining"] == True
    assert results["numerals"]["tabular_lining"] == True
    assert results["numerals"]["default_numerals"] == "proportional_lining"

    results = get_result("Foldit-VariableFont_wght.ttf")
    assert results["numerals"]["proportional_oldstyle"] == False
    assert results["numerals"]["tabular_oldstyle"] == False
    # Foldit has tabular_lining numerals by default and an additional .lf set
    # but they look identical to the tabular_lining set, so False is reported here.
    assert results["numerals"]["proportional_lining"] == False
    assert results["numerals"]["tabular_lining"] == True
    assert results["numerals"]["default_numerals"] == "tabular_lining"
