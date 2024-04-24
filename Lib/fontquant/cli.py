import sys
import json
from fontquant import CustomHarfbuzz, CustomTTFont, Base
import argparse


def cli():

    arg_parser = argparse.ArgumentParser(description="Measure font statistics.")
    arg_parser.add_argument("-i", action="append", help="Include metrics by their (partial) path")
    arg_parser.add_argument("-x", action="append", help="Exclude metrics by their (partial) path")
    arg_parser.add_argument(
        "-l",
        action="store",
        help=(
            "Variable instances locations to measure. Either 'stat' or 'fvar' or 'all' or "
            "'axis1=value1,axis2=value2;axis1=value1,axis2=value2;...'. With no input, only "
            "the origin location is measured."
        ),
    )
    arg_parser.add_argument(
        "-p",
        action="store_true",
        help=("Developer option: Profile the code. Outputs a list of functions sorted by total time. "),
    )
    arg_parser.add_argument("font", help="Font file (.ttf or .otf)")
    options = arg_parser.parse_args(sys.argv[1:])

    if options.p:
        import cProfile
        import pstats

        profiler = cProfile.Profile()
        profiler.enable()

    ttFont = CustomTTFont(sys.argv[-1])
    vhb = CustomHarfbuzz(sys.argv[-1])

    base = Base(ttFont, vhb, options.l)
    formatted = json.dumps(base.value(options.i, options.x), indent=2)
    print(formatted)

    if options.p:
        profiler.disable()
        stats = pstats.Stats(profiler).sort_stats("tottime")
        stats.print_stats()
