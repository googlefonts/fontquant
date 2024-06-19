from fontquant.helpers.var import instances_str_to_list, instances_list_to_str, sort_instances
from fontquant.helpers.settings import get_script_setting


def test_helpers():

    # Tests not only string conversions, but also axis sorting.
    # Note how input order is diffent from output order
    # and output numbers are float.

    assert instances_str_to_list("wght=400,wdth=100;wght=500,wdth=100") == [
        {"wdth": 100.0, "wght": 400.0},
        {"wdth": 100.0, "wght": 500.0},
    ]
    assert (
        instances_list_to_str([{"wght": 400, "wdth": 100}, {"wght": 500, "wdth": 100}])
        == "wdth=100.0,wght=400.0;wdth=100.0,wght=500.0"
    )
    assert sort_instances([{"wght": 400, "wdth": 100}, {"wght": 500, "wdth": 100}]) == [
        {"wdth": 100.0, "wght": 400.0},
        {"wdth": 100.0, "wght": 500.0},
    ]
    assert sort_instances("wght=400,wdth=100;wght=500,wdth=100") == "wdth=100.0,wght=400.0;wdth=100.0,wght=500.0"


def test_settings():
    assert get_script_setting("Latn", "stroke_contrast_glyphs") == "o,O"
