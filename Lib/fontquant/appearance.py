from fontquant import Metric, Percentage, Integer, String, Boolean
from fontquant.helpers.stroke_contrast import stroke_contrast
from beziers.path import BezierPath
from fontquant.helpers.pens import CustomStatisticsPen
from fontquant.helpers.beziers import removeOverlaps


class Stencil(Metric):
    """\
    Reports whether or not a font is a stencil font.
    """

    name = "Stencil"
    keyword = "stencil"
    data_type = Boolean

    def value(self, includes=None, excludes=None):
        glyph = self.ttFont.glyphname_for_char("a")
        if glyph:
            paths = BezierPath.fromFonttoolsGlyph(self.ttFont, glyph)
            paths = removeOverlaps(paths)

            if paths:
                path0 = paths[0]
                for path in paths[1:]:
                    if path0.intersection(path):
                        return {"value": False}
                    if path0.intersection(path.reverse()):
                        return {"value": False}

                return {"value": True}

        return {"value": False}


class LowercaseAStyle(Metric):
    """\
    Attempts to determine the style of the lowercase "a" to be single or double story.

    Only the most sturdy routines are used here. If the results are not conclusive,
    the metric will return None and you need to determine it manually.

    The error margin for recognizing the single story "a" is 0-2%, and for the double story "a" 4-7%.
    """

    name = "Lowercase a style"
    keyword = "lowercase_a_style"
    data_type = String
    example_value = "single_story"

    def value(self, includes=None, excludes=None):
        # TO DO:
        # Run this metric only if not unicase

        stencil_metric = Stencil(self.ttFont, self.vhb)
        stencil = stencil_metric.value()["value"]

        glyph = self.ttFont.glyphname_for_char("a")
        if glyph and not stencil:
            paths = BezierPath.fromFonttoolsGlyph(self.ttFont, glyph)
            paths = removeOverlaps(paths)

            # Two segments is valid for either single or double story
            if paths and len(paths) == 2:
                # Weight
                pen = CustomStatisticsPen()
                stats = pen.measure(self.ttFont, glyphs=[glyph])
                weight = stats["weight"]
                lower = (0.1, 1.7)
                higher = (0.3, 3.0)
                if weight < lower[0]:
                    threshold = lower[1]
                elif weight > higher[0]:
                    threshold = higher[1]
                else:
                    threshold = (weight - lower[0]) / (higher[0] - lower[0]) * (higher[1] - lower[1]) + lower[1]

                paths = sorted(paths, key=lambda path: path.length)
                path0_length = paths[0].length
                path1_length = paths[1].length
                ratio = path1_length / path0_length

                # Italic
                # Account for long tail of cursive single-story a
                slant_glyph = self.ttFont.glyphname_for_char("H")
                slant_stats = pen.measure(self.ttFont, glyphs=[slant_glyph])
                slant = slant_stats["slant"]
                if slant > 0.1:
                    threshold *= 1.2

                if ratio > threshold:
                    return {
                        "value": "double_story",
                        "debug": (weight, threshold, ratio),
                    }
                else:
                    return {
                        "value": "single_story",
                        "debug": (threshold, ratio),
                    }

        return {"value": None}


class LowercaseGStyle(Metric):
    """\
    Attempts to determine the style of the lowercase "g" to be single or double story.

    Only the most sturdy routines are used here. If the results are not conclusive,
    the metric will return None and you need to determine it manually.

    This metric is based on contour counting, which is not very reliable.
    A "g" which generally looks like a double story "g" but has an open lower bowl
    will be counted as single story, and a "g" in cursive writing that looks like
    a single story "g" but has a closed loop as part of an upstroke will be counted as double story.
    """

    name = "Lowercase g style"
    keyword = "lowercase_g_style"
    data_type = String
    example_value = "single_story"

    def value(self, includes=None, excludes=None):
        # TO DO:
        # Run this metric only if not unicase

        stencil_metric = Stencil(self.ttFont, self.vhb)
        stencil = stencil_metric.value()["value"]

        glyph = self.ttFont.glyphname_for_char("g")
        if glyph and not stencil:
            paths = BezierPath.fromFonttoolsGlyph(self.ttFont, glyph)
            paths = removeOverlaps(paths)

            # Two or three segments are valid
            if paths and len(paths) == 2:
                return {"value": "single_story"}

            elif paths and len(paths) == 3:
                return {"value": "double_story"}

        return {"value": None}


class StrokeContrastBase(object):
    measure_characters = {"fallback": "o", "Latn": "o", "Arab": "ه"}

    def get_character_to_measure(self):
        primary_script = self.ttFont.get_primary_script()
        if primary_script in self.measure_characters:
            return self.ttFont.glyphname_for_char(self.measure_characters[primary_script])
        else:
            return self.ttFont.glyphname_for_char(self.measure_characters["fallback"])

    def measure(self):
        character = self.get_character_to_measure()
        width = self.ttFont.getGlyphSet()[character].width
        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, character)
        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        self.parent.stroke_values = stroke_contrast(paths, width, ascender, descender)


class StrokeContrastRatio(Metric, StrokeContrastBase):
    """\
    Calculates the ratio of the stroke contrast,
    calculated in thinnest/thickest stroke.

    One representative character is measured for the font's primary script,
    such as the "o" for Latin.
    """

    name = "Stroke Contrast Ratio"
    keyword = "stroke_contrast_ratio"
    data_type = Percentage

    def value(self, includes=None, excludes=None):
        if not hasattr(self.parent, "stroke_values"):
            try:
                self.measure()
            except Exception as e:
                return {"value": str(e)}

        return {"value": self.shape_value(self.parent.stroke_values[0])}


class StrokeContrastAngle(Metric, StrokeContrastBase):
    """\
    Calculates the angle of the stroke contrast. An angle of 0° means
    vertical contrast, with positive angles being counter-clockwise.

    One representative character is measured for the font's primary script,
    such as the "o" for Latin.
    """

    name = "Stroke Contrast Angle"
    keyword = "stroke_contrast_angle"
    data_type = Integer

    def value(self, includes=None, excludes=None):
        if not hasattr(self.parent, "stroke_values"):
            try:
                self.measure()
            except Exception as e:
                return {"value": str(e)}

        return {"value": self.shape_value(self.parent.stroke_values[1])}


class Weight(Metric):
    """\
    Measures the weight of all letters in the primary script of the font.
    This metric measures the amount of ink per glyph as a percentage of an em square
    and returns the average of all glyphs measured.

    Based on fontTools.pens.statisticsPen.StatisticsPen
    """

    name = "Weight"
    keyword = "weight"
    data_type = Percentage

    def value(self, includes=None, excludes=None):
        pen = CustomStatisticsPen()
        stats = pen.measure(self.ttFont, glyphs=self.ttFont.get_glyphs_for_primary_script())

        return {"value": self.shape_value(stats["weight"])}


class Width(Metric):
    """\
    Measures the width of all letters in the primary script of the font.
    This metric measures the average width of all glyphs as a percentage of the UPM.

    Based on fontTools.pens.statisticsPen.StatisticsPen
    """

    name = "Width"
    keyword = "width"
    data_type = Percentage

    def value(self, includes=None, excludes=None):
        pen = CustomStatisticsPen()
        stats = pen.measure(self.ttFont, glyphs=self.ttFont.get_glyphs_for_primary_script())

        return {"value": self.shape_value(stats["width"])}


class Appearance(Metric):
    name = "Appearance"
    keyword = "appearance"
    children = [StrokeContrastRatio, StrokeContrastAngle, Weight, Width, LowercaseAStyle, LowercaseGStyle, Stencil]
