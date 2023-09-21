from vharfbuzz import Vharfbuzz
import uharfbuzz as hb
from fontTools import ttLib


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
        self.drawfuncs.set_move_to_func(move_to, buffer_list)
        self.drawfuncs.set_line_to_func(line_to, buffer_list)
        self.drawfuncs.set_cubic_to_func(cubic_to, buffer_list)
        self.drawfuncs.set_quadratic_to_func(quadratic_to, buffer_list)
        self.drawfuncs.set_close_path_func(close_path, buffer_list)

    def glyph_to_points(self, gid):
        if not hasattr(hb, "DrawFuncs"):
            raise ValueError("glyph_to_points_path requires uharfbuzz with draw function support")

        buffer_list = []
        self.setup_points_draw_funcs(buffer_list)
        self.drawfuncs.get_glyph_shape(self.hbfont, gid)
        return buffer_list

    def buf_to_bbox(self, buf):
        x_cursor = 0
        if "hhea" in self.ttfont:
            ascender = self.ttfont["hhea"].ascender + 500
            descender = self.ttfont["hhea"].descender - 500
            fullheight = ascender - descender
        elif "OS/2":
            ascender = self.ttfont["OS/2"].sTypoAscender + 500
            descender = self.ttfont["OS/2"].sTypoDescender - 500
            fullheight = ascender - descender
        else:
            fullheight = 1500
            descender = 500
        y_cursor = 0

        x_min = None
        x_max = None
        y_min = None
        y_max = None

        for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
            dx, dy = pos.position[0], pos.position[1]
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
        if "hhea" in self.ttfont:
            ascender = self.ttfont["hhea"].ascender + 500
            descender = self.ttfont["hhea"].descender - 500
            fullheight = ascender - descender
        elif "OS/2":
            ascender = self.ttfont["OS/2"].sTypoAscender + 500
            descender = self.ttfont["OS/2"].sTypoDescender - 500
            fullheight = ascender - descender
        else:
            fullheight = 1500
            descender = 500
        y_cursor = 0

        x_min = None
        x_max = None
        y_min = None
        y_max = None

        for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
            dx, dy = pos.position[0], pos.position[1]
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

        return x_cursor

    def str(self, string, options={}):
        """Return the shaped string buffer as a string."""
        buf = self.shape(string, options)
        return self.serialize_buf(buf)

    def bbox(self, string, options={}):
        """Return the shaped string buffer's  bbox."""
        buf = self.shape(string, options)
        return self.buf_to_bbox(buf)


class CustomTTFont(ttLib.TTFont):
    def has_feature(self, tag):
        return tag in [FeatureRecord.FeatureTag for FeatureRecord in self["GSUB"].table.FeatureList.FeatureRecord]


class Check(object):
    name = None
    keyword = None
    children = []
    interpretation_hint = None

    def __init__(self, ttFont, vhb, parent=None) -> None:
        self.ttFont = ttFont
        self.vhb = vhb
        self.parent = parent

    def JSON(self):
        dictionary = {}
        for child in self.children:
            instance = child(self.ttFont, self.vhb, parent=self)
            dictionary[instance.keyword] = instance.JSON()
        return dictionary

    def path(self):
        if self.parent:
            return self.parent.path() + [self.keyword]
        else:
            return [self.keyword] if self.keyword else []

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
        LB = "\n"
        if self.__doc__:
            markdown = f"""\
### {self.name} (`{"/".join(self.path())}`)

{" ".join([line.strip() for line in self.__doc__.splitlines()])}
"""
            if self.interpretation_hint:
                markdown += "\n_Interpretation Hint:_ " + (
                    " ".join([line.strip() for line in self.interpretation_hint.splitlines()]) + "\n\n"
                )

            return markdown

        else:
            markdown = ""

            if self.name:
                markdown += f"## {self.name}\n\n"

            for child in self.children:
                instance = child(self.ttFont, self.vhb, parent=self)
                markdown += instance.documentation()
            return markdown


from .casing import Casing
from .numerals import Numerals


class Base(Check):
    children = [Casing, Numerals]
