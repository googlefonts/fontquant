import sys
import os
from fontquant import Base

sys.path.append(os.path.dirname(os.path.realpath(__file__)) + "/../Lib")
doc_path = os.path.dirname(os.path.realpath(__file__)) + "/../Lib/fontquant/README.md"
header_path = os.path.dirname(os.path.realpath(__file__)) + "/../Lib/fontquant/README.header.md"


base = Base(None, None)

with open(header_path, "r") as header:
    with open(doc_path, "w") as out:
        out.write(header.read() + "\n\n" + "  \n".join(base.link_list()) + "\n\n" + base.documentation())
