//! The outline geometry that the stem terminal and serif quantifiers share.
//!
//! Everything here works in *deslanted* coordinates: the outline is sheared
//! horizontally by the font's slant so that a stem is vertical and the shapes
//! on the ends of stems can be measured as they were drawn. [`unslant`] maps
//! a point back into the glyph's own coordinates.
//!
//! The outline is sampled segment by segment. Sampling only decides *where* to
//! look; the geometry itself comes from the analytic tangents, so the
//! measurements do not depend on the sampling resolution.

use std::f64::consts::PI;

use kurbo::{Affine, BezPath, ParamCurve as _, PathSeg, Point, Shape as _, Vec2};

use crate::{
    MetricValue,
    error::FontquantError,
    quantifiers::appearance::stats::{SLANT, WholeFontStatistics},
};

/// Samples per Bezier segment. Only used to follow the outline; the geometry is
/// measured from the analytic tangents, not from the sampling.
pub(crate) const SAMPLES_PER_SEGMENT: usize = 16;

/// How far from vertical a wall may be and still count as part of a stem.
const VERTICAL_TOLERANCE_DEGREES: f64 = 15.0;

/// One piece of a contour, in deslanted coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentSlice {
    /// Index of the segment within its contour.
    pub index: usize,
    /// The piece of that segment which belongs to the chain.
    pub segment: PathSeg,
    /// Parameter range of `segment` covered by this slice.
    pub t0: f64,
    pub t1: f64,
}

/// Deslanted coordinates: undo the slant as a horizontal shear.
pub fn deslant_shear(slant: f64) -> Affine {
    Affine::new([1.0, 0.0, -slant, 1.0, 0.0, 0.0])
}

/// Map a point from deslanted coordinates back into the glyph's coordinates.
pub fn unslant(point: Point, slant: f64) -> Point {
    Point::new(point.x + slant * point.y, point.y)
}

/// Map a path from deslanted coordinates back into the glyph's coordinates.
pub fn unslant_path(path: &BezPath, slant: f64) -> BezPath {
    Affine::new([1.0, 0.0, slant, 1.0, 0.0, 0.0]) * path
}

/// The tangent of the slant shear, `x' = x - slant * y`.
pub(crate) fn slant_of(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &crate::Results,
) -> Result<f64, FontquantError> {
    if let Some((_, MetricValue::Angle(degrees))) = results.get(&SLANT.name) {
        // SLANT reports positive values for left-leaning shapes.
        return Ok(-degrees.to_radians().tan());
    }
    let statistics = WholeFontStatistics::new_from_font(font, location)?;
    Ok(-statistics.slant.to_radians().tan())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Sample {
    pub(crate) point: Point,
    pub(crate) tangent: Vec2,
    pub(crate) segment: usize,
    pub(crate) t: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Run {
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// +1 when the outline travels up this wall, -1 when it travels down.
    pub(crate) direction: i8,
}

pub(crate) fn samples(segments: &[PathSeg], shear: Affine) -> Vec<Sample> {
    let mut samples = Vec::with_capacity(segments.len() * SAMPLES_PER_SEGMENT);
    for (index, segment) in segments.iter().enumerate() {
        for k in 0..=SAMPLES_PER_SEGMENT {
            let t = k as f64 / SAMPLES_PER_SEGMENT as f64;
            let point = shear * segment.eval(t);
            let mut tangent = segment_tangent(segment, t);
            if tangent.hypot() < 1e-9 {
                tangent = segment.end() - segment.start();
            }
            if tangent.hypot() < 1e-9 {
                // A degenerate segment carries no direction of its own; leave
                // it out rather than letting it break the run.
                continue;
            }
            samples.push(Sample {
                point,
                tangent: tangent.normalize(),
                segment: index,
                t,
            });
        }
    }
    samples
}

fn segment_tangent(segment: &PathSeg, t: f64) -> Vec2 {
    use kurbo::ParamCurveDeriv as _;
    match segment {
        PathSeg::Line(line) => line.p1 - line.p0,
        PathSeg::Quad(quad) => quad.deriv().eval(t).to_vec2(),
        PathSeg::Cubic(cubic) => cubic.deriv().eval(t).to_vec2(),
    }
}

fn is_vertical(tangent: Vec2) -> bool {
    let from_vertical = tangent.x.abs().atan2(tangent.y.abs()).to_degrees();
    from_vertical <= VERTICAL_TOLERANCE_DEGREES
}

/// Maximal cyclic runs of samples whose tangent is close to vertical.
pub(crate) fn vertical_runs(samples: &[Sample]) -> Vec<Run> {
    let n = samples.len();
    let vertical = samples
        .iter()
        .map(|sample| is_vertical(sample.tangent))
        .collect::<Vec<_>>();
    if vertical.iter().all(|&v| v) || !vertical.iter().any(|&v| v) {
        return Vec::new();
    }
    let mut runs = Vec::new();
    for start in 0..n {
        if !vertical[start] || vertical[(start + n - 1) % n] {
            continue;
        }
        let mut end = start;
        for _ in 0..n - 1 {
            if !vertical[(end + 1) % n] {
                break;
            }
            end = (end + 1) % n;
        }
        let mut y_sum = 0.0;
        let mut i = start;
        loop {
            y_sum += samples[i].tangent.y;
            if i == end {
                break;
            }
            i = (i + 1) % n;
        }
        let direction = if y_sum >= 0.0 { 1 } else { -1 };
        runs.push(Run {
            start,
            end,
            direction,
        });
    }
    runs
}

/// The turn from one tangent to the next, as part of a running turn. Where
/// the outline reverses (at the pointed tip of a flare, say) the direction of
/// the reversal is ambiguous; take the one that keeps the running turn small,
/// so that a doubling back cancels out.
pub(crate) fn cap_step(turn: f64, previous: f64, current: f64) -> f64 {
    let step = signed_angle_difference(previous, current);
    if (step.abs() - PI).abs() > 1e-9 {
        return step;
    }
    let opposite = step - step.signum() * 2.0 * PI;
    if (turn + opposite).abs() < (turn + step).abs() {
        opposite
    } else {
        step
    }
}

pub(crate) fn signed_angle_difference(from: f64, to: f64) -> f64 {
    let mut difference = to - from;
    while difference > PI {
        difference -= 2.0 * PI;
    }
    while difference <= -PI {
        difference += 2.0 * PI;
    }
    difference
}

pub(crate) fn cap_width(samples: &[Sample], a: usize, b: usize) -> f64 {
    (samples[b].point.x - samples[a].point.x).abs()
}

pub(crate) fn run_length(samples: &[Sample], start: usize, end: usize) -> f64 {
    let n = samples.len();
    let mut length = 0.0;
    let mut i = start;
    while i != end {
        let next = (i + 1) % n;
        length += samples[next].point.distance(samples[i].point);
        i = next;
    }
    length
}

/// Split the cyclic sample range `from..=to` into pieces of the segments it
/// covers.
pub(crate) fn slices_between(
    segments: &[PathSeg],
    samples: &[Sample],
    from: usize,
    to: usize,
) -> Vec<SegmentSlice> {
    let n = samples.len();
    let mut slices = Vec::new();
    let mut i = from;
    let mut index = samples[from].segment;
    let mut t0 = samples[from].t;
    loop {
        if i == to {
            push_slice(&mut slices, segments, index, t0, samples[to].t);
            break;
        }
        let next = (i + 1) % n;
        if samples[next].segment == index {
            i = next;
        } else {
            push_slice(&mut slices, segments, index, t0, 1.0);
            index = samples[next].segment;
            t0 = 0.0;
            i = next;
        }
    }
    slices
}

fn push_slice(
    slices: &mut Vec<SegmentSlice>,
    segments: &[PathSeg],
    index: usize,
    t0: f64,
    t1: f64,
) {
    if t1 <= t0 {
        return;
    }
    slices.push(SegmentSlice {
        index,
        segment: segments[index].subsegment(t0..t1),
        t0,
        t1,
    });
}

pub(crate) fn is_ink(paths: &[&BezPath], point: Point) -> bool {
    paths.iter().map(|path| path.winding(point)).sum::<i32>() != 0
}

pub(crate) fn flatten(path: &BezPath, tolerance: f64) -> Vec<Point> {
    let mut points = Vec::new();
    kurbo::flatten(path.iter(), tolerance, |element| match element {
        kurbo::PathEl::MoveTo(point) | kurbo::PathEl::LineTo(point) => points.push(point),
        _ => {}
    });
    points
}

/// The Bezier segments of a chain, as one path in deslanted coordinates.
pub fn chain_path(slices: &[SegmentSlice]) -> BezPath {
    let mut path = BezPath::new();
    if let Some(first) = slices.first() {
        path.move_to(first.segment.start());
        for slice in slices {
            path.push(slice.segment.as_path_el());
        }
    }
    path
}

pub(crate) fn unit_tangent(slice: &SegmentSlice, at_end: bool) -> Option<Vec2> {
    // `segment` is already the piece described by `t0..t1`, so its own
    // parameter range is always 0..1.
    let t = if at_end { 1.0 } else { 0.0 };
    let mut tangent = segment_tangent(&slice.segment, t);
    if tangent.hypot() < 1e-9 {
        tangent = slice.segment.end() - slice.segment.start();
    }
    (tangent.hypot() > 1e-9).then(|| tangent.normalize())
}

pub(crate) fn corner_turn(incoming: Option<Vec2>, outgoing: Option<Vec2>) -> Option<f64> {
    let (incoming, outgoing) = (incoming?, outgoing?);
    Some(
        signed_angle_difference(incoming.atan2(), outgoing.atan2())
            .abs()
            .to_degrees(),
    )
}
