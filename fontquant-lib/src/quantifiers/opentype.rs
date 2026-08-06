use read_fonts::TableProvider;
use skrifa::{FontRef, Tag, setting::VariationSetting};

use crate::{MetricValue, error::FontquantError, quantifier};

pub(crate) fn get_fields(
    font: &FontRef,
    location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let os2 = font.os2()?;
    let weight_class = if let Some(wght) = location.iter().find(|s| s.selector == Tag::new(b"wght"))
    {
        wght.value as i32
    } else {
        os2.us_weight_class() as i32
    };
    let width_class = if let Some(wdth) = location.iter().find(|s| s.selector == Tag::new(b"wdth"))
    {
        wdth.value as i32
    } else {
        os2.us_width_class() as i32
    };

    results.add_metric(
        &crate::quantifiers::opentype::WEIGHT_CLASS,
        MetricValue::Integer(weight_class),
    );
    results.add_metric(
        &crate::quantifiers::opentype::WIDTH_CLASS,
        MetricValue::Integer(width_class),
    );
    Ok(())
}

quantifier!(
    WEIGHT_CLASS,
    "opentype/os2_weight_class",
    r#"Returns the weight class of the font as specified in the OS/2 table."#,
    MetricValue::Integer(400)
);

quantifier!(
    WIDTH_CLASS,
    "opentype/os2_width_class",
    r#"Returns the width class of the font as specified in the OS/2 table."#,
    MetricValue::Integer(5)
);
