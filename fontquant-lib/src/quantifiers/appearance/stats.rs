use crate::{
    error::FontquantError,
    monkeypatching::{MakeBezGlyphs, PrimaryScript},
    quantifier, MetricValue,
};
use fontations::skrifa;
use fontations::skrifa::{raw::TableProvider, setting::VariationSetting, FontRef};
use greencurves::{ComputeGreenStatistics, CurveStatistics, GreenStatistics};
pub struct WholeFontStatistics {
    pub weight: f64,
    #[allow(dead_code)]
    pub weight_perceptual: f64,
    pub width: f64,
    pub slant: f64,
}
impl WholeFontStatistics {
    pub fn new_from_font(
        font: &FontRef,
        location: &[VariationSetting],
    ) -> Result<Self, FontquantError> {
        let upem = font.head()?.units_per_em() as f64;
        let mut wght_sum = 0.0;
        let mut wght_sum_perceptual = 0.0;
        let mut wdth_sum = 0.0;
        let mut slnt_sum = 0.0;
        let glyphs = font.glyphs_for_primary_script().collect::<Vec<_>>();
        let hmtx = font.hmtx()?;
        // let names = GlyphNames::new(font);

        for glyph_id in glyphs.iter().copied() {
            let glyph_width = hmtx.advance(glyph_id).unwrap_or(0) as f64;
            let Some(bezglyph) =
                font.bezglyph_for_gid(location, Some(1.0 / upem as f32), glyph_id)?
            else {
                continue;
            };
            let glyph_stats = bezglyph
                .iter()
                .map(|p| p.green_statistics())
                .fold(GreenStatistics::default(), |acc, s| acc + s);
            wght_sum += glyph_stats.area().abs();
            wght_sum_perceptual += glyph_stats.area().abs() * glyph_width;
            wdth_sum += glyph_width;
            slnt_sum += glyph_stats.slant();
        }
        Ok(WholeFontStatistics {
            weight: wght_sum * upem / wdth_sum,
            weight_perceptual: wght_sum_perceptual / wdth_sum,
            width: wdth_sum / upem / glyphs.len() as f64,
            slant: -(slnt_sum / glyphs.len() as f64).atan().to_degrees(),
        })
    }

    pub fn gather_from_font(
        font: &skrifa::FontRef,
        location: &[skrifa::setting::VariationSetting],
        results: &mut crate::Results,
    ) -> Result<(), FontquantError> {
        let stats = WholeFontStatistics::new_from_font(font, location)?;
        results.add_metric(&WEIGHT, MetricValue::Metric(stats.weight));
        results.add_metric(&WIDTH, MetricValue::Metric(stats.width));
        results.add_metric(&SLANT, MetricValue::Angle(stats.slant));
        Ok(())
    }
}

quantifier!(WEIGHT,
    "weight",
    "Measures the weight of encoded characters of the font as the amount of ink per glyph as a percentage of an em square and returns the average of all glyphs measured.",
    MetricValue::Percentage(0.12)
);

quantifier!(WIDTH,
    "width",
    "Measures the width of encoded characters of the font as a percentage of the UPM and returns the average of all glyphs measured.",
    MetricValue::Percentage(0.12)
);

quantifier!(SLANT, "slant",
    "Measures the slant angle of encoded characters of the font in degrees and returns the average of all glyphs measured. Right-leaning shapes have negative numbers.",
    MetricValue::Angle(12.0)
);
