use crate::MetricGatherer;
use fontations::skrifa;

pub mod appearance;
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
];
