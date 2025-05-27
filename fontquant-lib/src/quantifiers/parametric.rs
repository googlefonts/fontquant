use std::sync::LazyLock;

use fontations::{read::TableProvider, skrifa};

use crate::{
    helpers::raycaster::{Direction, ProportionalPoint, Raycaster, Winding, EAST, NORTH},
    monkeypatching::MakeBezGlyphs,
    quantifier, MetricKey, MetricValue,
};

struct RaycasterBuilder<'a> {
    metric: &'a LazyLock<MetricKey>,
    glyph: char,
    start: ProportionalPoint,
    direction: Direction,
    specializer: fn(&mut Raycaster),
}

// We put them here so we can use them in the tests as well as in the main code
fn casters() -> [RaycasterBuilder<'static>; 9] {
    [
        RaycasterBuilder {
            metric: &XOPQ,
            glyph: 'H',
            start: ProportionalPoint::new(0.0, 0.5),
            direction: EAST,
            specializer: |caster| {
                caster.winding(Winding::Ink).jitter(0.2, 12);
            },
        },
        RaycasterBuilder {
            metric: &XOLC,
            glyph: 'n',
            start: ProportionalPoint::new(0.0, 0.4),
            direction: EAST,
            specializer: |caster| {
                caster.winding(Winding::Ink).jitter(0.2, 12);
            },
        },
        RaycasterBuilder {
            metric: &XOFI,
            glyph: '1',
            start: ProportionalPoint::new(0.0, 0.5),
            direction: EAST,
            specializer: |caster| {
                caster.winding(Winding::Ink);
            },
        },
        RaycasterBuilder {
            metric: &XTRA,
            glyph: 'H',
            start: ProportionalPoint::new(0.0, 0.5),
            direction: EAST,
            specializer: |caster| {
                caster.winding(Winding::Transparent);
            },
        },
        RaycasterBuilder {
            metric: &XTLC,
            glyph: 'n',
            start: ProportionalPoint::new(0.0, 0.4),
            direction: EAST,
            specializer: |caster| {
                caster.winding(Winding::Transparent);
            },
        },
        RaycasterBuilder {
            metric: &XTFI,
            glyph: '0',
            start: ProportionalPoint::new(0.0, 0.5),
            direction: EAST,
            specializer: |caster| {
                caster.jitter(0.05, 10);
                caster.winding(Winding::Transparent);
            },
        },
        RaycasterBuilder {
            metric: &YOPQ,
            glyph: 'H',
            start: ProportionalPoint::new(0.5, 0.25),
            direction: NORTH,
            specializer: |caster| {
                caster.jitter(0.1, 10);
                caster.winding(Winding::Ink);
            },
        },
        RaycasterBuilder {
            metric: &YOLC,
            glyph: 'f',
            start: ProportionalPoint::new(0.75, 0.5),
            direction: Direction::EndPoint(ProportionalPoint::new(0.75, 0.9)),
            specializer: |caster| {
                caster.jitter(0.1, 6);
                caster.winding(Winding::Ink);
            },
        },
        RaycasterBuilder {
            metric: &YOFI,
            glyph: '0',
            start: ProportionalPoint::new(0.5, 0.0),
            direction: NORTH,
            specializer: |caster| {
                caster.jitter(0.05, 12);
                caster.winding(Winding::Ink);
            },
        },
    ]
}
pub fn get_parametric(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), crate::FontquantError> {
    let upem = font.head()?.units_per_em();

    for caster in casters() {
        if let Some(glyph) = font.bezglyph_for_char(location, None, caster.glyph)? {
            let glyph = glyph.remove_overlaps()?;
            let mut raycaster = Raycaster::new(&glyph, caster.start, caster.direction);
            (caster.specializer)(&mut raycaster);
            let xopq = raycaster.median_pair_distance(true).round();
            // Upem scale
            results.add_metric(
                caster.metric,
                MetricValue::PerMille((xopq / upem as f64 * 1000.0).round()),
            );
        }
    }
    Ok(())
}

quantifier!(
    XOPQ,
    "parametric/XOPQ",
    "The font's X Opaque parametric value",
    MetricValue::PerMille(120.0)
);

quantifier!(
    XOLC,
    "parametric/XOLC",
    "The font's X Opaque lowercase parametric value",
    MetricValue::PerMille(120.0)
);

quantifier!(
    XOFI,
    "parametric/XOFI",
    "The font's X Opaque figures parametric value",
    MetricValue::PerMille(120.0)
);

quantifier!(
    XTRA,
    "parametric/XTRA",
    "The font's X Transparent parametric value",
    MetricValue::PerMille(120.0)
);

quantifier!(
    XTLC,
    "parametric/XTLC",
    "The font's X Transparent lowercase parametric value",
    MetricValue::PerMille(120.0)
);

quantifier!(
    XTFI,
    "parametric/XTFI",
    "The font's X Transparent figures parametric value",
    MetricValue::PerMille(120.0)
);
quantifier!(
    YOPQ,
    "parametric/YOPQ",
    "The font's Y Opaque parametric value",
    MetricValue::PerMille(120.0)
);
quantifier!(
    YOLC,
    "parametric/YOLC",
    "The font's Y Opaque lowercase parametric value",
    MetricValue::PerMille(120.0)
);
quantifier!(
    YOFI,
    "parametric/YOFI",
    "The font's Y Opaque lowercase figure value",
    MetricValue::PerMille(120.0)
);

#[cfg(test)]
mod tests {
    use fontations::skrifa;

    use crate::monkeypatching::MakeBezGlyphs;
    use std::{collections::HashMap, io::Write};

    use super::*;

    #[allow(clippy::expect_used, clippy::unwrap_used)]
    #[test]
    fn test_roboto() {
        let font_binary = include_bytes!("../../../tests/fonts/RobotoFlex-Var.ttf");
        let font = skrifa::FontRef::new(font_binary).unwrap();
        // These values are *uncorrected* for upem
        let expectations: HashMap<&'static str, f64> = [
            ("XOPQ", 192.0),
            ("XOLC", 186.0),
            ("XOFI", 192.0),
            ("XTRA", 734.0),
            ("XTLC", 480.0),
            ("XTFI", 548.0),
            ("YOPQ", 158.0),
            ("YOLC", 142.0),
            ("YOFI", 159.0),
        ]
        .into_iter()
        .collect();
        for builder in casters() {
            let glyph = font
                .bezglyph_for_char(&[], None, builder.glyph)
                .expect("Failed to get glyph")
                .expect("Glyph not found");
            let glyph = glyph.remove_overlaps().expect("Failed to remove overlaps");

            let mut raycaster = Raycaster::new(&glyph, builder.start, builder.direction);
            (builder.specializer)(&mut raycaster);
            let data = raycaster.draw();
            let mut file = std::fs::File::create(builder.metric.name.clone() + ".png").unwrap();
            file.write_all(data.as_bytes()).unwrap();
            let expected = expectations.get(builder.metric.name.as_str()).unwrap();
            let found = raycaster.median_pair_distance(true).round();
            assert_eq!(
                found, *expected,
                "{} should be {}, we found {}",
                builder.metric.name, expected, found
            );
        }
    }
}
