from fontquant import Metric, Percentage, Integer
from fontquant.helpers.stroke_contrast import stroke_contrast
from beziers.path import BezierPath
from fontquant.helpers.pens import CustomStatisticsPen


class StrokeContrastBase(object):
    measure_characters = {"fallback": "o", "Latn": "o"}

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
    For now, the lowercase "o" is used for the calculation.

    TODO: Choose test letter based on primary_script
    """

    name = "Stroke Contrast Ratio"
    keyword = "stroke_contrast_ratio"
    data_type = Percentage

    def value(self):
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
    For now, the lowercase "o" is used for the calculation.
    """

    name = "Stroke Contrast Angle"
    keyword = "stroke_contrast_angle"
    data_type = Integer

    def value(self):
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

    def value(self):
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

    def value(self):
        pen = CustomStatisticsPen()
        stats = pen.measure(self.ttFont, glyphs=self.ttFont.get_glyphs_for_primary_script())

        return {"value": self.shape_value(stats["width"])}


class Appearance(Metric):
    name = "Appearance"
    keyword = "appearance"
    children = [StrokeContrastRatio, StrokeContrastAngle, Weight, Width]
