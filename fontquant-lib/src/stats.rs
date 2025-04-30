use std::sync::LazyLock;

use crate::bezglyph::ScalerPen;
use crate::error::FontquantError;
use crate::{bezglyph::BezGlyph, monkeypatching::PrimaryScript};
use crate::{MetricGatherer, MetricKey, MetricValue};
use greencurves::{ComputeGreenStatistics, CurveStatistics, GreenStatistics};
use skrifa::{raw::TableProvider, setting::VariationSetting, FontRef, MetadataProvider};

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
        let collection = font.outline_glyphs();
        let loc = font.axes().location(location);
        let upem = font.head()?.units_per_em() as f64;
        let mut wght_sum = 0.0;
        let mut wght_sum_perceptual = 0.0;
        let mut wdth_sum = 0.0;
        let mut slnt_sum = 0.0;
        let glyphs = font.glyphs_for_primary_script().collect::<Vec<_>>();
        let hmtx = font.hmtx()?;
        // let names = GlyphNames::new(font);

        for glyph_id in glyphs.iter().copied() {
            let settings =
                skrifa::outline::DrawSettings::unhinted(skrifa::prelude::Size::unscaled(), &loc);
            let outlined = collection.get(glyph_id).ok_or(FontquantError::SkrifaDraw(
                skrifa::outline::DrawError::GlyphNotFound(glyph_id),
            ))?;
            let mut bezglyph = BezGlyph::default();
            let glyph_width = hmtx.advance(glyph_id).unwrap_or(0) as f64;
            let mut scaled_bezglyph = ScalerPen::new(&mut bezglyph, 1.0 / upem as f32);

            outlined.draw(settings, &mut scaled_bezglyph)?;
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
}

static WEIGHT: LazyLock<MetricKey> = LazyLock::new(|| {
    MetricKey {
    description: "Measures the weight of encoded characters of the font as the amount of ink per glyph as a percentage of an em square and returns the average of all glyphs measured.".to_string(),
    name: "weight".to_string(),
    example_value: MetricValue::Percentage(0.12),
}
});
static WIDTH: LazyLock<MetricKey> = LazyLock::new(|| {
    MetricKey {
    description: "Measures the width of encoded characters of the font as a percentage of the UPM and returns the average of all glyphs measured.".to_string(),
    name: "width".to_string(),
    example_value: MetricValue::Percentage(0.12),
}
});
static SLANT: LazyLock<MetricKey> = LazyLock::new(|| {
    MetricKey {
    description: "Measures the slant angle of encoded characters of the font in degrees and returns the average of all glyphs measured. Right-leaning shapes have negative numbers.".to_string(),
    name: "slant".to_string(),
    example_value: MetricValue::Angle(12.0),
}
});

impl MetricGatherer for WholeFontStatistics {
    fn gather_from_font(
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
