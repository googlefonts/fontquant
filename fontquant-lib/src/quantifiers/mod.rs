use skrifa;
pub mod appearance;
pub mod casing;
pub mod features;
pub mod numerals;
pub mod opentype;
pub mod parametric;

pub type QuantifierFn = fn(
    &skrifa::FontRef,
    &[skrifa::setting::VariationSetting],
    &mut crate::Results,
) -> Result<(), crate::FontquantError>;

pub const ALL_QUANTIFIERS: &[QuantifierFn] = &[
    appearance::WholeFontStatistics::gather_from_font,
    appearance::is_stencil_font,
    parametric::get_parametric,
    appearance::get_stroke_contrast,
    appearance::metrics::gather_from_font,
    casing::is_unicase,
    appearance::storys::check_lowercase_a_style, // Needs stencil and unicase first
    appearance::storys::check_lowercase_g_style, // Needs stencil and unicase first
    casing::test_casing,
    casing::get_lowercase_shapes,
    numerals::get_numeral_styles,
    features::gather_features,
    opentype::get_fields,
];
