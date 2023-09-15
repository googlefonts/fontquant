from vharfbuzz import Vharfbuzz
import uharfbuzz as hb
from fontTools import ttLib

from casing import case, c2sc, smcp
from numerals import (
    osf,
    tosf,
    lf,
    tf,
    default_numerals,
    slashed_zero,
    default_fractions,
    extended_fractions,
    inferiors,
    superiors,
)
from alternates import ss, salt, calt


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
            raise ValueError(
                "glyph_to_points_path requires uharfbuzz with draw function support"
            )

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
            glyph_path = [
                (x + x_cursor, y + y_cursor)
                for x, y in self.glyph_to_points(info.codepoint)
            ]
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
            glyph_path = [
                (x + x_cursor, y + y_cursor)
                for x, y in self.glyph_to_points(info.codepoint)
            ]
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
        return tag in [
            FeatureRecord.FeatureTag
            for FeatureRecord in self["GSUB"].table.FeatureList.FeatureRecord
        ]


if __name__ == "__main__":
    import sys

    ttFont = CustomTTFont(sys.argv[1])
    vhb = CustomHarfbuzz(sys.argv[1])

    print()
    print("### ALTERNATES")
    print("stylistic sets:", ss(ttFont, vhb))
    print("stylistic alternates:", salt(ttFont, vhb))
    print("contextual alternates:", calt(ttFont, vhb))

    print()
    print("### NUMERALS")
    print("osf:", osf(ttFont, vhb))
    print("tosf:", tosf(ttFont, vhb))
    print("lf:", lf(ttFont, vhb))
    print("tf:", tf(ttFont, vhb))
    print("default numerals:", default_numerals(ttFont, vhb))
    print("default fractions (1/4):", default_fractions(ttFont, vhb))
    print("extended fractions (1234/5678):", extended_fractions(ttFont, vhb))
    print("slashed zero:", slashed_zero(ttFont, vhb))
    print("inferiors:", inferiors(ttFont, vhb))
    print("superiors:", superiors(ttFont, vhb))

    print()
    print("### CASING")
    print(f"small caps: {smcp(ttFont, vhb):.2f} coverage of unicodedata's Ll glyphs")
    print(
        f"caps to small caps: {c2sc(ttFont, vhb):.2f} coverage of unicodedata's Lu glyphs"
    )
    print(
        f"case-sensitive punctuation: {case(ttFont, vhb):.2f} coverage of unicodedata's P* glyphs"
    )
