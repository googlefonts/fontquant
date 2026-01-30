use crate::{
    MetricValue,
    helpers::{
        raycaster::{EAST, NORTH, ProportionalPoint, Raycaster},
        strokecontrast,
    },
    monkeypatching::MakeBezGlyphs,
    quantifier,
};
use fontations::skrifa;

pub fn get_stroke_contrast(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), crate::FontquantError> {
    let Some(lower_o) = font.bezglyph_for_char(location, None, 'o')? else {
        return Ok(());
    };
    if let Some((contrast, angle)) = strokecontrast::stroke_contrast_antiqua(&lower_o) {
        results.add_metric(
            &STROKE_CONTRAST_ANTIQUA,
            MetricValue::Metric(contrast.into()),
        );
        if let Some(angle) = angle {
            results.add_metric(
                &STROKE_CONTRAST_ANTIQUA_ANGLE,
                MetricValue::Angle(angle.into()),
            );
        }
    }

    // Try raycasting approach
    let mut contrasts = vec![];
    let mut raycaster_north = Raycaster::new(&lower_o, ProportionalPoint::new(0.5, 0.0), NORTH);
    raycaster_north.jitter(0.2, 20);
    let mut raycaster_east = Raycaster::new(&lower_o, ProportionalPoint::new(0.0, 0.5), EAST);
    raycaster_east.jitter(0.2, 50);
    if let Some(lower_o_contrast) =
        strokecontrast::stroke_contrast_raycaster(&mut raycaster_north, &mut raycaster_east)
    {
        contrasts.push((lower_o_contrast, 5.0));
    }

    if let Some(cap_h) = font.bezglyph_for_char(location, None, 'H')? {
        let mut raycaster_north = Raycaster::new(&cap_h, ProportionalPoint::new(0.5, 0.0), NORTH);
        let mut raycaster_east = Raycaster::new(&cap_h, ProportionalPoint::new(0.0, 0.15), EAST);
        if let Some(cap_h_contrast) =
            strokecontrast::stroke_contrast_raycaster(&mut raycaster_north, &mut raycaster_east)
        {
            contrasts.push((cap_h_contrast, 3.0));
        }
    }

    if let Some(cap_t) = font.bezglyph_for_char(location, None, 'T')? {
        let mut raycaster_north = Raycaster::new(&cap_t, ProportionalPoint::new(0.1, 0.0), NORTH);
        let mut raycaster_east = Raycaster::new(&cap_t, ProportionalPoint::new(0.0, 0.5), EAST);
        if let Some(cap_t_contrast) =
            strokecontrast::stroke_contrast_raycaster(&mut raycaster_north, &mut raycaster_east)
        {
            contrasts.push((cap_t_contrast, 2.0));
        }
    }

    if let Some(lower_n) = font.bezglyph_for_char(location, None, 'n')? {
        let mut raycaster_north = Raycaster::new(&lower_n, ProportionalPoint::new(0.4, 0.5), NORTH);
        let mut raycaster_east = Raycaster::new(&lower_n, ProportionalPoint::new(0.0, 0.5), EAST);
        if let Some(lower_n_contrast) =
            strokecontrast::stroke_contrast_raycaster(&mut raycaster_north, &mut raycaster_east)
        {
            contrasts.push((lower_n_contrast, 4.0));
        }
    }

    // Possibly do some clever diagonal things with v and x here.

    let weighted_sum: f64 = contrasts.iter().map(|(c, w)| *w * *c as f64).sum();
    let sum_weight: f64 = contrasts.iter().map(|(_, w)| *w).sum();
    if sum_weight > 0.0 {
        results.add_metric(
            &STROKE_CONTRAST_RAYCASTER,
            MetricValue::Metric(weighted_sum / sum_weight),
        );
    }

    Ok(())
}

quantifier!(
    STROKE_CONTRAST_ANTIQUA,
    "stroke_contrast/antiqua",
    r#"The font's contrast of the lower-case 'o' glyph, for well-behaved antiqua fonts.

This quantifier measures the "expansion" stroke contrast of the lower-case 'o' glyph
by comparing the distance of the widest and narrowest parts of the stroke. This works
fine when the o is defined as two ellipses, and the basic stroke model of the font is
that of a broad-nibbed pen; it may get confused by other styles of font, instrokes and
outstrokes, or other glyphs that are not well-behaved antiqua fonts.
    "#,
    MetricValue::Metric(3.0)
);
quantifier!(
    STROKE_CONTRAST_ANTIQUA_ANGLE,
    "stroke_contrast/antiqua_angle",
    "The pen angle of the lower-case 'o' glyph, for well-behaved antiqua fonts.

See the caveats in `stroke_contrast/antiqua` quantifier.
",
    MetricValue::Angle(30.0)
);

quantifier!(
    STROKE_CONTRAST_RAYCASTER,
    "stroke_contrast/raycaster",
    r#"The font's stroke contrast, based on ray-casting.

This quantifier measures the "expansion" stroke contrast based on the horizontal
stroke and vertical stroke thickness of a number of glyphs, using a ray-casting
algorithm, and finds the weighted average of the results.
    "#,
    MetricValue::Metric(3.0)
);
