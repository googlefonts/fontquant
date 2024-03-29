from vharfbuzz import Vharfbuzz
import uharfbuzz as hb
from fontTools import ttLib
from fontquant.helpers.fontcontent import get_primary_script, get_glyphs_for_script


class CustomHarfbuzz(Vharfbuzz):
    def setup_points_draw_funcs(self, buffer_list):
        def move_to(x, y, buffer_list):
            buffer_list.append((x, y))

        def line_to(x, y, buffer_list):
            buffer_list.append((x, y))

        def cubic_to(c1x, c1y, c2x, c2y, x, y, buffer_list):
            buffer_list.append((c1x, c1y))
            buffer_list.append((c2x, c2y))
            buffer_list.append((x, y))

        def quadratic_to(c1x, c1y, x, y, buffer_list):
            buffer_list.append((c1x, c1y))
            buffer_list.append((x, y))

        def close_path(buffer_list):
            pass

        self.drawfuncs = hb.DrawFuncs()
        self.drawfuncs.set_move_to_func(move_to)
        self.drawfuncs.set_line_to_func(line_to)
        self.drawfuncs.set_cubic_to_func(cubic_to)
        self.drawfuncs.set_quadratic_to_func(quadratic_to)
        self.drawfuncs.set_close_path_func(close_path)

    def glyph_to_points(self, gid):
        if not hasattr(hb, "DrawFuncs"):
            raise ValueError("glyph_to_points_path requires uharfbuzz with draw function support")

        buffer_list = []
        self.setup_points_draw_funcs(buffer_list)
        self.hbfont.draw_glyph(gid, self.drawfuncs, buffer_list)

        return buffer_list

    def buf_to_bbox(self, buf):
        x_cursor = 0
        # if "hhea" in self.ttfont:
        #     # ascender = self.ttfont["hhea"].ascender + 500
        #     # descender = self.ttfont["hhea"].descender - 500
        #     # fullheight = ascender - descender
        # elif "OS/2":
        #     ascender = self.ttfont["OS/2"].sTypoAscender + 500
        #     descender = self.ttfont["OS/2"].sTypoDescender - 500
        #     # fullheight = ascender - descender
        # else:
        #     # fullheight = 1500
        #     descender = 500
        y_cursor = 0

        x_min = None
        x_max = None
        y_min = None
        y_max = None

        for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
            # dx, dy = pos.position[0], pos.position[1]
            glyph_path = [(x + x_cursor, y + y_cursor) for x, y in self.glyph_to_points(info.codepoint)]
            for x, y in glyph_path:
                if x_min is None or x < x_min:
                    x_min = x
                if x_max is None or x > x_max:
                    x_max = x
                if y_min is None or y < y_min:
                    y_min = y
                if y_max is None or y > y_max:
                    y_max = y
            x_cursor += pos.position[2]
            y_cursor += pos.position[3]

        return x_min, x_max, y_min, y_max

    def buf_to_width(self, buf):
        x_cursor = 0

        for _info, pos in zip(buf.glyph_infos, buf.glyph_positions):
            # dx, dy = pos.position[0], pos.position[1]
            x_cursor += pos.position[2]

        return x_cursor

    def str(self, string, options=None):
        """Return the shaped string buffer as a string."""
        buf = self.shape(string, options)
        return self.serialize_buf(buf)

    def bbox(self, string, options=None):
        """Return the shaped string buffer's  bbox."""
        buf = self.shape(string, options)
        return self.buf_to_bbox(buf)


class CustomTTFont(ttLib.TTFont):
    def has_feature(self, tag):
        return tag in [FeatureRecord.FeatureTag for FeatureRecord in self["GSUB"].table.FeatureList.FeatureRecord]

    def glyphname_for_char(self, char):
        """Convert a character to a glyph name."""
        cmap = self.getBestCmap()
        if ord(char) in cmap:
            return cmap[ord(char)]
        else:
            return None

    def get_primary_script(self):
        """Retrieve font's primary script."""
        return get_primary_script(self)

    def get_glyphs_for_primary_script(self):
        """Retrieve list of glyph names of font's primary_script."""
        return get_glyphs_for_script(self, self.get_primary_script())


class BaseDataType(object):
    def example_value(self, default_example_value):
        return self.shape_value(default_example_value) or None

    def return_value_description(self):
        return None

    def shape_value(self, value):
        return value


class Percentage(BaseDataType):
    def example_value(self, default_example_value):
        return self.shape_value(default_example_value) or 0.5

    def return_value_description(self):
        return "Percentage expressed as float 0—1 (e.g. `0.5`)"

    def shape_value(self, value):
        if value is not None:
            return round(value * 1000) / 1000
        else:
            return 0.0


class Boolean(BaseDataType):
    def example_value(self, default_example_value):
        return self.shape_value(default_example_value) or True

    def return_value_description(self):
        return "Boolean (`True`or `False`)"


class String(BaseDataType):
    def example_value(self, default_example_value):
        return self.shape_value(default_example_value) or "abc..."

    def return_value_description(self):
        return "String"


class Integer(BaseDataType):
    def example_value(self, default_example_value):
        return self.shape_value(default_example_value) or 5

    def return_value_description(self):
        return "Integer number (e.g. `5`)"


class Metric(object):
    name = None
    keyword = None
    children = []
    interpretation_hint = None
    data_type = None
    example_value = None
    fully_automatic = True

    def __init__(self, ttFont, vhb, parent=None) -> None:
        self.ttFont = ttFont
        self.vhb = vhb
        self.parent = parent

    def shape_value(self, value):
        return self.data_type().shape_value(value)

    def find_check(self, path):
        for child in self.children:
            instance = child(self.ttFont, self.vhb, parent=self)
            if instance.path() == path.split("/"):
                return instance
            else:
                found = instance.find_check(path)
                if found:
                    return found
        return None

    def is_included(self, includes):
        path = "/".join(self.path())
        # We are at root
        if path == "":
            return True
        if includes:
            for include in includes:
                include_root = include.split("/")[0]
                path_root = path.split("/")[0]
                # We are category root
                if include_root == path_root == path:
                    return True
                # We are normal metric
                elif path.startswith(include):
                    return True
            return False
        else:
            return True

    def is_excluded(self, excludes):
        path = "/".join(self.path())
        # We are at root
        if path == "":
            return False
        if excludes:
            for exclude in excludes:
                exclude_root = exclude.split("/")[0]
                path_root = path.split("/")[0]
                # We are category root
                if exclude_root == path_root == path:
                    return True
                # We are normal metric
                elif path.startswith(exclude):
                    return True
            return False
        else:
            return False

    def value(self, includes=None, excludes=None):
        dictionary = {}
        for child in self.children:
            instance = child(self.ttFont, self.vhb, parent=self)
            if instance.is_included(includes) and not instance.is_excluded(excludes):
                dictionary[instance.keyword] = instance.value(includes, excludes)
            elif not includes and not excludes:
                dictionary[instance.keyword] = instance.value(includes, excludes)

        return dictionary

    def path(self):
        if self.parent:
            return self.parent.path() + [self.keyword]
        else:
            return [self.keyword] if self.keyword else []

    def base(self):
        if self.parent:
            return self.parent.base()
        else:
            return self

    def link_list(self):
        if self.__doc__:
            link = "/".join(self.path()).replace("/", "").replace(" ", "-")
            return [f'  * [{self.name}](#{self.name.lower().replace(" ", "-")}-{link})']
        else:
            check_list = []
            if self.name:
                check_list.append("* " + self.name + ":")
            for child in self.children:
                instance = child(self.ttFont, self.vhb, parent=self)
                new_list = instance.link_list()
                if new_list:
                    check_list += new_list
            return check_list

    def index(self):
        if self.__doc__:
            return "/".join(self.path()), self.name
        else:
            check_list = []
            for child in self.children:
                instance = child(self.ttFont, self.vhb, parent=self)
                new_list = instance.index()
                if new_list:
                    check_list += new_list
            return check_list

    def documentation(self):
        join_sequence = '"]["'

        if self.__doc__:
            markdown = f"""\
### {self.name} (`{"/".join(self.path())}`)

{" ".join([line.strip() for line in self.__doc__.splitlines()])}
"""
            if self.interpretation_hint:
                markdown += "\n_Interpretation Hint:_ " + (
                    " ".join([line.strip() for line in self.interpretation_hint.splitlines()]) + "\n\n"
                )

            if self.data_type:
                markdown += f"""\n_Return Value:_ {self.data_type().return_value_description()}

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["{join_sequence.join(self.path())}"]["value"]
print(value)
>>> {self.data_type().example_value(self.example_value)}
```

"""

            return markdown

        else:
            markdown = ""

            if self.name:
                markdown += f"## {self.name}\n\n"

            for child in self.children:
                instance = child(self.ttFont, self.vhb, parent=self)
                markdown += instance.documentation()
            return markdown


from .casing import Casing  # noqa E402 (Circular import)
from .numerals import Numerals  # noqa E402 (Circular import)
from .appearance import Appearance  # noqa E402 (Circular import)


class Base(Metric):
    children = [Casing, Numerals, Appearance]


def quantify(font_path, includes=None, excludes=None):
    ttFont = CustomTTFont(font_path)
    vhb = CustomHarfbuzz(font_path)

    base = Base(ttFont, vhb)
    return base.value(includes, excludes)
