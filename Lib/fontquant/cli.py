import sys
import json
from fontquant import quantify
import argparse


def cli():

    arg_parser = argparse.ArgumentParser(description="Measure font statistics.")
    arg_parser.add_argument(
        "-i",
        action="append",
        help="Include metrics by their (partial) path. You can use several -i flags. Only these will be used.",
    )
    arg_parser.add_argument(
        "-x",
        action="append",
        help="Exclude metrics by their (partial) path. You can use several -x flags. All except these will be used.",
    )
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
        "--primary_script",
        action="store",
        help=("Primary script as per https://github.com/google/fonts/tree/main/lang/Lib/gflanguages/data/scripts"),
    )
    arg_parser.add_argument(
        "--profile",
        action="store_true",
        help=("Developer option: Profile the code. Outputs a list of functions sorted by total time. "),
    )
    arg_parser.add_argument(
        "--debug",
        action="store_true",
        help=("Developer option: Add debugging information to the output."),
    )
    arg_parser.add_argument(
        "--show",
        action="store_true",
        help=("Developer option: If there's anything to show visually, show it. Only works with activated debugging."),
    )
    arg_parser.add_argument("font", help="Font file (.ttf or .otf)")
    options = arg_parser.parse_args(sys.argv[1:])

    if options.profile:
        import cProfile
        import pstats

        profiler = cProfile.Profile()
        profiler.enable()

    if options.debug and options.show:
        import matplotlib
        print("Backend is", matplotlib.get_backend())
    else:
        print("No plot output")

    formatted = json.dumps(
        quantify(
            options.font,
            includes=options.i,
            excludes=options.x,
            locations=options.l,
            debug=options.debug,
            show=options.show,
            primary_script=options.primary_script,
        ),
        indent=2,
    )
    print(formatted)

    if options.profile:
        profiler.disable()
        stats = pstats.Stats(profiler).sort_stats("tottime")
        stats.print_stats()
