use crate::{
    helpers::raycaster::{ProportionalPoint, Raycaster, Winding, NORTH},
    monkeypatching::MakeBezGlyphs as _,
    quantifier, MetricValue,
};

quantifier!(
    WAIST,
    "structure/waist",
    r#"Position of crossbars for letters like 'A', 'H', 'e', etc.
"#,
    MetricValue::String("high".to_string())
);

pub(crate) fn measure_waist(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), crate::FontquantError> {
    let Some(h_glyph) = font.bezglyph_for_char(location, Some(1.0), 'H')? else {
        println!("No 'H' glyph found");
        return Ok(());
    };
    let Some(bbox) = h_glyph.bbox() else {
        println!("No bounding box for 'H' glyph");
        return Ok(());
    };
    // Find the crossbar by raycasting vertically through the glyph. We do this in parametric too.
    let mut raycaster = Raycaster::new(
        &h_glyph,
        ProportionalPoint::new(0.5, 0.1),
        crate::helpers::raycaster::Direction::EndPoint(ProportionalPoint::new(0.5, 0.9)),
    );
    raycaster.jitter(0.3, 10);
    raycaster.winding(Winding::Ink);
    let pairs = raycaster.pairs();
    // Find the average y-coordinate of the crossbar pairs.
    let y_values: Vec<f64> = pairs
        .iter()
        .flatten()
        .map(|pt| (pt.0.y + pt.1.y) / 2.0)
        .collect();
    if y_values.is_empty() {
        println!("No crossbar found");
        return Ok(());
    }
    let average_y = y_values.iter().sum::<f64>() / y_values.len() as f64;
    // Don't use bbox.y0 because of descenders; use baseline
    let crossbar_y_proportion = (average_y) / (bbox.y1);
    let waist_position = if crossbar_y_proportion < 0.4 {
        "low"
    } else if crossbar_y_proportion > 0.6 {
        "high"
    } else {
        "middle"
    };
    results.add_metric(&WAIST, MetricValue::String(waist_position.to_string()));
    Ok(())
}
