use std::collections::HashMap;

use fontations::skrifa;
use fontations::skrifa::{setting::VariationSetting, FontRef, GlyphId, MetadataProvider};
use unicode_script::UnicodeScript;

use crate::{
    bezglyph::{BezGlyph, ScalerPen},
    error::FontquantError,
};

pub(crate) trait PrimaryScript {
    fn primary_script(&self) -> String;
    fn glyphs_for_primary_script(&self) -> impl Iterator<Item = GlyphId>;
}

impl PrimaryScript for FontRef<'_> {
    fn primary_script(&self) -> String {
        let mut script_count = HashMap::new();
        let codepoints = self.charmap().mappings().map(|(u, _gid)| u);
        for c in codepoints.filter_map(char::from_u32) {
            for script in c.script_extension().iter() {
                let name = script.short_name();
                if !["Zinh", "Zyyy", "Zzzz"].contains(&name) {
                    *script_count.entry(name).or_insert(0) += 1;
                }
            }
        }
        let most_common = script_count.iter().max_by_key(|(_, count)| **count);
        if let Some((script, _)) = most_common {
            script.to_string()
        } else {
            "Latn".to_string()
        }
    }

    fn glyphs_for_primary_script(&self) -> impl Iterator<Item = GlyphId> {
        let primary_script = self.primary_script();
        self.charmap()
            .mappings()
            .flat_map(|(cp, gid)| char::from_u32(cp).map(|c| (c, gid)))
            .filter_map(move |(c, gid)| {
                c.script_extension()
                    .iter()
                    .any(|s| s.short_name() == primary_script)
                    .then_some(gid)
            })
    }
}

pub(crate) trait MakeBezGlyphs {
    fn bezglyph_for_char(
        &self,
        location: &[VariationSetting],
        scale: Option<f32>,
        c: char,
    ) -> Result<Option<BezGlyph>, FontquantError>;
    fn bezglyph_for_gid(
        &self,
        location: &[VariationSetting],
        scale: Option<f32>,
        gid: GlyphId,
    ) -> Result<Option<BezGlyph>, FontquantError>;
}
impl MakeBezGlyphs for FontRef<'_> {
    fn bezglyph_for_char(
        &self,
        location: &[VariationSetting],
        scale: Option<f32>,
        c: char,
    ) -> Result<Option<BezGlyph>, FontquantError> {
        let Some(glyph_id) = self.charmap().map(c as u32) else {
            return Ok(None);
        };
        self.bezglyph_for_gid(location, scale, glyph_id)
    }
    fn bezglyph_for_gid(
        &self,
        location: &[VariationSetting],
        scale: Option<f32>,
        glyph_id: GlyphId,
    ) -> Result<Option<BezGlyph>, FontquantError> {
        let collection = self.outline_glyphs();
        let loc = self.axes().location(location);

        let settings =
            skrifa::outline::DrawSettings::unhinted(skrifa::prelude::Size::unscaled(), &loc);
        let outlined = collection.get(glyph_id).ok_or(FontquantError::SkrifaDraw(
            skrifa::outline::DrawError::GlyphNotFound(glyph_id),
        ))?;
        let mut bezglyph = BezGlyph::default();
        if let Some(scale) = scale {
            let mut scaled_bezglyph = ScalerPen::new(&mut bezglyph, scale);
            outlined.draw(settings, &mut scaled_bezglyph)?;
        } else {
            outlined.draw(settings, &mut bezglyph)?;
        }
        Ok(Some(bezglyph))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyphs_for_primary_script() {
        #[allow(clippy::unwrap_used)] // it's a test suite
        let fontref = skrifa::FontRef::new(include_bytes!(
            "../../tests/fonts/AllertaStencil-Regular.ttf"
        ))
        .unwrap();
        assert_eq!(fontref.primary_script(), "Latn");
        let glyphs: Vec<_> = fontref.glyphs_for_primary_script().collect();
        assert_eq!(glyphs.len(), 120);
    }
}
