use fontations::skrifa;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontquantError {
    #[error("skrifa could not parse the font: {0}")]
    SkrifaParse(#[from] skrifa::raw::ReadError),
    #[error("skrifa could not draw the font: {0}")]
    SkrifaDraw(#[from] skrifa::outline::DrawError),
    #[error("skia could not simplify a glyph")]
    SkiaError,
}
