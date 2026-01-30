use fontations::skrifa;
use itertools::process_results;
use kurbo::Shape;

use crate::{
    MetricValue,
    bezglyph::{BezGlyph, bezpaths_to_skpath, skpath_to_bezglyph},
    monkeypatching::MakeBezGlyphs,
    quantifier,
};

pub fn is_stencil_font(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), crate::FontquantError> {
    let characters = ['A', 'O', 'a', 'e', 'o', 'p'];
    let glyphs_we_have = characters
        .iter()
        .flat_map(|c| font.bezglyph_for_char(location, None, *c).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    let result = process_results(glyphs_we_have.iter().map(is_stencil_glyph), |mut iter| {
        iter.all(|c| c)
    })?;
    results.add_metric(&STENCIL, MetricValue::Boolean(result));
    Ok(())
}

quantifier!(
    STENCIL,
    "stencil",
    "Reports whether or not a font is a stencil font. It recognizes a stencil font correctly, but may sometimes mis-report non-stencil fonts as stencil fonts because it only looks at a limited set of characters for speed optimization.",
    MetricValue::Boolean(false)
);

fn is_stencil_glyph(glyph: &BezGlyph) -> Result<bool, crate::FontquantError> {
    let simplified = glyph.remove_overlaps()?;
    let total_length = simplified.iter().map(|p| p.perimeter(0.01)).sum::<f64>();
    if simplified.iter().count() > 0 && total_length > 0.0 {
        for path1 in simplified.iter() {
            for path2 in simplified.iter() {
                if path1 == path2 {
                    continue;
                }
                // Check if there is no boolean intersection between the two paths
                // (i.e. not just that the paths don't cross, but that there is no intersection area)
                let p1_skia = bezpaths_to_skpath(std::slice::from_ref(path1));
                let p2_skia = bezpaths_to_skpath(std::slice::from_ref(path2));
                let intersection = skia_safe::op(&p1_skia, &p2_skia, skia_safe::PathOp::Intersect)
                    .unwrap_or_default();
                let bg: BezGlyph = skpath_to_bezglyph(intersection);
                if !bg.is_empty() {
                    return Ok(false);
                }
            }
        }
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use skrifa::FontRef;

    use super::*;

    #[test]
    fn test_stencil() {
        #![allow(clippy::unwrap_used, clippy::expect_used)]
        let mut results = crate::Results::new();
        is_stencil_font(
            &FontRef::new(include_bytes!(
                "../../../../tests/fonts/AllertaStencil-Regular.ttf"
            ))
            .unwrap(),
            &[],
            &mut results,
        )
        .expect("Shouldn't fail");
        println!("{:?}", results);
        assert_eq!(
            results.get("stencil").unwrap().1,
            MetricValue::Boolean(true)
        );
    }
}
