from fontquant import Metric, Percentage, Integer
from fontquant.helpers.stroke_contrast import stroke_contrast
from beziers.path import BezierPath
from fontquant.helpers.pens import CustomStatisticsPen


class StrokeContrastRatio(Metric):
    """\
    Calculates the ratio of the stroke contrast,
    calculated in thinnest/thickest stroke.
    For now, the lowercase "o" is used for the calculation.

    TODO: Choose test letter based on primary_script
    """

    name = "Stroke Contrast Ratio"
    keyword = "stroke_contrast_ratio"
    data_type = Percentage

    def value(self):
        character = self.ttFont.glyphname_for_char("o")
        width = self.ttFont.getGlyphSet()[character].width
        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, character)
        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        if not hasattr(self.parent, "stroke_values"):
            try:
                self.parent.stroke_values = stroke_contrast(paths, width, ascender, descender)
            except Exception as e:
                return {"value": str(e)}

        return {"value": self.shape_value(self.parent.stroke_values[0])}


class StrokeContrastAngle(Metric):
    """\
    Calculates the angle of the stroke contrast. An angle of 0° means
    vertical contrast, with positive angles being counter-clockwise.
    For now, the lowercase "o" is used for the calculation.
    """

    name = "Stroke Contrast Angle"
    keyword = "stroke_contrast_angle"
    data_type = Integer

    def value(self):
        character = self.ttFont.glyphname_for_char("o")
        width = self.ttFont.getGlyphSet()[character].width
        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, character)
        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        if not hasattr(self.parent, "stroke_values"):
            try:
                self.parent.stroke_values = stroke_contrast(paths, width, ascender, descender)
            except Exception as e:
                return {"value": str(e)}

        return {"value": self.shape_value(self.parent.stroke_values[1])}


class Weight(Metric):
    """\
    Measures the weight of all letters in the primary script of the font.
    This metric measures the amount of ink per glyph as a percentage of an em square
    and returns the average of all glyphs measured.

    TO DO: Base the glyphs to measure on the font's primary script.

    Based on fontTools.pens.statisticsPen.StatisticsPen
    """

    name = "Weight"
    keyword = "weight"
    data_type = Percentage

    def value(self):
        pen = CustomStatisticsPen()
        stats = pen.measure(self.ttFont)

        return {"value": self.shape_value(stats["weight"])}


class Width(Metric):
    """\
    Measures the width of all letters in the primary script of the font.
    This metric measures the average width of all glyphs as a percentage of the UPM.

    TO DO: Base the glyphs to measure on the font's primary script.

        Based on fontTools.pens.statisticsPen.StatisticsPen
    """

    name = "Width"
    keyword = "width"
    data_type = Percentage

    def value(self):
        pen = CustomStatisticsPen()
        stats = pen.measure(self.ttFont)

        return {"value": self.shape_value(stats["width"])}


class Appearance(Metric):
    name = "Appearance"
    keyword = "appearance"
    children = [StrokeContrastRatio, StrokeContrastAngle, Weight, Width]
