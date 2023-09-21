import json
from fontquant import CustomHarfbuzz, CustomTTFont, Base


def cli():
    import sys

    ttFont = CustomTTFont(sys.argv[-1])
    vhb = CustomHarfbuzz(sys.argv[-1])

    base = Base(ttFont, vhb)
    formatted = json.dumps(base.JSON(), indent=2)
    print(formatted)
