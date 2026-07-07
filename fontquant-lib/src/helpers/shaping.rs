use std::collections::HashMap;

use harfrust::{GlyphBuffer, ShapeOptions};
use skrifa::{FontRef, GlyphId, MetadataProvider, Tag};

pub fn shape_with_features(font: &FontRef, text: &str, features: &[Tag]) -> GlyphBuffer {
    #[allow(clippy::unwrap_used)] // May God forgive
    let harfrust_fontref = harfrust::FontRef::new(font.data().as_bytes()).unwrap();
    let shaper_data = harfrust::ShaperData::new(&harfrust_fontref);
    let shaper_builder = shaper_data.shaper(&harfrust_fontref);
    let shaper = shaper_builder.build();
    let features = features
        .iter()
        .map(|&tag| harfrust::Feature::new(tag, 1, ..))
        .collect::<Vec<_>>();
    let plan = harfrust::ShapePlan::new(
        &shaper,
        harfrust::Direction::LeftToRight,
        Some(harfrust::script::LATIN),
        None,
        &features,
    );
    let mut buffer = harfrust::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(harfrust::Direction::LeftToRight);
    buffer.set_script(harfrust::script::LATIN);
    let options = ShapeOptions::new().plan(Some(&plan)).features(&features);
    shaper.shape(buffer, options)
}

/// Returns `true` if shaping `text` with `features_a` produces a different result than shaping
/// with `features_b`. This mirrors the Python `differs(vhb, string, features1, features2)` helper
/// which compares two arbitrary feature sets rather than always comparing against the default.
pub fn shapes_differently_between(
    font: &FontRef,
    text: &str,
    features_a: &[Tag],
    features_b: &[Tag],
) -> bool {
    #[allow(clippy::unwrap_used)] // May God forgive
    let harfrust_fontref = harfrust::FontRef::new(font.data().as_bytes()).unwrap();
    let shaper_data = harfrust::ShaperData::new(&harfrust_fontref);
    let shaper_builder = shaper_data.shaper(&harfrust_fontref);
    let shaper = shaper_builder.build();

    let make_plan = |tags: &[Tag]| {
        harfrust::ShapePlan::new(
            &shaper,
            harfrust::Direction::LeftToRight,
            Some(harfrust::script::LATIN),
            None,
            &tags
                .iter()
                .map(|&tag| harfrust::Feature::new(tag, 1, ..))
                .collect::<Vec<_>>(),
        )
    };

    let shaped = |plan: &harfrust::ShapePlan, features: &[Tag]| {
        let hb_features = features
            .iter()
            .map(|&tag| harfrust::Feature::new(tag, 1, ..))
            .collect::<Vec<_>>();
        let mut buf = harfrust::UnicodeBuffer::new();
        buf.push_str(text);
        buf.set_direction(harfrust::Direction::LeftToRight);
        buf.set_script(harfrust::script::LATIN);
        let options = ShapeOptions::new().plan(Some(plan)).features(&hb_features);
        shaper
            .shape(buf, options)
            .serialize(&shaper, harfrust::SerializeFlags::default())
    };

    let plan_a = make_plan(features_a);
    let plan_b = make_plan(features_b);
    shaped(&plan_a, features_a) != shaped(&plan_b, features_b)
}

pub fn shapes_differently_with_features(font: &FontRef, text: &str, features: &[Tag]) -> bool {
    #[allow(clippy::unwrap_used)] // May God forgive
    let harfrust_fontref = harfrust::FontRef::new(font.data().as_bytes()).unwrap();
    let shaper_data = harfrust::ShaperData::new(&harfrust_fontref);
    let shaper_builder = shaper_data.shaper(&harfrust_fontref);
    let shaper = shaper_builder.build();
    let plan = harfrust::ShapePlan::new(
        &shaper,
        harfrust::Direction::LeftToRight,
        Some(harfrust::script::LATIN),
        None,
        &[],
    );
    let mut buffer = harfrust::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(harfrust::Direction::LeftToRight);
    buffer.set_script(harfrust::script::LATIN);
    let options = ShapeOptions::new().plan(Some(&plan));
    let buffer = shaper.shape(buffer, options);
    let glyphs_no_feature = buffer.serialize(&shaper, harfrust::SerializeFlags::default());

    let hb_features = features
        .iter()
        .map(|&tag| harfrust::Feature::new(tag, 1, ..))
        .collect::<Vec<_>>();
    let plan_with_feature = harfrust::ShapePlan::new(
        &shaper,
        harfrust::Direction::LeftToRight,
        Some(harfrust::script::LATIN),
        None,
        &hb_features,
    );
    let mut buffer_with_feature = harfrust::UnicodeBuffer::new();
    buffer_with_feature.push_str(text);
    buffer_with_feature.set_direction(harfrust::Direction::LeftToRight);
    buffer_with_feature.set_script(harfrust::script::LATIN);
    let options_with_feature = ShapeOptions::new()
        .plan(Some(&plan_with_feature))
        .features(&hb_features);
    let buffer_with_feature = shaper.shape(buffer_with_feature, options_with_feature);
    let glyphs_with_feature =
        buffer_with_feature.serialize(&shaper, harfrust::SerializeFlags::default());

    glyphs_no_feature != glyphs_with_feature
}

pub fn ratio_of_different_shapes<T: Fn(char) -> bool>(
    font: &FontRef,
    predicate: T,
    feature: Tag,
) -> f64 {
    let codepoints = font.charmap().mappings().collect::<HashMap<u32, GlyphId>>();
    let chars = codepoints
        .keys()
        .filter_map(|unicode| char::from_u32(*unicode))
        .filter(|c| predicate(*c))
        .collect::<Vec<char>>();
    let char_count = chars.len() as f64;
    let different_shapes_count = chars
        .iter()
        .filter(|c| shapes_differently_with_features(font, &c.to_string(), &[feature]))
        .count() as f64;
    different_shapes_count / char_count
}
