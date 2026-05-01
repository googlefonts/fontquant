from fontquant import Metric, Percentage, String, Boolean, Angle, PerMille, Integer
from fontquant.helpers.stroke_contrast import stroke_contrast
from beziers.path import BezierPath
from fontquant.helpers.pens import CustomStatisticsPen
from fontquant.helpers.bezier import removeOverlaps
from fontquant.helpers.var import instance_dict_to_str
from fontquant.helpers.settings import get_script_setting
from fontquant.helpers.raycasting import Raycaster
from fontquant.casing import Unicase
from math import degrees, atan
import statistics


class Stencil(Metric):
    """\
    Reports whether or not a font is a stencil font.

    It recognizes a stencil font correctly, but may sometimes mis-report non-stencil fonts
    as stencil fonts because it only looks at a limited set of characters for speed optimization.
    """

    name = "Stencil"
    keyword = "stencil"
    data_type = Boolean
    # These need to be letters with counters:
    measure_characters = {
        "fallback": ["A", "O", "a", "e", "o", "p"],
        "Arab": ["ه", "و"],
    }

    def is_stencil(self, glyph):
        try:
            paths = BezierPath.fromFonttoolsGlyph(self.ttFont, glyph)
            paths = removeOverlaps(paths)

            if paths and sum([path.length for path in paths]) > 0:
                for path1 in paths:
                    for path2 in paths:
                        if path1 != path2:
                            if path1.intersection(path2):
                                return False
                            if path1.intersection(path2.reverse()):
                                return False

                return True

            return False
        except Exception:
            return False

    def value(self, includes=None, excludes=None):
        glyphs = self.measure_characters["fallback"]
        if self.ttFont.get_primary_script() in self.measure_characters:
            glyphs = self.measure_characters[self.ttFont.get_primary_script()]
        glyphs = [self.ttFont.glyphname_for_char(glyph) for glyph in glyphs]

        if any(glyphs):
            return {"value": all([self.is_stencil(glyph) for glyph in glyphs])}
        else:
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
    fully_automatic = False

    def value(self, includes=None, excludes=None):
        stencil = Stencil(self.ttFont, self.vhb, self.variable).value()["value"]
        unicase = Unicase(self.ttFont, self.vhb, self.variable).value()["value"]

        glyph = self.ttFont.glyphname_for_char("a")
        if glyph and not stencil and not unicase:
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
                elif slant > 0.2:
                    threshold *= 1.4

                if ratio > threshold:
                    return {
                        "value": "double_story",
                    }
                else:
                    return {
                        "value": "single_story",
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
    fully_automatic = False

    def value(self, includes=None, excludes=None):
        stencil = Stencil(self.ttFont, self.vhb, self.variable).value()["value"]
        unicase = Unicase(self.ttFont, self.vhb, self.variable).value()["value"]

        glyph = self.ttFont.glyphname_for_char("g")
        if glyph and not stencil and not unicase:
            paths = BezierPath.fromFonttoolsGlyph(self.ttFont, glyph)
            paths = removeOverlaps(paths)

            # Two or three segments are valid
            if paths and len(paths) == 2:
                return {"value": "single_story"}

            elif paths and len(paths) == 3:
                return {"value": "double_story"}

        return {"value": None}


class StrokeContrastBase(Metric):
    def get_character_to_measure(self):

        if self.parent.parent.primary_script and get_script_setting(
            self.parent.parent.primary_script, "stroke_contrast_glyphs"
        ):
            characters = str(get_script_setting(self.parent.parent.primary_script, "stroke_contrast_glyphs")).split(
                ","
            )
        else:
            characters = str(get_script_setting("Latn", "stroke_contrast_glyphs")).split(",")

        return characters[0]

    def measure(self):
        character = self.get_character_to_measure()
        print(f"Measure stroke using '{character}'")
        width = 1000

        descender = self.ttFont["hhea"].descender
        ascender = self.ttFont["hhea"].ascender
        assert descender <= 0

        self.parent.parent._stroke_values = {}

        if self.variable:
            for instance in self.variable:

                paths = self.paths_for_glyph(character, instance)

                self.parent.parent._stroke_values[instance_dict_to_str(instance)] = stroke_contrast(
                    paths, width, ascender, descender, show=self.parent.parent.debug and self.parent.parent.show
                )
        else:
            paths = self.paths_for_glyph(character)

            self.parent.parent._stroke_values["default"] = stroke_contrast(
                paths, width, ascender, descender, show=self.parent.parent.debug and self.parent.parent.show
            )

    def value(self, includes=None, excludes=None):

        response = {}

        if not hasattr(self.parent.parent, "_stroke_values"):
            self.measure()

        if self.variable:
            values = {}
            for instance in self.variable:
                values[instance_dict_to_str(instance)] = self.parent.parent._stroke_values[
                    instance_dict_to_str(instance)
                ][self.result_index]

            response["value"] = values
        else:
            response["value"] = self.parent.parent._stroke_values["default"][self.result_index]

        if self.parent.parent.debug:
            response["character"] = self.get_character_to_measure()

        return response


class StrokeContrastRatio(StrokeContrastBase):
    """\
    Calculates the ratio of the stroke contrast,
    calculated in thinnest/thickest stroke.

    One representative character is measured for the font's primary script,
    such as the "o" for Latin.

    Note that the two stroke contrast metrics (ratio and angle) are calculated in the same function.
    For efficiency, query both metrics at during the same call.
    """

    name = "Stroke Contrast Ratio"
    keyword = "stroke_contrast_ratio"
    data_type = Percentage
    variable_aware = True
    result_index = 0


class StrokeContrastAngle(StrokeContrastBase):
    """\
    Calculates the angle of the stroke contrast. An angle of 0° means
    vertical contrast, with positive angles being counter-clockwise.

    One representative character is measured for the font's primary script,
    such as the "o" for Latin.

    Note that the two stroke contrast metrics (ratio and angle) are calculated in the same function.
    For efficiency, query both metrics at during the same call.
    """

    name = "Stroke Contrast Angle"
    keyword = "stroke_contrast_angle"
    data_type = Angle
    variable_aware = True
    result_index = 1


class StatisticsPenMetrics(Metric):
    value_name = None

    def value(self, includes=None, excludes=None):

        if self.variable:
            values = {}
            for instance in self.variable:
                pen = CustomStatisticsPen()
                stats = pen.measure(self.ttFont, location=instance, glyphs=self.ttFont.get_glyphs_for_primary_script())
                values[instance_dict_to_str(instance)] = self.shape_value(stats[self.value_name])

            return {"value": values}
        else:
            pen = CustomStatisticsPen()
            stats = pen.measure(self.ttFont, glyphs=self.ttFont.get_glyphs_for_primary_script())

            return {"value": self.shape_value(self.value_adjustment(stats[self.value_name]))}

    def value_adjustment(self, value):
        return value


class Weight(StatisticsPenMetrics):
    """\
    Measures the weight of encoded characters of the font
    as the amount of ink per glyph as a percentage of an em square
    and returns the average of all glyphs measured.
    Based on `fontTools.pens.statisticsPen.StatisticsPen`
    """

    name = "Weight"
    keyword = "weight"
    data_type = Percentage
    variable_aware = True
    value_name = "weight"


class Width(StatisticsPenMetrics):
    """\
    Measures the width of encoded characters of the font
    as a percentage of the UPM
    and returns the average of all glyphs measured.
    Based on `fontTools.pens.statisticsPen.StatisticsPen`
    """

    name = "Width"
    keyword = "width"
    data_type = Percentage
    variable_aware = True
    value_name = "width"


class Slant(StatisticsPenMetrics):
    """\
    Measures the slanted angle of encoded characters of the font
    in degrees and returns the average of all glyphs measured.
    Right-leaning shapes have negative numbers.
    Based on `fontTools.pens.statisticsPen.StatisticsPen`
    """

    name = "Slant"
    keyword = "slant"
    data_type = Angle
    variable_aware = True
    value_name = "slant"

    def value_adjustment(self, value):
        return -degrees(atan(value))


class VerticalMetrics(Metric):
    def value(self, includes=None, excludes=None):
        if self.variable:
            values = {}
            for instance in self.variable:

                paths = self.paths_for_glyph(self.character, instance)

                values[instance_dict_to_str(instance)] = self.shape_value(
                    self.specific_value(paths) * 1000 / self.ttFont["head"].unitsPerEm
                )

            return {"value": values}
        else:
            paths = self.paths_for_glyph(self.character)

            return {"value": self.shape_value(self.specific_value(paths) * 1000 / self.ttFont["head"].unitsPerEm)}

    def specific_value(self, paths):
        raise NotImplementedError


class XHeight(VerticalMetrics):
    """\
    Reports x-height of `x`.
    """

    name = "x-Height"
    keyword = "x_height"
    variable_aware = True
    data_type = PerMille
    character = "x"

    def specific_value(self, paths):
        return max([path.bounding_box().max_y() for path in paths])


class CapHeight(VerticalMetrics):
    """\
    Reports cap-height of `H`.
    """

    name = "Cap-Height"
    keyword = "cap_height"
    variable_aware = True
    data_type = PerMille
    character = "H"

    def specific_value(self, paths):
        return max([path.bounding_box().max_y() for path in paths])


class Ascender(VerticalMetrics):
    """\
    Reports ascender of `h`.
    """

    name = "Ascender"
    keyword = "ascender"
    variable_aware = True
    data_type = PerMille
    character = "h"

    def specific_value(self, paths):
        return max([path.bounding_box().max_y() for path in paths])


class Descender(VerticalMetrics):
    """\
    Reports descender of `y`.
    """

    name = "Descender"
    keyword = "descender"
    variable_aware = True
    data_type = PerMille
    character = "y"

    def specific_value(self, paths):
        return min([path.bounding_box().min_y() for path in paths])


class Parametric(Metric):

    variable_aware = True
    data_type = Integer

    many_samples = 12

    parama = {
        "XOPQ": dict(glyph="H", start_point=(0.0, 0.5), direction="E", winding="ink", samples=many_samples),
        "XOLC": dict(glyph="n", start_point=(0.0, 0.4), direction="E", winding="ink", samples=many_samples),
        "XOFI": dict(glyph="1", start_point=(0.0, 0.5), direction="E", winding="ink"),
        "XTRA": dict(glyph="H", start_point=(0.0, 0.5), direction="E", winding="transparent", samples=many_samples),
        "XTLC": dict(glyph="n", start_point=(0.0, 0.4), direction="E", winding="transparent", samples=many_samples),
        "XTFI": dict(glyph="0", start_point=(0.0, 0.5), jittering=0.05, direction="E", winding="transparent"),
        "YOPQ": dict(
            glyph="H", start_point=(0.5, 0.25), direction="N", samples=int(many_samples * 0.8), winding="ink"
        ),
        "YOLC": dict(
            glyph="f",
            start_point=(0.75, 0.5),
            end_point=(0.75, 0.9),
            jittering=0.1,
            samples=many_samples // 2,
            winding="ink",
        ),
        "YOFI": dict(
            glyph="0", start_point=(0.5, 0), direction="N", samples=many_samples, jittering=0.05, winding="ink"
        ),
    }

    def value(self, includes=None, excludes=None):

        if self.variable:
            values = {}
            for instance in self.variable:

                paths = self.paths_for_glyph(self.parama[self.__class__.__name__]["glyph"], instance)
                values[instance_dict_to_str(instance)] = self.shape_value(self.specific_value(paths))

            return {"value": values}
        else:

            paths = self.paths_for_glyph(self.parama[self.__class__.__name__]["glyph"])
            return {"value": self.shape_value(self.specific_value(paths))}

    def specific_value(self, paths):
        parameter = self.__class__.__name__
        settings = self.parama[parameter]
        caster = Raycaster(
            paths,
            settings["start_point"],
            endpoint=settings.get("end_point"),
            direction=settings.get("direction"),
        )
        if "winding" in settings:
            caster = caster.winding(settings["winding"])
        caster = caster.jitter(settings.get("jittering", 0.2), settings.get("samples", 10))
        pairs = caster.pairs()
        try:
            value = round(pairs.median_pair_distance())
            # Upem scale
            value = round(value * 1000 / self.ttFont["head"].unitsPerEm)
        except statistics.StatisticsError:
            print(f"Could not determine {parameter}")
            value = None

        return value


class XOPQ(Parametric):
    """\
    Reports parametric axis XOPQ.
    """

    name = "XOPQ"
    keyword = "XOPQ"


class XOLC(Parametric):
    """\
    Reports parametric axis XOLC.
    """

    name = "XOLC"
    keyword = "XOLC"


class XOFI(Parametric):
    """\
    Reports parametric axis XOFI.
    """

    name = "XOFI"
    keyword = "XOFI"


class XTRA(Parametric):
    """\
    Reports parametric axis XTRA.
    """

    name = "XTRA"
    keyword = "XTRA"


class XTLC(Parametric):
    """\
    Reports parametric axis XTLC.
    """

    name = "XTLC"
    keyword = "XTLC"


class XTFI(Parametric):
    """\
    Reports parametric axis XTFI.
    """

    name = "XTFI"
    keyword = "XTFI"


class YOPQ(Parametric):
    """\
    Reports parametric axis YOPQ.
    """

    name = "YOPQ"
    keyword = "YOPQ"


class YOLC(Parametric):
    """\
    Reports parametric axis YOLC.
    """

    name = "YOLC"
    keyword = "YOLC"


class YOFI(Parametric):
    """\
    Reports parametric axis YOFI.
    """

    name = "YOFI"
    keyword = "YOFI"


class Appearance(Metric):
    name = "Appearance"
    keyword = "appearance"
    children = [
        StrokeContrastRatio,
        StrokeContrastAngle,
        Weight,
        Width,
        Slant,
        LowercaseAStyle,
        LowercaseGStyle,
        Stencil,
        XHeight,
        CapHeight,
        Ascender,
        Descender,
        XOPQ,
        XOLC,
        XOFI,
        XTRA,
        XTLC,
        XTFI,
        YOPQ,
        YOLC,
        YOFI,
    ]
