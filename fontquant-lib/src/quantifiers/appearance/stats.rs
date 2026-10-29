use crate::{
    bezglyph::BezGlyph,
    error::FontquantError,
    monkeypatching::{MakeBezGlyphs, PrimaryScript},
    quantifier, MetricValue,
};
use kurbo::{ParamCurveMoments, Point, Shape, Vec2};
use skrifa::{self, prelude::Size, MetadataProvider};
use skrifa::{raw::TableProvider, setting::VariationSetting, FontRef};

const EPSILON: f64 = 0.001;
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

        for glyph_id in glyphs.iter().copied() {
            let normalized = font.axes().location(location);

            let glyph_width = font
                .glyph_metrics(Size::unscaled(), &normalized)
                .advance_width(glyph_id)
                .unwrap_or(0.0) as f64;
            let Some(bezglyph) =
                font.bezglyph_for_gid(location, Some(1.0 / upem as f32), glyph_id)?
            else {
                continue;
            };
            let slant = glyph_slant(&bezglyph)?;
            let area = bezglyph
                .iter()
                .map(|p| p.area())
                .fold(0.0, |acc, s| acc + s);

            wght_sum += area.abs();
            wght_sum_perceptual += area.abs() * glyph_width;
            wdth_sum += glyph_width;
            slnt_sum += slant;
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
        results.add_metric(
            &WEIGHT_PERCEPTUAL,
            MetricValue::Metric(stats.weight_perceptual),
        );
        Ok(())
    }
}

fn glyph_slant(bezglyph: &BezGlyph) -> Result<f64, FontquantError> {
    let bezglyph = bezglyph.remove_overlaps()?;

    let area = bezglyph.iter().map(|p| p.area()).sum::<f64>();
    if area == 0.0 {
        return Ok(0.0);
    }

    let (moment_x, moment_y, moment_yy, moment_xy) =
        bezglyph
            .iter()
            .fold((0.0, 0.0, 0.0, 0.0), |(mx, my, myy, mxy), path| {
                let moments = path.moments();
                (
                    mx + moments.moment_x,
                    my + moments.moment_y,
                    myy + moments.moment_yy,
                    mxy + moments.moment_xy,
                )
            });

    let mean_x = moment_x / area;
    let mean_y = moment_y / area;
    let variance_y = (moment_yy / area - mean_y * mean_y).abs();
    let covariance = moment_xy / area - mean_x * mean_y;
    let slant = if variance_y != 0.0 {
        covariance / variance_y
    } else {
        f64::NAN
    };
    if slant.abs() > EPSILON {
        Ok(slant)
    } else {
        Ok(0.0)
    }
}

quantifier!(
    WEIGHT,
    "appearance/weight",
    "Measures the weight of encoded characters of the font as the amount of ink per glyph as a percentage of an em square and returns the average of all glyphs measured.",
    MetricValue::Percentage(0.12)
);

quantifier!(
    WEIGHT_PERCEPTUAL,
    "appearance/weight_perceptual",
    "Measures the weight of encoded characters of the font as the amount of ink per glyph and returns the average of all glyphs measured.",
    MetricValue::Metric(123.0)
);

quantifier!(
    WIDTH,
    "appearance/width",
    "Measures the width of encoded characters of the font as a percentage of the UPM and returns the average of all glyphs measured.",
    MetricValue::Percentage(0.12)
);

quantifier!(
    SLANT,
    "appearance/slant",
    "Measures the slant angle of encoded characters of the font in degrees and returns the average of all glyphs measured. Right-leaning shapes have negative numbers.",
    MetricValue::Angle(12.0)
);

pub(crate) trait CurveStatistics {
    fn slant(&self) -> f64;
    fn center_of_mass(&self) -> Point;
    fn variance(&self) -> Vec2;
    fn covariance(&self) -> f64;
}

impl CurveStatistics for kurbo::BezPath {
    fn slant(&self) -> f64 {
        let slant = self.covariance() / self.variance().y;
        if slant.abs() > EPSILON {
            slant
        } else {
            0.0
        }
    }
    fn center_of_mass(&self) -> Point {
        let area = self.area();
        let moments = self.moments();
        Point::new(moments.moment_x / area, moments.moment_y / area)
    }
    /// Find the variance of the path
    fn variance(&self) -> Vec2 {
        let moments = self.moments();
        let area = self.area();
        let mean = self.center_of_mass();
        Vec2::new(
            (moments.moment_xx / area - mean.x * mean.x).abs(),
            (moments.moment_yy / area - mean.y * mean.y).abs(),
        )
    }

    /// Find the covariance of the path
    fn covariance(&self) -> f64 {
        let area = self.area();
        let mean = self.center_of_mass();
        let moments = self.moments();
        moments.moment_xy / area - mean.x * mean.y
    }
}

#[allow(clippy::unwrap_used)] // it's a test suite
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slant_not_extreme() {
        let fontref = skrifa::FontRef::new(include_bytes!(
            "../../../../tests/fonts/Wallpoet-Subset.ttf"
        ))
        .unwrap();
        let stats = WholeFontStatistics::new_from_font(&fontref, &[]).unwrap();
        // This is an upright font
        assert!(
            stats.slant.abs() < 2.0,
            "Slant is too extreme: {}",
            stats.slant
        );
    }
}
