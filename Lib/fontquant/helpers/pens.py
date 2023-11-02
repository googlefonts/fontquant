from fontTools.pens.statisticsPen import StatisticsPen


class CustomStatisticsPen(StatisticsPen):
    def measure(self, ttFont, location=None, glyphs=None):
        if not glyphs:
            glyphs = ttFont.getGlyphOrder()

        return self._measure(
            ttFont.getGlyphSet(location=location),
            ttFont["head"].unitsPerEm,
            glyphs,
        )

    def _measure(self, glyphset, upem, glyphs):
        # Taken from https://github.com/fonttools/fonttools/blob/main/Lib/fontTools/pens/statisticsPen.py#L73

        from fontTools.pens.transformPen import TransformPen
        from fontTools.misc.transform import Scale

        wght_sum = 0
        wght_sum_perceptual = 0
        wdth_sum = 0
        slnt_sum = 0
        slnt_sum_perceptual = 0
        for glyph_name in glyphs:
            if glyph_name in glyphset:
                # I don't know why but even though I check for glyph names in glypset,
                # I still get KeyError exceptions. So I'm wrapping this in a try/except.
                # I don't even know why glyphs are missing in the first place,
                # because glyphs variable is populated with ttFont.getGlyphOrder().
                try:
                    glyph = glyphset[glyph_name]
                    pen = StatisticsPen(glyphset=glyphset)
                    transformer = TransformPen(pen, Scale(1.0 / upem))
                    glyph.draw(transformer)

                    wght_sum += abs(pen.area)
                    wght_sum_perceptual += abs(pen.area) * glyph.width
                    wdth_sum += glyph.width
                    slnt_sum += pen.slant
                    slnt_sum_perceptual += pen.slant * glyph.width
                except KeyError:
                    pass

        return {
            "weight": wght_sum * upem / wdth_sum,
            "weight_perceptual": wght_sum_perceptual / wdth_sum,
            "width": wdth_sum / upem / len(glyphs),
            "slant": slnt_sum / len(glyphs),
        }
