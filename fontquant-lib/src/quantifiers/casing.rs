use crate::{
    MetricValue, error::FontquantError, helpers::shaping::ratio_of_different_shapes,
    monkeypatching::MakeBezGlyphs, quantifier,
};
use std::collections::HashMap;

use read_fonts::TableProvider as _;
use skrifa::{FontRef, GlyphId, MetadataProvider, Tag, setting::VariationSetting};
use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategory, GeneralCategoryGroup, UnicodeGeneralCategory};

const EXCLUDE_UPPERCASE: [char; 3] = ['Q', 'J', 'Ŋ'];
const EXCLUDE_LOWERCASE: [char; 3] = ['μ', 'ŋ', 'ƒ'];
const EXCEPTIONS_C2SC: [char; 1] = ['Ω'];
const EXCEPTIONS_SMCP: [char; 5] = ['µ', 'ℓ', 'ᾳ', 'ƒ', 'μ'];

quantifier!(
    UNICASE,
    "casing/unicase",
    r#"Reports whether or not a font is unicase (lowercase and uppercase letters being of the same height).
    To check for different shapes of lowercase letters compared to uppercase, use the `Lowercase Shapes` metric."#,
    MetricValue::Boolean(false)
);
pub fn is_unicase(
    font: &FontRef,
    location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let codepoints: HashMap<u32, GlyphId> = font.charmap().mappings().collect();
    let upem = font.head()?.units_per_em() as f64;
    let height_threshold = upem * 0.1;
    let mut lowest_list = vec![];
    let mut highest_list = vec![];
    for unicode in codepoints.keys() {
        let Some(c) = char::from_u32(*unicode) else {
            continue;
        };
        if c.general_category() == GeneralCategory::UppercaseLetter
            && !EXCLUDE_UPPERCASE.contains(&c)
            && String::from(c).nfd().count() == 1
            && let Some(bbox) = font
                .bezglyph_for_char(location, None, c)?
                .and_then(|bez| bez.bbox())
        {
            lowest_list.push(bbox.min_y());
            highest_list.push(bbox.max_y());
        }
    }
    if lowest_list.is_empty() || highest_list.is_empty() {
        return Ok(());
    }
    let highest_average = highest_list.iter().sum::<f64>() / highest_list.len() as f64;
    let lowest_average = lowest_list.iter().sum::<f64>() / lowest_list.len() as f64;
    let mut unicase_count = 0;
    let mut char_count = 0;

    for unicode in codepoints.keys() {
        let Some(c) = char::from_u32(*unicode) else {
            continue;
        };
        if matches!(
            c.general_category(),
            GeneralCategory::UppercaseLetter | GeneralCategory::LowercaseLetter
        ) && !EXCLUDE_UPPERCASE.contains(&c)
            && !EXCLUDE_LOWERCASE.contains(&c)
            && String::from(c).nfd().count() == 1
            && let Some(bbox) = font
                .bezglyph_for_char(location, None, c)?
                .and_then(|bez| bez.bbox())
        {
            char_count += 1;
            if (bbox.max_y() - highest_average).abs() < height_threshold
                && (lowest_average - bbox.min_y()).abs() < height_threshold
            {
                unicase_count += 1;
            }
        }
    }

    results.add_metric(
        &UNICASE,
        MetricValue::Boolean(unicase_count as f64 / char_count as f64 > 0.95),
    );

    Ok(())
}

quantifier!(
    SMCP,
    "casing/smallcaps",
    r#"Consider fonts to have a functioning `smcp` feature if the value is above `0.95` (95%),
    as there are some characters that are lowercase letters but don't typically get shaped by the `smcp` feature,
    e.g. `µ`.
    Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.rs` to see your
    values rise."#,
    MetricValue::Percentage(0.5)
);

quantifier!(
    C2SC,
    "casing/caps-to-smallcaps",
    r#"Consider fonts to have a functioning `c2sc` feature if the value is above `0.95` (95%),
    as there are some characters that are uppercase letters but don't typically get shaped by the `c2sc` feature,
    e.g. `Ohm`.
    Alternatively, consider contributing exceptions to the `exceptions_c2sc` variable in `casing.rs` to see your
    values rise."#,
    MetricValue::Percentage(0.5)
);

quantifier!(
    CASE,
    "casing/case_sensitive_punctuation",
    r#"Returns the percentage of characters that are punctuation (`P*`)
    and get shaped by the `case` feature."#,
    MetricValue::Percentage(0.5)
);

pub(crate) fn test_casing(
    font: &FontRef,
    _location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let smcp_ratio = ratio_of_different_shapes(
        font,
        |c| c.is_lowercase() && !EXCEPTIONS_SMCP.contains(&c),
        Tag::new(b"smcp"),
    );
    let c2sc_ratio = ratio_of_different_shapes(
        font,
        |c| c.is_uppercase() && !EXCEPTIONS_C2SC.contains(&c),
        Tag::new(b"c2sc"),
    );
    let case_ratio = ratio_of_different_shapes(
        font,
        |c| c.general_category_group() == GeneralCategoryGroup::Punctuation,
        Tag::new(b"case"),
    );

    // We store percentages as 0-1 and display as 0-100.
    results.add_metric(&SMCP, MetricValue::Percentage(smcp_ratio));
    results.add_metric(&C2SC, MetricValue::Percentage(c2sc_ratio));
    results.add_metric(&CASE, MetricValue::Percentage(case_ratio));

    Ok(())
}

quantifier!(
    LOWERCASE_SHAPES,
    "casing/lowercase_shapes",
    r#"Returns the shapes of lowercase-codepoint characters.
    Possible values are `uppercase`, `lowercase`, and `smallcaps`.
    This check compares the contour count (and the average height) of uppercase and lowercase letters,
    so it compares actual outline construction. In that sense it's different from the `Unicase` metric which only looks
    at dimensions and allows upper/lowercase shapes to be different as long as they are of similar height."#,
    MetricValue::String("lowercase".to_string())
);

pub(crate) fn get_lowercase_shapes(
    font: &FontRef,
    location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let mut uc_upperbounds = vec![];
    let mut uc_lowerbounds = vec![];
    let mut lc_upperbounds = vec![];
    let mut lc_lowerbounds = vec![];
    let mut uc_contours_in_lowercase = vec![];
    for char in [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ] {
        let Some(bezglyph_uc) = font.bezglyph_for_char(location, None, char)? else {
            continue;
        };
        let bezglyph_uc = bezglyph_uc.remove_overlaps()?;
        #[allow(clippy::unwrap_used)] // Uppercase letters definitely have a lowercase variant
        let Some(bezglyph_lc) =
            font.bezglyph_for_char(location, None, char.to_lowercase().next().unwrap())?
        else {
            continue;
        };
        let bezglyph_lc = bezglyph_lc.remove_overlaps()?;
        if bezglyph_uc.iter().count() == bezglyph_lc.iter().count() {
            uc_contours_in_lowercase.push(1);
        } else {
            uc_contours_in_lowercase.push(0);
        }
        if bezglyph_uc.iter().count() == 0 || bezglyph_lc.iter().count() == 0 {
            continue;
        }
        let uc_upperbound = bezglyph_uc.bbox().map(|bbox| bbox.max_y()).unwrap_or(0.0);
        let uc_lowerbound = bezglyph_uc.bbox().map(|bbox| bbox.min_y()).unwrap_or(0.0);
        let lc_upperbound = bezglyph_lc.bbox().map(|bbox| bbox.max_y()).unwrap_or(0.0);
        let lc_lowerbound = bezglyph_lc.bbox().map(|bbox| bbox.min_y()).unwrap_or(0.0);
        uc_upperbounds.push(uc_upperbound);
        uc_lowerbounds.push(uc_lowerbound);
        lc_upperbounds.push(lc_upperbound);
        lc_lowerbounds.push(lc_lowerbound);
    }

    // 2.0* was in original Python, not convinced by it
    let uc_average_height = (uc_upperbounds.iter().sum::<f64>()
        - uc_lowerbounds.iter().sum::<f64>())
        / (2.0 * uc_upperbounds.len() as f64);
    let lc_average_height = (lc_upperbounds.iter().sum::<f64>()
        - lc_lowerbounds.iter().sum::<f64>())
        / (2.0 * lc_upperbounds.len() as f64);
    let lc_to_uc_height_ratio = lc_average_height / uc_average_height;
    let uc_lc_outline_fidelity =
        uc_contours_in_lowercase.iter().sum::<i32>() as f64 / uc_contours_in_lowercase.len() as f64;

    let answer = if uc_lc_outline_fidelity > 0.9 {
        if lc_to_uc_height_ratio < 0.8 {
            "smallcaps"
        } else {
            "uppercase"
        }
    } else {
        "lowercase"
    };

    results.add_metric(&LOWERCASE_SHAPES, MetricValue::String(answer.to_string()));

    Ok(())
}
