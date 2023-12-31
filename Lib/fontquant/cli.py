import sys
import json
from fontquant import CustomHarfbuzz, CustomTTFont, Base
import argparse


def cli():
    arg_parser = argparse.ArgumentParser(description="Measure font statistics.")
    arg_parser.add_argument("-i", action="append", help="Include metrics by their (partial) path")
    arg_parser.add_argument("-x", action="append", help="Exclude metrics by their (partial) path")
    arg_parser.add_argument("font", help="Font file (.ttf or .otf)")
    options = arg_parser.parse_args(sys.argv[1:])

    ttFont = CustomTTFont(sys.argv[-1])
    vhb = CustomHarfbuzz(sys.argv[-1])

    base = Base(ttFont, vhb)
    formatted = json.dumps(base.value(options.i, options.x), indent=2)
    print(formatted)
