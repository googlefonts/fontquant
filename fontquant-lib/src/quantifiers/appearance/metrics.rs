use std::collections::{HashMap, HashSet};

use crate::{MetricValue, error::FontquantError, monkeypatching::MakeBezGlyphs, quantifier};
use fontations::{
    read::{ReadError, tables::gdef::GlyphClassDef},
    skrifa::{
        FontRef, MetadataProvider, prelude::Size, raw::TableProvider, setting::VariationSetting,
    },
    types::{GlyphId, GlyphId16},
};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

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
    if let Some(space_width) = font
        .charmap()
        .map('n' as u32)
        .and_then(|gid| glyph_metrics.advance_width(gid))
    {
        results.add_metric(
            &N_WIDTH,
            MetricValue::PerMille(space_width as f64 / upem * 1000.0),
        );
    }
    if let Some(i_width) = font
        .charmap()
        .map('i' as u32)
        .and_then(|gid| glyph_metrics.advance_width(gid))
    {
        results.add_metric(
            &I_WIDTH,
            MetricValue::PerMille(i_width as f64 / upem * 1000.0),
        );
    }
    let stats = glyph_metrics_stats(font)?;
    results.add_metric(&MONOSPACED, MetricValue::Boolean(stats.seems_monospaced));
    results.add_metric(&MOST_COMMON_WIDTH, {
        if stats.most_common_width == 0 {
            MetricValue::PerMille(0.0)
        } else {
            MetricValue::PerMille(stats.most_common_width as f64 / upem * 1000.0)
        }
    });

    Ok(())
}

quantifier!(
    X_HEIGHT,
    "x_height",
    "Measures the height of the letter 'x' in font units per mille.",
    MetricValue::PerMille(0.6)
);
quantifier!(
    N_WIDTH,
    "n_width",
    "Measures the width of the letter 'n' in font units per mille.",
    MetricValue::PerMille(0.6)
);
quantifier!(
    SPACE_WIDTH,
    "space_width",
    "Measures the width of the space character in font units per mille.",
    MetricValue::PerMille(0.5)
);
quantifier!(
    I_WIDTH,
    "i_width",
    "Measures the width of the i character in font units per mille.",
    MetricValue::PerMille(0.5)
);
quantifier!(
    MOST_COMMON_WIDTH,
    "most_common_width",
    "Measures the most common width of all glyphs in font units per mille.",
    MetricValue::PerMille(0.5)
);

quantifier!(
    MONOSPACED,
    "monospaced",
    "Tests to see if the font seems monospaced, based on whether there are fewer than 2 different widths for the ASCII characters.",
    MetricValue::Boolean(false)
);

// Stolen from fontspector

#[allow(dead_code)] // We'll use it one day
struct GlyphMetricsStats {
    // At least 80% of encoded ASCII glyphs have the same width
    seems_monospaced: bool,
    // Largest advance width in the font
    width_max: u16,
    // Most common width
    most_common_width: u16,
}

fn most_common<I>(iter: impl Iterator<Item = I>) -> Option<(I, usize)>
where
    I: Eq,
    I: std::hash::Hash,
{
    let mut map = HashMap::new();
    for item in iter {
        *map.entry(item).or_insert(0) += 1;
    }
    map.into_iter().max_by_key(|(_, count)| *count)
}

fn glyph_metrics_stats(f: &FontRef) -> Result<GlyphMetricsStats, ReadError> {
    let metrics = f.hmtx()?;
    let ascii_glyph_ids = (32u32..127)
        .flat_map(|ch| f.charmap().map(ch))
        .collect::<Vec<_>>();
    // Here we have to be careful of the h_metrics function, because it
    // only returns metrics for the first numLongMetrics glyphs; everything
    // afterwards is repeated, which can throw off our frequency analysis.
    let all_widths = (0..f.maxp()?.num_glyphs().into())
        .flat_map(|id| metrics.advance(GlyphId::new(id as u32)))
        .filter(|x| *x != 0);
    let width_max = all_widths.clone().max().unwrap_or(0);
    let (most_common_width, _most_common_count) = most_common(all_widths).unwrap_or((0, 0));
    if ascii_glyph_ids.len() > 76 {
        // More than 80% of ASCII glyphs are present
        let ascii_widths = ascii_glyph_ids
            .iter()
            .flat_map(|id| metrics.advance(*id))
            .filter(|x| *x != 0);
        let ascii_widths_count = ascii_widths.clone().count() as f32;
        let (_most_common_ascii_width, most_common_ascii_count) =
            most_common(ascii_widths).unwrap_or((0, 0));

        let seems_monospaced = most_common_ascii_count as f32 > ascii_widths_count * 0.8;
        return Ok(GlyphMetricsStats {
            seems_monospaced,
            width_max,
            most_common_width,
        });
    }

    let mut widths = HashSet::new();
    for codepoint in f.charmap().mappings().map(|(u, _gid)| u) {
        #[allow(clippy::unwrap_used)] // We know it's mapped!
        let glyphid = f.charmap().map(codepoint).unwrap();
        // Skip separators, control and GDEF marks
        if char::from_u32(codepoint)
            .map(|c| {
                matches!(
                    c.general_category(),
                    GeneralCategory::LineSeparator
                        | GeneralCategory::ParagraphSeparator
                        | GeneralCategory::Control
                )
            })
            .unwrap_or(false)
            || gdef_class(f, glyphid) == GlyphClassDef::Mark
        {
            continue;
        }
        if let Some(width) = metrics.advance(glyphid)
            && width != 0
        {
            widths.insert(width);
        }
    }

    Ok(GlyphMetricsStats {
        seems_monospaced: widths.len() <= 2,
        width_max,
        most_common_width,
    })
}

pub fn gdef_class(f: &FontRef, glyph_id: impl Into<GlyphId>) -> GlyphClassDef {
    if let Some(Ok(class_def)) = f.gdef().ok().and_then(|gdef| gdef.glyph_class_def()) {
        GlyphId16::try_from(glyph_id.into())
            .map(|gid| class_def.get(gid))
            .map_or(GlyphClassDef::Unknown, GlyphClassDef::new)
    } else {
        GlyphClassDef::Unknown
    }
}
