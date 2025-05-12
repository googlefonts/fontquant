use itertools::{process_results, Itertools};
use kurbo::{flatten, BezPath, PathEl, PathSeg, Shape};

use crate::{
    bezglyph::{bezpaths_to_skpath, skpath_to_bezglyph, BezGlyph},
    monkeypatching::MakeBezGlyphs,
    quantifier, MetricValue,
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

quantifier!(STENCIL,
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
                let p1_skia = bezpaths_to_skpath(&[path1.clone()]);
                let p2_skia = bezpaths_to_skpath(&[path2.clone()]);
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

fn bezpath_intersects(b1: &BezPath, b2: &BezPath) -> bool {
    if b1.bounding_box().intersect(b2.bounding_box()).area() < f64::EPSILON {
        return false;
    }
    let mut pts1 = vec![];
    let mut pts2 = vec![];
    flatten(b1, 0.1, |el| match el {
        PathEl::MoveTo(a) => pts1.push(a),
        PathEl::LineTo(a) => pts1.push(a),
        _ => {}
    });
    flatten(b2, 0.1, |el| match el {
        PathEl::MoveTo(a) => pts2.push(a),
        PathEl::LineTo(a) => pts2.push(a),
        _ => {}
    });
    for (&la1, &la2) in pts1.iter().circular_tuple_windows() {
        for (&lb1, &lb2) in pts2.iter().circular_tuple_windows() {
            let seg1 = PathSeg::Line(kurbo::Line::new(la1, la2));
            let seg2 = kurbo::Line::new(lb1, lb2);
            if !seg1.intersect_line(seg2).is_empty() {
                return true;
            }
        }
    }
    false
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
                "../../../tests/fonts/AllertaStencil-Regular.ttf"
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
