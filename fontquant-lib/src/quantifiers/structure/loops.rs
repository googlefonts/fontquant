use std::fmt::Display;

use kurbo::{BezPath, PathEl, Point, Shape as _};

use crate::{
    monkeypatching::MakeBezGlyphs as _, quantifiers::appearance::stats::CurveStatistics as _,
    quantifier, MetricValue,
};

quantifier!(
    LOOPS,
    "structure/loops",
    r#"Fully or partially closed loops vary in structure. In Latin, this is how they typically map out:
    
Humanist/Old style: oval
Geometric sans/Slab serif: circle
Grotesque/Didone: superellipse
Square grotesque: rounded rectangle
"#,
    MetricValue::String("oval".to_string())
);

#[derive(Debug, Eq, Hash, PartialEq)]
enum LoopStyle {
    Triangular,
    RoundTriangular,
    Oval,
    Circular,
    Superellipse,
    RoundedRectangle,
    Rectangular,
}
impl Display for LoopStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopStyle::Triangular => write!(f, "triangular"),
            LoopStyle::RoundTriangular => write!(f, "round triangular"),
            LoopStyle::Oval => write!(f, "oval"),
            LoopStyle::Circular => write!(f, "circular"),
            LoopStyle::Superellipse => write!(f, "superellipse"),
            LoopStyle::RoundedRectangle => write!(f, "rounded rectangle"),
            LoopStyle::Rectangular => write!(f, "rectangular"),
        }
    }
}

const TEST_GLYPHS: [char; 5] = ['o', 'O', 'd', 'p', 'b'];

pub(crate) fn check_loop_style(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), crate::FontquantError> {
    // Try each and vote
    let styles = TEST_GLYPHS
        .iter()
        .flat_map(|&ch| {
            let glyph = font
                .bezglyph_for_char(location, Some(1.0), ch)
                .ok()
                .flatten()?;
            // Contours may overlap (a "d" is often a bowl plus a separate
            // stem), so take the union first to get the real outer silhouette.
            let glyph = glyph.remove_overlaps().ok()?;
            analyse_glyph_style(&glyph.0)
        })
        .collect::<Vec<_>>();
    let most_common = styles
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, style| {
            *acc.entry(style).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(style, _)| style);

    if let Some(style) = most_common {
        results.add_metric(&LOOPS, MetricValue::String(style.to_string()));
    }

    Ok(())
}

/// Minimum ratio of contour area to convex hull area for a contour to count as a
/// simple loop. Rejects things like the outer contour of "p", which has a stem
/// sticking out of the bowl.
const MIN_CONVEXITY: f64 = 0.97;

/// Axis ratios within this of 1.0 are called circular.
const CIRCULAR_TOLERANCE: f64 = 0.07;

/// Area over bounding box below these is closer to that of a triangle (0.5)
/// than of an ellipse (pi/4, about 0.785).
const TRIANGULAR_FILL: f64 = 0.65;
const ROUND_TRIANGULAR_FILL: f64 = 0.73;

/// Corner fullness this close to the corner of the bounding box leaves no room
/// for a visible corner radius.
const RECTANGULAR_FULLNESS: f64 = 0.99;

/// Boundaries on the fitted superellipse exponent (see `loop_metrics`). An
/// ellipse is exactly 2 and a rectangle is unbounded.
///
/// Measured on the outer contour of "o" across a few thousand Google Fonts the
/// exponent rises with the squareness of the design: Poppins 2.11, Inter 2.15,
/// Lato 2.20, Playfair Display 2.27, Abril Fatface 2.37, Archivo 2.44, Oswald
/// 3.04, Rajdhani 3.63. Round designs cluster between 2.0 and 2.3, so these cuts
/// separate round bowls from clearly squarish ones; they are too close together
/// to tell a geometric bowl from a grotesque one, which is left to `aspect` and
/// to the other quantifiers.
const ELLIPTICAL_EXPONENT: f64 = 2.25;
const SUPERELLIPSE_EXPONENT: f64 = 2.75;

/// Accuracy of the polyline approximation, in font units. The measurements are
/// exact for the curves themselves, so this only has to be small compared to
/// the contour.
const FLATTEN_TOLERANCE: f64 = 0.1;

/// Measurements of a single contour, all taken in the normalised frame of
/// `normalise`, so they do not depend on slant or on the font's units.
#[derive(Debug)]
struct LoopMetrics {
    /// Area over bounding box area. Triangle 0.5, ellipse 0.785, rectangle 1.
    fill: f64,
    /// Ratio of the principal axes; 1 for a circle. Squareness is left to
    /// `exponent`, so this is what separates circular from oval.
    aspect: f64,
    /// How far the outline reaches into the corner of the bounding box: 0.707
    /// for an ellipse, rising to 1.0 for a rectangle.
    fullness: f64,
    /// Exponent n of the superellipse |x/a|^n + |y/b|^n = 1 that matches the
    /// corner fullness: 2 is an ellipse, 4 a squircle, higher is squarer.
    exponent: f64,
    /// Area over convex hull area; below 1 the contour has dents or appendages.
    convexity: f64,
}

/// Classify the outer contour of a glyph, or `None` if it is not a simple loop.
///
/// Counters are ignored: their shape often differs from the outer contour (for
/// example, Didone counters are more superelliptical than the outer bowls), and
/// it is the outer contour that reads as the glyph's circularity.
fn analyse_glyph_style(paths: &[BezPath]) -> Option<LoopStyle> {
    let contours = paths
        .iter()
        .filter_map(|path| Some((path, flatten_closed(path)?, path.area())))
        .collect::<Vec<_>>();

    // Winding direction is not standardised: TrueType outer contours run
    // clockwise and CFF outer contours run counter-clockwise, and individual
    // glyphs may be reversed too. So we take the direction holding the bulk of
    // the glyph's area as the outer direction - an outer contour always encloses
    // more than the counters inside it.
    let (positive, negative) =
        contours
            .iter()
            .fold((0.0f64, 0.0f64), |(pos, neg), (_, _, area)| {
                if *area > 0.0 {
                    (pos + area.abs(), neg)
                } else {
                    (pos, neg + area.abs())
                }
            });
    let outer_is_positive = positive > negative;

    let (path, _, _) = contours
        .iter()
        .filter(|(_, _, area)| (*area > 0.0) == outer_is_positive)
        .max_by(|(_, _, a), (_, _, b)| a.abs().total_cmp(&b.abs()))?;

    analyse_loop(path)
}

/// Classify a single closed contour.
fn analyse_loop(path: &BezPath) -> Option<LoopStyle> {
    let metrics = loop_metrics(path)?;
    if metrics.convexity < MIN_CONVEXITY {
        return None;
    }
    log::debug!("loop {metrics:?}");
    Some(classify(&metrics))
}

fn loop_metrics(path: &BezPath) -> Option<LoopMetrics> {
    let area = path.area().abs();
    let points = flatten_closed(path)?;
    let hull_area = polygon_area(&convex_hull(&points)).abs();
    if area <= 0.0 || hull_area <= 0.0 {
        return None;
    }

    let (normalised, aspect) = normalise(path, &points)?;
    let fullness = corner_fullness(&normalised);
    let exponent = if fullness >= 1.0 {
        f64::INFINITY
    } else if fullness <= 0.0 {
        2.0
    } else {
        -std::f64::consts::LN_2 / fullness.ln()
    };

    Some(LoopMetrics {
        fill: polygon_area(&normalised).abs() / 4.0,
        aspect,
        fullness,
        exponent,
        convexity: area / hull_area,
    })
}

/// Re-express the contour so that measurements do not depend on italic slant or
/// on the font's units: undo the slant, scale the second moments to unity so an
/// ellipse becomes a circle, and stretch the bounding box to the unit box.
///
/// Slant is treated as a horizontal shear, x -> x - k*y, because that is what an
/// oblique design is: a shear leaves y alone, so it preserves both the vertical
/// proportions and the area. A loop that was rotated rather than sheared is not
/// undone.
///
/// Returns the transformed points and the axis ratio, 1 for a circle and a/b for
/// an ellipse with semi-axes a and b.
fn normalise(path: &BezPath, points: &[Point]) -> Option<(Vec<Point>, f64)> {
    if path.area() == 0.0 {
        return None;
    }
    let variance = path.variance();
    // The shear that removes the x/y correlation, and with it the tilt of a
    // slanted ellipse, whatever its axes.
    let shear = path.slant();
    let centroid = path.center_of_mass();
    let desheared_xx = variance.x - shear * shear * variance.y;
    if desheared_xx <= 0.0 || variance.y <= 0.0 {
        return None;
    }

    // Scaling the second moments to a constant maps an ellipse onto a circle and
    // leaves a superellipse's exponent alone, since it rescales both axes by the
    // same factor.
    let (scale_x, scale_y) = (
        1.0 / (2.0 * desheared_xx.sqrt()),
        1.0 / (2.0 * variance.y.sqrt()),
    );
    let scaled = points
        .iter()
        .map(|p| {
            let dx = (p.x - centroid.x) - shear * (p.y - centroid.y);
            Point::new(dx * scale_x, (p.y - centroid.y) * scale_y)
        })
        .collect::<Vec<_>>();

    let (min_x, max_x, min_y, max_y) = extents(&scaled);
    let (half_w, half_h) = ((max_x - min_x) / 2.0, (max_y - min_y) / 2.0);
    if half_w <= 0.0 || half_h <= 0.0 {
        return None;
    }
    let (mid_x, mid_y) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let normalised = scaled
        .iter()
        .map(|p| Point::new((p.x - mid_x) / half_w, (p.y - mid_y) / half_h))
        .collect();
    Some((
        normalised,
        (variance.y.max(desheared_xx) / variance.y.min(desheared_xx)).sqrt(),
    ))
}

/// How far the outline reaches into the corner of the unit box, where
/// |x| = |y|. A superellipse |x/a|^n + |y/b|^n = 1 reaches 2^(-1/n) there, which
/// inverts to its exponent.
fn corner_fullness(points: &[Point]) -> f64 {
    let mut per_quadrant = [0.0f64; 4];
    for (i, &p) in points.iter().enumerate() {
        let q = points[(i + 1) % points.len()];
        let quadrant = usize::from(p.x >= 0.0) + 2 * usize::from(p.y >= 0.0);
        let mut best = p.x.abs().min(p.y.abs());
        // Interpolate the point where this edge crosses |x| = |y|, so the answer
        // does not depend on where the curves happened to be split.
        let (p_zero, q_zero) = (p.x.abs() - p.y.abs(), q.x.abs() - q.y.abs());
        if p_zero * q_zero < 0.0 {
            let t = p_zero / (p_zero - q_zero);
            let crossing = p.lerp(q, t);
            best = best.max(crossing.x.abs().min(crossing.y.abs()));
        }
        per_quadrant[quadrant] = per_quadrant[quadrant].max(best);
    }
    let mut fullness = per_quadrant;
    fullness.sort_by(f64::total_cmp);
    // Median of the four quadrants, so one distorted corner cannot decide it.
    (fullness[1] + fullness[2]) / 2.0
}

fn classify(metrics: &LoopMetrics) -> LoopStyle {
    if metrics.fill < TRIANGULAR_FILL {
        LoopStyle::Triangular
    } else if metrics.fill < ROUND_TRIANGULAR_FILL {
        LoopStyle::RoundTriangular
    } else if metrics.fullness >= RECTANGULAR_FULLNESS {
        LoopStyle::Rectangular
    } else if metrics.exponent < ELLIPTICAL_EXPONENT {
        if metrics.aspect.ln().abs() < CIRCULAR_TOLERANCE {
            LoopStyle::Circular
        } else {
            LoopStyle::Oval
        }
    } else if metrics.exponent < SUPERELLIPSE_EXPONENT {
        LoopStyle::Superellipse
    } else {
        LoopStyle::RoundedRectangle
    }
}

fn extents(points: &[Point]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(min_x, max_x, min_y, max_y), p| {
            (
                min_x.min(p.x),
                max_x.max(p.x),
                min_y.min(p.y),
                max_y.max(p.y),
            )
        },
    )
}

/// Flatten the path into a single closed polyline; `None` if it has several
/// subpaths.
fn flatten_closed(path: &BezPath) -> Option<Vec<Point>> {
    let mut points = Vec::new();
    let mut subpaths = 0;
    kurbo::flatten(path.iter(), FLATTEN_TOLERANCE, |el| match el {
        PathEl::MoveTo(p) => {
            subpaths += 1;
            points.push(p);
        }
        PathEl::LineTo(p) => points.push(p),
        _ => {}
    });
    if subpaths != 1 || points.len() < 3 {
        return None;
    }
    if points.first() == points.last() {
        points.pop();
    }
    Some(points)
}

fn polygon_area(points: &[Point]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        / 2.0
}

/// Andrew's monotone chain.
fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut pts = points.to_vec();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: Point, a: Point, b: Point| (a - o).cross(b - o);
    let mut hull: Vec<Point> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        for &p in &pts {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
        if pass == 0 {
            pts.reverse();
        }
    }
    hull
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // it's a test suite
mod tests {
    use super::*;
    use kurbo::{Affine, Ellipse, Rect};

    fn analyse(path: &BezPath) -> Option<LoopStyle> {
        analyse_glyph_style(std::slice::from_ref(path))
    }

    fn metrics(path: &BezPath) -> LoopMetrics {
        loop_metrics(path).unwrap()
    }

    fn reversed(path: &BezPath) -> BezPath {
        path.reverse_subpaths()
    }

    /// A superellipse |x|^n + |y|^n = a^n as a polygon.
    fn superellipse(n: f64, a: f64, b: f64) -> BezPath {
        let mut path = BezPath::new();
        let steps = 720;
        for i in 0..=steps {
            let t = i as f64 / steps as f64 * std::f64::consts::TAU;
            let (sin, cos) = t.sin_cos();
            let x = cos.abs().powf(2.0 / n) * cos.signum() * a;
            let y = sin.abs().powf(2.0 / n) * sin.signum() * b;
            if i == 0 {
                path.move_to((x, y));
            } else {
                path.line_to((x, y));
            }
        }
        path.close_path();
        path
    }

    #[test]
    fn ellipse_metrics() {
        let m = metrics(&Ellipse::new((10., 5.), (40., 10.), 0.0).to_path(0.001));
        assert!((m.aspect - 4.0).abs() < 0.01, "aspect {}", m.aspect);
        assert!((m.fill - std::f64::consts::FRAC_PI_4).abs() < 0.001);
        assert!((m.exponent - 2.0).abs() < 0.02, "exponent {}", m.exponent);
        assert!((m.fullness - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.005);
    }

    #[test]
    fn circle_and_oval() {
        assert_eq!(
            analyse(&Ellipse::new((0., 0.), (100., 100.), 0.0).to_path(0.001)),
            Some(LoopStyle::Circular)
        );
        assert_eq!(
            analyse(&Ellipse::new((0., 0.), (80., 100.), 0.0).to_path(0.001)),
            Some(LoopStyle::Oval)
        );
    }

    #[test]
    fn slanted_oval_is_still_oval() {
        let italic = Affine::new([1.0, 0.0, 0.2, 1.0, 0.0, 0.0]);
        let path = italic * Ellipse::new((0., 0.), (80., 100.), 0.0).to_path(0.001);
        let m = metrics(&path);
        assert!((m.aspect - 1.25).abs() < 0.01, "aspect {}", m.aspect);
        assert!((m.exponent - 2.0).abs() < 0.02, "exponent {}", m.exponent);
        assert_eq!(analyse(&path), Some(LoopStyle::Oval));

        // Rotation is not undone, but a circle has nothing to undo.
        let rotated = Affine::rotate(0.4) * Ellipse::new((0., 0.), (100., 100.), 0.0).to_path(0.001);
        assert_eq!(analyse(&rotated), Some(LoopStyle::Circular));
    }

    #[test]
    fn rectangle_is_rectangular() {
        assert_eq!(
            analyse(&Rect::new(0., 0., 80., 100.).to_path(0.001)),
            Some(LoopStyle::Rectangular)
        );
    }

    #[test]
    fn superellipse_exponent_measured() {
        for n in [2.5, 3.0, 4.0, 6.0] {
            for (a, b) in [(100.0, 100.0), (100.0, 80.0)] {
                let m = metrics(&superellipse(n, a, b));
                assert!(
                    (m.exponent - n).abs() < 0.1,
                    "expected exponent {n}, got {}",
                    m.exponent
                );
            }
        }
    }

    #[test]
    fn squircle_bands() {
        assert_eq!(
            analyse(&superellipse(2.5, 100.0, 100.0)),
            Some(LoopStyle::Superellipse)
        );
        assert_eq!(
            analyse(&superellipse(4.0, 100.0, 100.0)),
            Some(LoopStyle::RoundedRectangle)
        );
    }

    #[test]
    fn inner_counter_ignored() {
        // A squarish outer contour with an oppositely-wound circular counter,
        // as in an "o": the outer contour decides the answer.
        let outer = Rect::new(0., 0., 100., 100.).to_path(0.001);
        let counter = reversed(&Ellipse::new((50., 50.), (30., 30.), 0.0).to_path(0.001));
        assert_eq!(
            analyse_glyph_style(&[counter, outer]),
            Some(LoopStyle::Rectangular)
        );

        // ...and regardless of which winding the font uses.
        let outer = reversed(&Rect::new(0., 0., 100., 100.).to_path(0.001));
        let counter = Ellipse::new((50., 50.), (30., 30.), 0.0).to_path(0.001);
        assert_eq!(
            analyse_glyph_style(&[counter, outer]),
            Some(LoopStyle::Rectangular)
        );
    }

    #[test]
    fn non_convex_outer_with_neat_counter_rejected() {
        // Like a "p": the outer contour has a stem, so it is not a loop, and the
        // counter is not consulted.
        let mut stemmed = BezPath::new();
        let points = [
            (0., 0.),
            (100., 0.),
            (100., 100.),
            (60., 100.),
            (60., 300.),
            (40., 300.),
            (40., 100.),
            (0., 100.),
        ];
        for (i, (x, y)) in points.iter().enumerate() {
            if i == 0 {
                stemmed.move_to((*x, *y));
            } else {
                stemmed.line_to((*x, *y));
            }
        }
        stemmed.close_path();
        let counter = reversed(&Ellipse::new((30., 50.), (20., 20.), 0.0).to_path(0.001));
        assert_eq!(analyse_glyph_style(&[stemmed, counter]), None);
    }
}
