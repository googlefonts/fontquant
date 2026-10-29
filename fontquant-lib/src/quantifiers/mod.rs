use std::sync::LazyLock;

use skrifa;

use crate::{
    quantifiers::{
        appearance::{
            metrics::{
                ASCENDER, CAP_HEIGHT, DESCENDER, I_WIDTH, MONOSPACED, MOST_COMMON_WIDTH, N_WIDTH,
                PROPORTION_LOWERCASE, PROPORTION_UPPERCASE, SPACE_WIDTH, X_HEIGHT,
            },
            stats::{SLANT, WEIGHT, WEIGHT_PERCEPTUAL, WIDTH},
            stencil::STENCIL,
            storys::{LOWERCASE_A_STYLE, LOWERCASE_G_STYLE},
            strokecontrast::{
                STROKE_CONTRAST_ANTIQUA, STROKE_CONTRAST_ANTIQUA_ANGLE, STROKE_CONTRAST_RAYCASTER,
            },
        },
        casing::{C2SC, CASE, LOWERCASE_SHAPES, SMCP, UNICASE},
        features::{FEATURE_LIST, FEATURE_STYLISTIC_SETS},
        numerals::{
            DEFAULT_NUMERALS, ENCODED_FRACTIONS_CHECK, EXTENDED_FRACTIONS, PLN, PON, SINF,
            SLASHED_ZERO, SUPS, TLN, TON,
        },
        opentype::{WEIGHT_CLASS, WIDTH_CLASS},
        parametric::*,
        structure::{loops::LOOPS, waist::WAIST},
    },
    MetricKey,
};
pub mod appearance;
pub mod casing;
pub mod features;
pub mod numerals;
pub mod opentype;
pub mod parametric;
pub mod structure;

pub type QuantifierFn = fn(
    &skrifa::FontRef,
    &[skrifa::setting::VariationSetting],
    &mut crate::Results,
) -> Result<(), crate::FontquantError>;

macro_rules! all_quantifiers {
    ($(quantifier $f:path => [$($quantifier:expr ),+])+) => {
        pub const ALL_QUANTIFIERS: &[(&LazyLock<MetricKey>, QuantifierFn)] = &[
            $(
                $(
                    (&$quantifier, $f),
                )+
            )+
        ];
    }
}
all_quantifiers!(
    quantifier appearance::WholeFontStatistics::gather_from_font => [WEIGHT, WEIGHT_PERCEPTUAL, WIDTH, SLANT]
    quantifier parametric::get_parametric => [XCLR, XCLS, XOFI, XOFI, XOLC, XOPQ, XTFI, XTLC, XTRA, YOFI, YOFI, YOLC, YOPQ, YTAS, YTDE]
    quantifier appearance::is_stencil_font => [STENCIL]
    quantifier appearance::get_stroke_contrast => [STROKE_CONTRAST_ANTIQUA, STROKE_CONTRAST_ANTIQUA_ANGLE, STROKE_CONTRAST_RAYCASTER]
    quantifier appearance::metrics::gather_from_font => [X_HEIGHT, CAP_HEIGHT, ASCENDER, DESCENDER, N_WIDTH, SPACE_WIDTH, I_WIDTH, MOST_COMMON_WIDTH, MONOSPACED, PROPORTION_LOWERCASE, PROPORTION_UPPERCASE]
    quantifier casing::is_unicase => [UNICASE]
    quantifier appearance::storys::check_lowercase_a_style => [LOWERCASE_A_STYLE]
    quantifier appearance::storys::check_lowercase_g_style => [LOWERCASE_G_STYLE]
    quantifier casing::test_casing => [SMCP, C2SC, CASE]
    quantifier casing::get_lowercase_shapes => [LOWERCASE_SHAPES]
    quantifier numerals::get_numeral_styles => [DEFAULT_NUMERALS, TON, PON, PLN, TLN, SINF, SUPS, ENCODED_FRACTIONS_CHECK, EXTENDED_FRACTIONS, SLASHED_ZERO]
    quantifier features::gather_features => [FEATURE_LIST, FEATURE_STYLISTIC_SETS]
    quantifier opentype::get_fields => [WEIGHT_CLASS, WIDTH_CLASS]
    quantifier structure::loops::check_loop_style => [LOOPS]
    quantifier structure::waist::measure_waist => [WAIST]
);
