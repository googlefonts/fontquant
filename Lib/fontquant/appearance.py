import logging
from fontquant import Check, Percentage, Integer
from fontquant.helpers.stroke_contrast import stroke_contrast
from beziers.path import BezierPath
import github_action_utils as gha_utils


class StrokeContrastRatio(Check):
    """\
    Calculates the ratio of the stroke contrast,
    calculated in thinnest/thickest stroke.
    For now, the lowercase "o" is used for the calculation.
    """

    name = "Stroke Contrast Ratio"
    keyword = "stroke_contrast_ratio"
    data_type = Percentage

    def value(self):
        character = "o" if "o" in self.ttFont.getGlyphSet() else "uni006F"
        width = self.ttFont.getGlyphSet()[character].width
        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, character)
        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        if not hasattr(self.parent, "stroke_values"):
            try:
                self.parent.stroke_values = stroke_contrast(paths, width, ascender, descender)
            except Exception as e:
                gha_utils.error(str(e))
                return {"value": None, "error": str(e)}

        return {"value": self.parent.stroke_values[0]}


class StrokeContrastAngle(Check):
    """\
    Calculates the angle of the stroke contrast. An angle of 0° means
    vertical contrast, with positive angles being counter-clockwise.
    For now, the lowercase "o" is used for the calculation.
    """

    name = "Stroke Contrast Angle"
    keyword = "stroke_contrast_angle"
    data_type = Integer

    def value(self):
        character = "o" if "o" in self.ttFont.getGlyphSet() else "uni006F"
        width = self.ttFont.getGlyphSet()[character].width
        paths = BezierPath.fromFonttoolsGlyph(self.ttFont, character)
        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        if not hasattr(self.parent, "stroke_values"):
            try:
                self.parent.stroke_values = stroke_contrast(paths, width, ascender, descender)
            except Exception as e:
                gha_utils.error(str(e))
                return {"value": None, "error": str(e)}

        return {"value": self.parent.stroke_values[1]}


class Appearance(Check):
    name = "Appearance"
    keyword = "appearance"
    children = [StrokeContrastRatio, StrokeContrastAngle]
