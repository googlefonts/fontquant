use crate::{error::FontquantError, monkeypatching::MakeBezGlyphs, quantifier, MetricValue};
use fontations::skrifa::{
    prelude::Size, raw::TableProvider, setting::VariationSetting, FontRef, MetadataProvider,
};

pub fn gather_from_font(
    font: &FontRef,
    location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let upem = font.head()?.units_per_em() as f64;
    let normalized = font.axes().location(location);
    let glyph_metrics = font.glyph_metrics(Size::unscaled(), &normalized);
    // if let Some(x_box) = font
    //     .charmap()
    //     .map('x' as u32)
    //     .and_then(|gid| glyph_metrics.bounds(gid))
    // {
    // Not variation aware, see https://github.com/googlefonts/fontations/issues/1528
    if let Some(x_box) = font
        .bezglyph_for_char(location, None, 'x')?
        .and_then(|bez| bez.bbox())
    {
        results.add_metric(
            &X_HEIGHT,
            MetricValue::PerMille(x_box.max_y() / upem * 1000.0),
        );
    }
    if let Some(space_width) = font
        .charmap()
        .map(' ' as u32)
        .and_then(|gid| glyph_metrics.advance_width(gid))
    {
        results.add_metric(
            &SPACE_WIDTH,
            MetricValue::PerMille(space_width as f64 / upem * 1000.0),
        );
    }

    Ok(())
}

quantifier!(
    X_HEIGHT,
    "x_height",
    "Measures the height of the letter 'x' in font units per mille.",
    MetricValue::PerMille(0.6)
);
quantifier!(
    SPACE_WIDTH,
    "space_width",
    "Measures the width of the space character in font units per mille.",
    MetricValue::PerMille(0.5)
);
