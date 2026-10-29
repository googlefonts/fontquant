// Settings:

use std::collections::HashSet;

use read_fonts::TableProvider;
use skrifa::{
    FontRef, GlyphId, MetadataProvider, Tag,
    instance::{LocationRef, Size},
    setting::VariationSetting,
};

use crate::{
    MetricValue,
    error::FontquantError,
    helpers::shaping::{
        ratio_of_different_shapes, shape_with_features, shapes_differently_between,
        shapes_differently_with_features,
    },
    monkeypatching::MakeBezGlyphs,
    quantifier,
};

/// How much of the height of an "x" must the upper and lower bbox variance of numerals
/// be to be considered PON?
const PON_THRESHOLD: f64 = 0.1;

/// How much may the width variance of table figures be to be considered TLN_MATRIX?
/// Measured in variance / UPM.
const TLN_THRESHOLD: f64 = 0.005;

/// Constants:
const NUMERALS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const ENCODED_FRACTIONS: &[&str] = &[
    "1/4", "1/2", "3/4", "1/7", "1/9", "1/10", "1/3", "2/3", "1/5", "2/5", "3/5", "4/5", "1/6",
    "5/6", "1/8", "3/8", "5/8", "7/8", "1/", "0/3",
];

const ARBITRARY_FRACTIONS: &[&str] = &["12/99", "1/3", "1234/9876", "1/1234567890"];

// Numeral sets:
const PON_LABEL: &str = "proportional_oldstyle";
const TON_LABEL: &str = "tabular_oldstyle";
const PLN_LABEL: &str = "proportional_lining";
const TLN_LABEL: &str = "tabular_lining";

// Activation matrix
const PON_MATRIX: (&str, &str) = ("onum", "pnum");
const TON_MATRIX: (&str, &str) = ("onum", "tnum");
const PLN_MATRIX: (&str, &str) = ("lnum", "pnum");
const TLN_MATRIX: (&str, &str) = ("lnum", "tnum");

#[allow(clippy::unwrap_used)] // We know these are valid tags
fn tags_from_matrix(matrix: (&str, &str)) -> Vec<Tag> {
    vec![
        Tag::new_checked(matrix.0.as_bytes()).unwrap(),
        Tag::new_checked(matrix.1.as_bytes()).unwrap(),
    ]
}

fn shaped_numeral(font: &FontRef, text: &str, features: &[Tag]) -> Option<GlyphId> {
    let glyphs = shape_with_features(font, text, features);
    // For our purposes, we expect one glyph
    let info = glyphs.glyph_infos().iter().next()?;
    Some(info.glyph_id.into())
}

fn vertical_variance(
    font: &FontRef,
    features: &[Tag],
) -> Result<Option<(f64, f64)>, FontquantError> {
    let mut upper = vec![];
    let mut lower = vec![];
    for &numeral in NUMERALS {
        let Some(gid) = shaped_numeral(font, &numeral.to_string(), features) else {
            continue;
        };
        let Some(bbox) = font
            .bezglyph_for_gid(&[], None, gid)?
            .and_then(|g| g.bbox())
        else {
            continue;
        };
        upper.push(bbox.max_y());
        lower.push(bbox.min_y());
    }
    if let Some(max_upper) = upper.iter().copied().reduce(f64::max)
        && let Some(min_upper) = upper.iter().copied().reduce(f64::min)
        && let Some(max_lower) = lower.iter().copied().reduce(f64::max)
        && let Some(min_lower) = lower.iter().copied().reduce(f64::min)
    {
        Ok(Some((max_upper - min_upper, max_lower - min_lower)))
    } else {
        Ok(None)
    }
}

fn horizontal_variance(font: &FontRef, features: &[Tag]) -> Result<f64, FontquantError> {
    let mut width = vec![];
    let glyph_metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    for &numeral in NUMERALS {
        let Some(gid) = shaped_numeral(font, &numeral.to_string(), features) else {
            continue;
        };
        let Some(advance) = glyph_metrics.advance_width(gid) else {
            continue;
        };
        width.push(advance);
    }
    if let Some(max_width) = width.iter().copied().reduce(f32::max)
        && let Some(min_width) = width.iter().copied().reduce(f32::min)
    {
        Ok((max_width - min_width).into())
    } else {
        Ok(0.0)
    }
}

fn numeral_style_heuristics(
    font: &FontRef,
    features: &[Tag],
) -> Result<&'static str, FontquantError> {
    let x_height = font
        .bezglyph_for_char(&[], None, 'x')?
        .and_then(|g| g.bbox())
        .map(|b| b.max_y() - b.min_y())
        .unwrap_or(0.0);
    let (upper_variance, lower_variance) = vertical_variance(font, features)?.unwrap_or((0.0, 0.0));
    // Vertical:
    // Compare upper and lower bbox variance to x-height
    let vertical =
        if upper_variance > x_height * PON_THRESHOLD && lower_variance > x_height * PON_THRESHOLD {
            "onum"
        } else {
            "lnum"
        };
    // Allow some variance in width
    let horizontal = if horizontal_variance(font, features)? / (font.head()?.units_per_em() as f64)
        < TLN_THRESHOLD
    {
        "tnum"
    } else {
        "pnum"
    };

    match (vertical, horizontal) {
        PON_MATRIX => Ok(PON_LABEL),
        TON_MATRIX => Ok(TON_LABEL),
        PLN_MATRIX => Ok(PLN_LABEL),
        TLN_MATRIX => Ok(TLN_LABEL),
        _ => unreachable!(),
    }
}

quantifier!(
    TON,
    "numerals/tabular_oldstyle",
    r#"Returns a boolean of whether or not the font has functioning set of _tabular oldstyle_ numerals,
    either by default or activatable by the `onum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't."#,
    MetricValue::Boolean(true)
);
quantifier!(
    PON,
    "numerals/proportional_oldstyle",
    r#"Returns a boolean of whether or not the font has functioning set of _proportional oldstyle_ numerals,
    either by default or activatable by the `onum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't."#,
    MetricValue::Boolean(true)
);
quantifier!(
    PLN,
    "numerals/proportional_lining",
    r#"Returns a boolean of whether or not the font has functioning set of _proportional lining_ numerals,
    either by default or activatable by the `lnum`/`pnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't."#,
    MetricValue::Boolean(true)
);
quantifier!(
    TLN,
    "numerals/tabular_lining",
    r#"Returns a boolean of whether or not the font has functioning set of _tabular lining_ numerals,
    either by default or activatable by the `lnum`/`tnum` features.
    This check also performs heuristics to see whether the activated numeral set matches the common
    expectations on width/height variance and returns `False` if it doesn't."#,
    MetricValue::Boolean(true)
);
quantifier!(
    DEFAULT_NUMERALS,
    "numerals/default_numerals",
    r#"Returns the default numeral set
    (out of `proportional_oldstyle`, `tabular_oldstyle`, `proportional_lining`, `tabular_lining`).
    "#,
    MetricValue::String("proportional_oldstyle".to_string())
);
quantifier!(
    SINF,
    "numerals/inferior_numerals",
    "Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%). _A partial support is useless in practice._",
    MetricValue::Percentage(1.0)
);
quantifier!(
    SUPS,
    "numerals/superior_numerals",
    "Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%). _A partial support is useless in practice._",
    MetricValue::Percentage(1.0)
);
quantifier!(
    ENCODED_FRACTIONS_CHECK,
    "numerals/encoded_fractions",
    "Returns percentage of encoded default fractions (e.g. 1/2) that are shaped by the `frac` feature.",
    MetricValue::Percentage(1.0)
);
quantifier!(
    EXTENDED_FRACTIONS,
    "numerals/arbitrary_fractions",
    "Returns a boolean of whether or not arbitrary fractions (e.g. 12/99) can be shaped by the `frac` feature.",
    MetricValue::Boolean(true)
);
quantifier!(
    SLASHED_ZERO,
    "numerals/slashed_zero",
    "Returns the percentage of tested feature combinations where `zero` changes the shaping result.",
    MetricValue::Percentage(1.0)
);
fn ton_matrix(font: &FontRef) -> bool {
    shapes_differently_with_features(font, "0123456789", &tags_from_matrix(TON_MATRIX))
}
fn pon_matrix(font: &FontRef) -> bool {
    shapes_differently_with_features(font, "0123456789", &tags_from_matrix(PON_MATRIX))
}
fn pln_matrix(font: &FontRef) -> bool {
    shapes_differently_with_features(font, "0123456789", &tags_from_matrix(PLN_MATRIX))
}
fn tln_matrix(font: &FontRef) -> bool {
    shapes_differently_with_features(font, "0123456789", &tags_from_matrix(TLN_MATRIX))
}

pub(crate) fn get_numeral_styles(
    font: &FontRef,
    _location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let frac = Tag::new(b"frac");
    let arbitrary_fractions = ARBITRARY_FRACTIONS
        .iter()
        .all(|string| shapes_differently_with_features(font, string, &[frac]));
    let encoded_fractions_ratio = ENCODED_FRACTIONS
        .iter()
        .filter(|string| shapes_differently_with_features(font, string, &[frac]))
        .count() as f64
        / ENCODED_FRACTIONS.len() as f64;
    let mut slashed_zero_checks: Vec<(&str, Vec<Tag>, Vec<Tag>)> = vec![
        // Basic: does `zero` change the default shaping of "0"?
        ("0", vec![], vec![Tag::new(b"zero")]),
    ];
    // sups: does adding `zero` on top of `sups` change the shaping of "0"?
    if shapes_differently_with_features(font, "0", &[Tag::new(b"sups")]) {
        slashed_zero_checks.push((
            "0",
            vec![Tag::new(b"sups")],
            vec![Tag::new(b"zero"), Tag::new(b"sups")],
        ));
    }
    // sinf: does adding `zero` on top of `sinf` change the shaping of "0"?
    if shapes_differently_with_features(font, "0", &[Tag::new(b"sinf")]) {
        slashed_zero_checks.push((
            "0",
            vec![Tag::new(b"sinf")],
            vec![Tag::new(b"zero"), Tag::new(b"sinf")],
        ));
    }
    // frac: does adding `zero` on top of `frac` change "0/1" / "1/0"?
    if arbitrary_fractions {
        slashed_zero_checks.push(("0/1", vec![frac], vec![Tag::new(b"zero"), frac]));
        slashed_zero_checks.push(("1/0", vec![frac], vec![Tag::new(b"zero"), frac]));
    }
    let slashed_zero_ratio = slashed_zero_checks
        .iter()
        .filter(|(string, base, with_zero)| {
            shapes_differently_between(font, string, base, with_zero)
        })
        .count() as f64
        / slashed_zero_checks.len() as f64;

    let default = default_numerals(font);
    results.add_metric(&DEFAULT_NUMERALS, MetricValue::String(default.to_string()));
    results.add_metric(
        &TON,
        MetricValue::Boolean(
            (ton_matrix(font)
                && numeral_style_heuristics(font, &tags_from_matrix(TON_MATRIX))? == TON_LABEL)
                || default == TON_LABEL,
        ),
    );
    results.add_metric(
        &PON,
        MetricValue::Boolean(
            (pon_matrix(font)
                && numeral_style_heuristics(font, &tags_from_matrix(PON_MATRIX))? == PON_LABEL)
                || default == PON_LABEL,
        ),
    );
    results.add_metric(
        &PLN,
        MetricValue::Boolean(
            (pln_matrix(font)
                && numeral_style_heuristics(font, &tags_from_matrix(PLN_MATRIX))? == PLN_LABEL)
                || default == PLN_LABEL,
        ),
    );
    results.add_metric(
        &TLN,
        MetricValue::Boolean(
            (tln_matrix(font)
                && numeral_style_heuristics(font, &tags_from_matrix(TLN_MATRIX))? == TLN_LABEL)
                || default == TLN_LABEL,
        ),
    );
    results.add_metric(
        &SINF,
        MetricValue::Percentage(ratio_of_different_shapes(
            font,
            |c| c.is_ascii_digit(),
            harfrust::Tag::new(b"sinf"),
        )),
    );
    results.add_metric(
        &SUPS,
        MetricValue::Percentage(ratio_of_different_shapes(
            font,
            |c| c.is_ascii_digit(),
            harfrust::Tag::new(b"sups"),
        )),
    );
    results.add_metric(
        &ENCODED_FRACTIONS_CHECK,
        MetricValue::Percentage(encoded_fractions_ratio),
    );
    results.add_metric(
        &EXTENDED_FRACTIONS,
        MetricValue::Boolean(arbitrary_fractions),
    );
    results.add_metric(&SLASHED_ZERO, MetricValue::Percentage(slashed_zero_ratio));
    Ok(())
}

fn default_numerals(font: &FontRef) -> &'static str {
    let mut numeralsets = HashSet::from([PON_LABEL, TON_LABEL, PLN_LABEL, TLN_LABEL]);
    if pon_matrix(font) {
        numeralsets.remove(PON_LABEL);
    }
    if ton_matrix(font) {
        numeralsets.remove(TON_LABEL);
    }
    if pln_matrix(font) {
        numeralsets.remove(PLN_LABEL);
    }
    if tln_matrix(font) {
        numeralsets.remove(TLN_LABEL);
    }
    if numeralsets.len() == 1 {
        #[allow(clippy::unwrap_used)]
        numeralsets.into_iter().next().unwrap()
    } else {
        numeral_style_heuristics(font, &[]).unwrap_or("unknown")
    }
}
