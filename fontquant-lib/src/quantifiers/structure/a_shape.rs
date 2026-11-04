//! The shape of the counter of a double-storey `a`.
//!
//! The bowl of a double-storey `a` encloses a counter, and the way that counter
//! is finished at its top right corner is one of the things that gives a
//! typeface its character. The corner is where the underside of the arch — the
//! stroke joining the bowl to the stem — meets the stem, and the four shapes
//! that type designers distinguish are the four things that stroke can do:
//!
//! * **convex**: it comes down from above, curving into the stem, so the
//!   counter's top edge is descending as it reaches the corner;
//! * **concave**: it sweeps up from below, so the counter comes to a point;
//! * **flat**: it runs horizontally into the stem;
//! * **angled**: it runs into the stem as a straight diagonal.
//!
//! # How it works
//!
//! Only the counter is measured, so the glyph must have one. The bowl's counter
//! is the lowest counter of the `a` — the one wound the other way from the
//! outermost contour — and the glyph must look like a double-storey `a`, which
//! is to say its counter must stop well below the top of the glyph, leaving
//! room for the arch above it. A single-storey bowl fills most of the height.
//!
//! The counter is then sampled evenly by distance and the top right corner is
//! taken to be the point furthest along the diagonal. Looking back from there
//! along the counter's top edge gives the direction and the curvature of the
//! stroke that arrives at the corner, which is what the four shapes are made
//! of.
//!
//! Fonts that do something else — a detached bowl, an open counter, a
//! single-storey `a` — get no classification.

use kurbo::{Affine, BezPath, Point};
use read_fonts::TableProvider as _;

use crate::{MetricValue, error::FontquantError, monkeypatching::MakeBezGlyphs as _, quantifier};

use super::outline::{deslant_shear, flatten, resample, slant_of};

/// The probe glyph.
const GLYPH: char = 'a';

/// How many points the counter is resampled to.
const SAMPLE_POINTS: usize = 600;

/// How far back along the counter's top edge the corner is measured, as a
/// fraction of the counter's width.
const SPAN: f64 = 0.1;

/// How many points from the corner are used to decide which way the top edge
/// runs.
const CORNER_PROBE: usize = 8;

/// The top of the counter must sit below this fraction of the glyph's height
/// for the glyph to read as a double-storey `a`. A single-storey bowl reaches
/// most of the way up.
const DOUBLE_STOREY_TOP: f64 = 0.7;

/// Arriving at the corner more steeply downwards than this makes the counter
/// convex, and more steeply upwards than [`CONCAVE_DEGREES`] makes it concave.
const CONVEX_DEGREES: f64 = -3.0;
const CONCAVE_DEGREES: f64 = 19.0;

/// Within this many degrees of horizontal the corner is flat — unless the
/// counter's top edge as a whole climbs more steeply than [`STEEP_SLOPE`], in
/// which case the flatness is just the rounding of an angled corner.
const FLAT_DEGREES: f64 = 3.0;
const STEEP_SLOPE: f64 = 0.35;

/// The shape of the counter at the top right corner of the bowl of a
/// double-storey `a`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AShape {
    /// The arch comes down from above and curves into the stem.
    Convex,
    /// The arch sweeps up from below, so the counter comes to a point.
    Concave,
    /// The arch runs horizontally into the stem.
    Flat,
    /// The arch runs into the stem as a straight diagonal.
    Angled,
}

impl std::fmt::Display for AShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AShape::Convex => "convex",
            AShape::Concave => "concave",
            AShape::Flat => "flat",
            AShape::Angled => "angled",
        })
    }
}

/// How the top edge of the counter arrives at its top right corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AFeatures {
    /// Direction of the stroke arriving at the corner, in degrees away from
    /// horizontal. Negative is descending, positive is ascending.
    pub arrival: f64,
    /// How far the arriving stroke bows away from a straight line, positive
    /// when it lies above that line.
    pub bow: f64,
    /// Slope of the counter's top edge across the width of the counter.
    pub slope: f64,
}

/// The counter of the bowl of a double-storey `a`, as evenly spaced points in
/// deslanted coordinates.
pub fn bowl_counter(paths: &[&BezPath], shear: Affine, upem: f64) -> Option<Vec<Point>> {
    let outline = paths
        .iter()
        .map(|path| {
            let mut points = flatten(&(shear * *path), upem / 4000.0);
            if let (Some(first), Some(last)) = (points.first().copied(), points.last().copied())
                && first != last
            {
                points.push(first);
            }
            points
        })
        .filter(|points| points.len() > 2)
        .collect::<Vec<_>>();
    if outline.len() < 2 {
        return None;
    }

    let areas = outline
        .iter()
        .map(|points| signed_area(points))
        .collect::<Vec<_>>();
    let outer = areas
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .map(|(index, _)| index)?;
    let winding = areas[outer].signum();
    let mut counters = outline
        .iter()
        .zip(&areas)
        .enumerate()
        .filter(|(index, (_, area))| *index != outer && area.signum() != winding)
        .collect::<Vec<_>>();
    if counters.is_empty() {
        return None;
    }
    counters.sort_by(|a, b| {
        let low = |points: &Vec<Point>| {
            points
                .iter()
                .map(|point| point.y)
                .fold(f64::INFINITY, f64::min)
        };
        low(a.1.0).total_cmp(&low(b.1.0))
    });

    let glyph = &outline[outer];
    let glyph_bottom = glyph
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min);
    let glyph_top = glyph
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let height = glyph_top - glyph_bottom;
    if height <= 0.0 {
        return None;
    }
    let counter = counters[0].1.0;
    let counter_top = counter
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);
    // A single-storey bowl fills the glyph; the bowl of a double-storey `a`
    // stops low enough to leave the arch above it.
    if (counter_top - glyph_bottom) / height > DOUBLE_STOREY_TOP {
        return None;
    }
    Some(resample(counter, SAMPLE_POINTS))
}

fn signed_area(points: &[Point]) -> f64 {
    let mut area = 0.0;
    for (index, point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        area += point.x * next.y - next.x * point.y;
    }
    area / 2.0
}

/// Measure how the counter's top edge arrives at its top right corner.
pub fn a_features(counter: &[Point]) -> Option<AFeatures> {
    if counter.len() < 8 {
        return None;
    }
    let left = counter
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let right = counter
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let bottom = counter
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min);
    let top = counter
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let width = right - left;
    let height = top - bottom;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let normalised = counter
        .iter()
        .map(|point| Point::new((point.x - left) / width, (point.y - bottom) / height))
        .collect::<Vec<_>>();

    let corner = normalised
        .iter()
        .enumerate()
        .max_by(|a, b| (a.1.x + a.1.y).total_cmp(&(b.1.x + b.1.y)))
        .map(|(index, _)| index)?;
    let count = normalised.len();
    let probe = CORNER_PROBE.min(count / 4).max(1);
    // The top edge is the way along the boundary that leaves the corner
    // heading leftwards.
    let step = if normalised[(corner + probe) % count].x
        <= normalised[(corner + count - probe) % count].x
    {
        1
    } else {
        count - 1
    };

    let mut span = Vec::new();
    for _ in 0..count {
        let index = (corner + step * (span.len() + 1)) % count;
        span.push(normalised[index]);
        if normalised[corner].x - normalised[index].x >= SPAN {
            break;
        }
    }
    span.reverse();
    span.push(normalised[corner]);
    if span.len() < 3 {
        return None;
    }

    let start = span[0];
    let end = normalised[corner];
    let chord = end - start;
    let length = chord.hypot();
    if length <= 0.0 {
        return None;
    }
    let arrival = chord.y.atan2(chord.x.abs()).to_degrees();
    let normal = kurbo::Vec2::new(-chord.y, chord.x) / length;
    let bow = span
        .iter()
        .map(|point| (*point - start).dot(normal))
        .fold(0.0, |a: f64, b| if b.abs() > a.abs() { b } else { a });

    // Slope of the counter's top edge: the highest point in each column of the
    // counter, fitted with a straight line.
    let columns = 32;
    let mut tops = vec![f64::NEG_INFINITY; columns];
    for point in &normalised {
        let column = ((point.x * (columns - 1) as f64).round() as usize).min(columns - 1);
        tops[column] = tops[column].max(point.y);
    }
    let used = (2..columns - 2)
        .filter(|column| tops[*column].is_finite())
        .collect::<Vec<_>>();
    if used.len() < 4 {
        return None;
    }
    let scale = (columns - 1) as f64;
    let mean_x = used.iter().map(|column| *column as f64).sum::<f64>() / used.len() as f64;
    let mean_y = used.iter().map(|column| tops[*column]).sum::<f64>() / used.len() as f64;
    let mut covariance = 0.0;
    let mut variance = 0.0;
    for column in &used {
        let dx = (*column as f64 - mean_x) / scale;
        covariance += dx * (tops[*column] - mean_y);
        variance += dx * dx;
    }
    let slope = if variance > 0.0 {
        covariance / variance
    } else {
        0.0
    };

    Some(AFeatures {
        arrival,
        bow,
        slope,
    })
}

/// Classify the counter of a double-storey `a`.
pub fn classify_a(features: &AFeatures) -> AShape {
    if features.arrival >= CONCAVE_DEGREES {
        AShape::Concave
    } else if features.arrival < CONVEX_DEGREES {
        AShape::Convex
    } else if features.arrival < FLAT_DEGREES {
        if features.slope < STEEP_SLOPE {
            AShape::Flat
        } else {
            AShape::Angled
        }
    } else {
        AShape::Angled
    }
}

/// Measure the counter of the bowl of a font's `a`, if it has one.
pub fn measure_a(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &crate::Results,
) -> Result<Option<(AFeatures, AShape)>, FontquantError> {
    let Some(bezglyph) = font.bezglyph_for_char(location, None, GLYPH)? else {
        return Ok(None);
    };
    let bezglyph = bezglyph
        .remove_overlaps()
        .unwrap_or_else(|_| bezglyph.clone());
    let paths = bezglyph.iter().collect::<Vec<_>>();
    let upem = font.head()?.units_per_em() as f64;
    let slant = slant_of(font, location, results)?;
    let shear = deslant_shear(slant);
    let Some(counter) = bowl_counter(&paths, shear, upem) else {
        return Ok(None);
    };
    Ok(a_features(&counter).map(|features| (features, classify_a(&features))))
}

/// Fill in the metric for the shape of the `a` of a font.
pub fn detect_a_shape(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    if let Some((_, shape)) = measure_a(font, location, results)? {
        results.add_metric(&DOUBLE_STOREY_A, MetricValue::String(shape.to_string()));
    }
    Ok(())
}

quantifier!(
    DOUBLE_STOREY_A,
    "structure/double_storey_a",
    r#"The shape of the counter at the top right corner of the bowl of a
    double-storey `a`, one of `convex`, `concave`, `flat` or `angled`.

    * **convex**: the arch comes down from above and curves into the stem, so the counter's top edge
      is descending as it reaches the corner;
    * **concave**: the arch sweeps up from below, so the counter comes to a point;
    * **flat**: the arch runs horizontally into the stem;
    * **angled**: the arch runs into the stem as a straight diagonal.

    Only the counter of the bowl is measured, so a glyph without one — a detached bowl, an open
    counter, or a single-storey `a` — gets no value."#,
    MetricValue::String("none".to_string())
);

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // it's a test suite
mod tests {
    use super::*;

    fn font(name: &str) -> Vec<u8> {
        let path = format!("{}/../tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).unwrap()
    }

    fn measure(name: &str) -> Option<AShape> {
        let data = font(name);
        let font = skrifa::FontRef::new(&data).unwrap();
        let results = crate::Results::new();
        measure_a(&font, &[], &results)
            .unwrap()
            .map(|(_, shape)| shape)
    }

    #[test]
    fn shapes_for_test_fonts() {
        for (name, shape) in [
            ("Asul-Regular.ttf", AShape::Convex),
            ("Marcellus-Regular.ttf", AShape::Flat),
            ("Ysabeau[wght].ttf", AShape::Angled),
            ("BacasimeAntique-Regular.ttf", AShape::Concave),
            ("OpenSans-VariableFont_wdth,wght.ttf", AShape::Angled),
        ] {
            assert_eq!(measure(name), Some(shape), "{name}");
        }
    }

    #[test]
    fn font_without_a_counter_is_not_measured() {
        // A pixel font: the `a` is a stack of rectangles with no counter.
        assert_eq!(measure("Redacted-Regular.ttf"), None);
    }

    fn polygon(points: &[(f64, f64)]) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(points[0]);
        for point in &points[1..] {
            path.line_to(*point);
        }
        path.close_path();
        path
    }

    #[test]
    fn bowl_counter_finds_the_lower_counter_and_rejects_a_full_height_one() {
        // An outer contour wound one way and a counter wound the other.
        let outer = polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 120.0), (0.0, 120.0)]);
        let low = polygon(&[(20.0, 10.0), (20.0, 60.0), (70.0, 60.0), (70.0, 10.0)]);
        let high = polygon(&[(20.0, 10.0), (20.0, 100.0), (80.0, 100.0), (80.0, 10.0)]);

        let double = [&outer, &low];
        let counter = bowl_counter(&double, Affine::IDENTITY, 1000.0);
        assert!(
            counter.is_some(),
            "a low counter is the bowl of a double-storey a"
        );

        // A counter that reaches most of the way up the glyph is a
        // single-storey bowl.
        let single = [&outer, &high];
        assert_eq!(bowl_counter(&single, Affine::IDENTITY, 1000.0), None);
    }
}
