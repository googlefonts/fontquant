use kurbo::{BezPath, PathEl, Point, Vec2};
use skrifa::{FontRef, raw::TableProvider, setting::VariationSetting};

use crate::{
    MetricValue, bezglyph::BezGlyph, error::FontquantError, monkeypatching::MakeBezGlyphs,
    quantifier,
};

quantifier!(
    ROUGHNESS,
    "appearance/roughness",
    r#"How much longer the outlines are than the most efficient path which follows them.

Each contour is replaced by the polyline of least length which never strays more than a
tolerance away from it, and the contour's length is compared to that polyline's length. A
straight edge scores zero, a smooth bowl around 0.2%, and outlines with hand-drawn or
digitised rough edges several percent.

The tolerance is swept over a range of scales so both fine edge noise and larger wobbles
are counted. Every tolerance is capped at half the average stroke width, so that genuine
features such as hairlines and serifs are never simplified away; this stops high-contrast
designs from being mistaken for rough ones.

Measured over the basic Latin letters and figures, and reported as the percentage by which
the outline is longer than the efficient path, weighted by contour and glyph length.
Typical values: Inter 1.3, IM Fell 2.9, Rubik Beastly 11.9, Rubik Bubbles 17.7. Suggested
bands: below 2.2 clean, 2.2 to 2.8 slightly distressed, 2.8 to 5 rough display, 5 to 9 very
rough, above 9 extreme."#,
    MetricValue::Metric(1.0)
);

/// Scales, as a fraction of the em, at which the outline is simplified.
const SCALES: [f64; 5] = [0.005, 0.01, 0.02, 0.04, 0.08];

/// Every tolerance is capped at this fraction of the average stroke width.
const STROKE_WIDTH_CAP: f64 = 0.5;

/// Accuracy of the polyline used as the outline, as a fraction of the em. It only has to
/// be well below the smallest entry of `SCALES`.
const FLATTEN_TOLERANCE: f64 = 0.0002;

pub fn get_roughness(
    font: &FontRef,
    location: &[VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let upem = font.head()?.units_per_em() as f64;
    let mut total_excess = 0.0;
    let mut total_length = 0.0;
    for c in ('A'..='Z').chain('a'..='z').chain('0'..='9') {
        let Some(glyph) = font.bezglyph_for_char(location, None, c)? else {
            continue;
        };
        let Some((excess, length)) = glyph_roughness(&glyph, upem) else {
            continue;
        };
        total_excess += excess * length;
        total_length += length;
    }

    if total_length > 0.0 {
        results.add_metric(
            &ROUGHNESS,
            MetricValue::Metric(total_excess / total_length * 100.0),
        );
    }
    Ok(())
}

/// Length-weighted excess path length of a single glyph, and the glyph's outline length.
fn glyph_roughness(glyph: &BezGlyph, upem: f64) -> Option<(f64, f64)> {
    let tolerance = upem * FLATTEN_TOLERANCE;
    let contours = glyph
        .iter()
        .flat_map(|path| flatten(path, tolerance))
        .collect::<Vec<_>>();

    let mut area = 0.0;
    let mut length = 0.0;
    for contour in &contours {
        area += contour.area();
        length += contour.length();
    }
    if area <= 0.0 || length <= 0.0 {
        return None;
    }

    // Twice the area over the perimeter is the average stroke width, and the cap stops
    // the simplification from swallowing strokes thinner than the tolerance.
    let cap = STROKE_WIDTH_CAP * 2.0 * area / length;
    let mut excess = 0.0;
    for contour in &contours {
        let contour_length = contour.length();
        if contour_length <= 0.0 {
            continue;
        }
        for scale in SCALES {
            let simplified = contour.simplified_length((scale * upem).min(cap));
            if simplified > 0.0 {
                excess += contour_length * (contour_length / simplified - 1.0);
            }
        }
    }
    let excess = excess / SCALES.len() as f64 / length;

    Some((excess, length))
}

/// A flattened contour, with the closing segment counted if the original path was closed.
struct Contour {
    points: Vec<Point>,
    closed: bool,
}

impl Contour {
    fn new(mut points: Vec<Point>, closed: bool) -> Option<Self> {
        while points.len() > 1 && points.first() == points.last() {
            points.pop();
        }
        if points.len() < 2 {
            return None;
        }
        if closed {
            // The start point of a contour is a designer choice; anchoring the
            // simplification at the point furthest from the centroid keeps the measure
            // independent of it.
            let centroid = points
                .iter()
                .fold(Vec2::ZERO, |sum, point| sum + (*point - Point::ORIGIN))
                / points.len() as f64;
            let centroid = Point::ORIGIN + centroid;
            let furthest = points
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    (**a - centroid)
                        .hypot2()
                        .total_cmp(&(**b - centroid).hypot2())
                })
                .map(|(index, _)| index)
                .unwrap_or(0);
            points.rotate_left(furthest);
        }
        Some(Contour { points, closed })
    }

    fn length(&self) -> f64 {
        let chords = self
            .points
            .windows(2)
            .map(|pair| pair[0].distance(pair[1]))
            .sum::<f64>();
        if self.closed {
            chords + self.points[0].distance(*self.points.last().unwrap_or(&self.points[0]))
        } else {
            chords
        }
    }

    fn area(&self) -> f64 {
        if !self.closed {
            return 0.0;
        }
        let double_area = self
            .points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f64>();
        (double_area / 2.0).abs()
    }

    /// Length of the shortest polyline which stays within `tolerance` of the contour.
    fn simplified_length(&self, tolerance: f64) -> f64 {
        let mut points = self.points.clone();
        if self.closed {
            points.push(points[0]);
        }
        let simplified = simplify(&points, tolerance);
        simplified
            .windows(2)
            .map(|pair| pair[0].distance(pair[1]))
            .sum()
    }
}

fn flatten(path: &BezPath, tolerance: f64) -> Vec<Contour> {
    let mut contours = Vec::new();
    let mut points = Vec::new();
    let mut closed = false;
    kurbo::flatten(path.iter(), tolerance, |element| match element {
        PathEl::MoveTo(point) => {
            if let Some(contour) = Contour::new(std::mem::take(&mut points), closed) {
                contours.push(contour);
            }
            closed = false;
            points.push(point);
        }
        PathEl::LineTo(point) => points.push(point),
        PathEl::ClosePath => {
            if let Some(contour) = Contour::new(std::mem::take(&mut points), true) {
                contours.push(contour);
            }
            closed = true;
        }
        _ => {}
    });
    if let Some(contour) = Contour::new(points, closed) {
        contours.push(contour);
    }
    contours
}

/// Douglas–Peucker simplification. The result is the greedy shortest polyline which stays
/// within `tolerance` of the input, up to the usual approximation of the algorithm.
fn simplify(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;

    let mut pending = vec![(0, points.len() - 1)];
    while let Some((first, last)) = pending.pop() {
        if last <= first + 1 {
            continue;
        }
        let mut furthest = first;
        let mut distance = 0.0;
        for (index, point) in points.iter().enumerate().take(last).skip(first + 1) {
            let candidate = distance_to_line(*point, points[first], points[last]);
            if candidate > distance {
                furthest = index;
                distance = candidate;
            }
        }
        if distance > tolerance {
            keep[furthest] = true;
            pending.push((first, furthest));
            pending.push((furthest, last));
        }
    }

    points
        .iter()
        .zip(keep)
        .filter_map(|(point, keep)| keep.then_some(*point))
        .collect()
}

fn distance_to_line(point: Point, start: Point, end: Point) -> f64 {
    let line = end - start;
    let length = line.hypot();
    if length == 0.0 {
        (point - start).hypot()
    } else {
        ((point - start).cross(line) / length).abs()
    }
}

#[cfg(test)]
mod tests {
    use skrifa::FontRef;

    use super::*;

    fn roughness(bytes: &[u8]) -> f64 {
        #![allow(clippy::unwrap_used, clippy::expect_used)]
        let mut results = crate::Results::new();
        get_roughness(&FontRef::new(bytes).unwrap(), &[], &mut results).expect("Shouldn't fail");
        let (_, MetricValue::Metric(value)) = results
            .get("appearance/roughness")
            .expect("Roughness should be reported")
        else {
            panic!("Roughness should be a metric")
        };
        *value
    }

    #[test]
    fn test_roughness() {
        let clean = roughness(include_bytes!(
            "../../../../tests/fonts/OpenSans-VariableFont_wdth,wght.ttf"
        ));
        let rough = roughness(include_bytes!(
            "../../../../tests/fonts/HomemadeApple-Regular.ttf"
        ));
        println!("clean {clean}, rough {rough}");
        assert!(
            clean < 2.0,
            "A clean font should score below 2, got {clean}"
        );
        assert!(
            rough > 4.0,
            "A rough font should score above 4, got {rough}"
        );
    }

    #[test]
    fn test_simplify_keeps_corners() {
        let square = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(0.0, 0.0),
        ];
        assert_eq!(simplify(&square, 1.0).len(), 5);
    }

    #[test]
    fn test_straight_line_has_no_excess() {
        #![allow(clippy::expect_used)]
        let contour = Contour::new(
            (0..=10).map(|i| Point::new(f64::from(i), 0.0)).collect(),
            false,
        )
        .expect("Contour should be valid");
        assert!(contour.simplified_length(1.0) - contour.length() < 1e-9);
    }
}
