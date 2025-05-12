use crate::MetricGatherer;

pub mod appearance;

pub type QuantifierFn = fn(
    &skrifa::FontRef,
    &[skrifa::setting::VariationSetting],
    &mut crate::Results,
) -> Result<(), crate::FontquantError>;

pub const ALL_QUANTIFIERS: &[QuantifierFn] = &[
    appearance::WholeFontStatistics::gather_from_font,
    appearance::is_stencil_font,
];
