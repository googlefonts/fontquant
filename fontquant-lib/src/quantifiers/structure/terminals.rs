//! Detection of stem terminals in the letters `l`, `m` and `n`.
//!
//! This is the first half of terminal classification. Given a glyph, we look
//! for the ends of its stems and return the chain of Bezier segments that makes
//! up each one: the two roughly parallel walls of the stem, and the run of
//! segments that joins them around the end.
//!
//! # How it works
//!
//! Every contour is sampled segment by segment. A sample is part of a **wall**
//! when the outline is travelling almost vertically, which after deslanting
//! means almost parallel to the stem. Maximal runs of wall samples are joined by
//! runs that are *not* vertical, and those in-between runs are the candidate
//! terminals. A pair of consecutive walls is a stem end when
//!
//! * the two walls run in opposite directions (up one side, down the other),
//! * together with the joining run they turn by about 180 degrees (a run with a
//!   pointed tip, where the outline doubles back on itself, is measured the way
//!   round that keeps the turn small),
//! * the run spans roughly the width of the stem, rather than the width of a
//!   counter or of the whole glyph,
//! * it does not bulge out along the stem much more than the stem is wide (so
//!   that an arch joining two stems is not mistaken for a terminal), and
//! * the region just inside the run, between the walls, is ink, while the
//!   region just outside is not.
//!
//! The last two tests are what separate the top of a stem from an arch: at the
//! top of an `n`, for example, the run between the outer walls spans the whole
//! arch and the point between the walls is the counter, not ink.
//!
//! A flared stem usually ends in a short, almost vertical piece where the flare
//! turns over, which looks exactly like a wall. Such a piece is part of the
//! terminal, so a pair whose wall is too short to be a wall is extended through
//! the neighbouring run until it reaches a real one.
//!
//! All of the geometry is measured after deslanting the outline with the font's
//! slant, so italic stems are vertical and the corner angles are the angles the
//! designer drew rather than their sheared image.
//!
//! The letters `l`, `m` and `n` all have a single contour with no counters, so
//! the chains can be contiguous slices of that contour. Serif fonts are not
//! (yet) supported: their stems end in serifs, which do not have the shape this
//! detector is looking for.
//!
//! # Classification
//!
//! The shape of the cap and the width profile of the stem are classified
//! separately, because a terminal can be, for example, both angled and flared.
//!
//! [`classify_terminal`] categorises the cap:
//!
//! * **blunted** - a flat cap with roughly 90 degree corners;
//! * **angled** - a flat cap whose corners are not at 90 degrees, usually
//!   because the end of the stem is cut at a slant;
//! * **rounded** - a continuous, roughly semicircular curve joining the walls;
//! * **soft** - a rounded-rectangle cap: a straight top (or an apex flatter
//!   than a circle) with rounded corners.
//!
//! [`classify_stem_profile`] categorises the width of the stem:
//!
//! * **flared** - the stem is wider at the terminal than one stem width down;
//! * **straight** - it does not change width appreciably;
//! * **tapered** - the stem is narrower at the terminal.
//!
//! [`most_common_shape`] and [`most_common_profile`] pick the most common value
//! across a font, and return `None` on a tie. They fill in the
//! `structure/stem_terminals` and `structure/stem_profile` metrics.
//!
//! # Limitations
//!
//! Some designs do not cap the top of the stems of `n` and `m`: the outer wall
//! runs straight into the arch, and the top left corner is a corner of the
//! outline rather than the join between two walls. Those glyphs have no chain
//! at the top left, so they yield one terminal fewer. The same is true of an
//! `l` whose foot is a tail rather than an end of the stem. Fonts for which no
//! terminal can be found at all, which includes most serif designs, get no
//! classification.

use std::{collections::HashSet, f64::consts::PI};

use kurbo::{Affine, BezPath, ParamCurve as _, PathSeg, Point, Shape as _, Vec2};
use read_fonts::TableProvider as _;

use crate::{
    MetricValue,
    error::FontquantError,
    monkeypatching::MakeBezGlyphs as _,
    quantifier,
    quantifiers::appearance::stats::{SLANT, WholeFontStatistics},
};

/// Samples per Bezier segment. Only used to follow the outline; the geometry is
/// measured from the analytic tangents, not from the sampling.
const SAMPLES_PER_SEGMENT: usize = 16;

/// How far from vertical a wall may be and still count as part of a stem.
const VERTICAL_TOLERANCE_DEGREES: f64 = 15.0;

/// Total turn through a terminal, from one wall to the other. A blunt or
/// rounded terminal turns by 180 degrees; the bounds leave room for very soft
/// corners and for outlines whose walls are slightly curved.
const MIN_TURN_DEGREES: f64 = 140.0;
const MAX_TURN_DEGREES: f64 = 220.0;

/// Plausible terminal widths, as a fraction of the em.
const MIN_WIDTH_EM: f64 = 0.008;
const MAX_WIDTH_EM: f64 = 0.35;

/// How far the terminal may bulge along the stem, relative to the stem width.
/// Flared terminals turn over along a long curve, so this has to be generous;
/// arches are rejected by the ink test rather than by their depth.
const MAX_DEPTH_RATIO: f64 = 6.0;
const MIN_DEPTH_RATIO: f64 = -0.5;

/// Each wall must run at least this fraction of the stem width away from the
/// terminal. This is only a guard against reading a tiny vertical wobble as a
/// wall: the inner wall of a stem whose arch springs from just below its top
/// can be very short.
const MIN_WALL_RATIO: f64 = 0.15;

/// A stem at least this much wider (or narrower) at the terminal than one stem
/// width down the stem is flared (or tapered).
const FLARED_RATIO: f64 = 1.05;
const TAPERED_RATIO: f64 = 0.85;

/// A terminal is flat when this much of it is straight and it does not bulge
/// much. Flat terminals are blunted or angled; the rest are rounded or soft.
const FLAT_STRAIGHT_RATIO: f64 = 0.8;
const FLAT_BULGE_RATIO: f64 = 0.15;

/// A flat terminal is angled when its chord is this far from horizontal.
const ANGLED_CHORD_DEGREES: f64 = 4.0;

/// A curved terminal is soft when it contains a straight line at least this
/// fraction of its width, or when its apex is flatter than a circle of this
/// many widths. Otherwise it is rounded.
const SOFT_LINE_RATIO: f64 = 0.05;
const SOFT_APEX_RATIO: f64 = 0.9;

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

/// One end of a stem.
///
/// `side_a`, `cap` and `side_b` are consecutive pieces of the outline: from the
/// far end of one wall, around the terminal, and down the other wall. They are
/// in deslanted coordinates, so the walls are vertical and an italic terminal
/// can be measured as it was drawn. Use [`unslant`] to map them back into the
/// glyph's own coordinate system.
#[derive(Debug, Clone)]
pub struct StemTerminal {
    pub glyph: char,
    /// Whether this is the top end of the stem (rather than the bottom).
    pub top: bool,
    /// Distance between the two walls where they meet the terminal.
    pub width: f64,
    /// How far the terminal reaches beyond the ends of the walls, along the
    /// stem. Zero for a flat, blunt terminal.
    pub depth: f64,
    /// Net signed turn from one wall to the other, in degrees.
    pub turn: f64,
    pub side_a: Vec<SegmentSlice>,
    pub cap: Vec<SegmentSlice>,
    pub side_b: Vec<SegmentSlice>,
    /// Where the first wall meets the terminal.
    pub junction_a: Point,
    /// Where the terminal meets the second wall.
    pub junction_b: Point,
    /// Index of the contour the chain belongs to.
    pub contour: usize,
    /// The shear that deslants the glyph, as used for `junction_a` and
    /// `junction_b`.
    pub slant: f64,
}

impl StemTerminal {
    /// All of the segments in the chain, from the far end of `side_a` to the
    /// far end of `side_b`.
    pub fn segments(&self) -> impl Iterator<Item = SegmentSlice> + '_ {
        self.side_a
            .iter()
            .chain(self.cap.iter())
            .chain(self.side_b.iter())
            .copied()
    }
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

/// Find the stem terminals in a single glyph.
///
/// `paths` are the contours of the glyph, already with overlaps removed, and
/// `slant` is the shear coefficient of the font's slant as a tangent (see
/// [`deslant_shear`]). `upem` is only used to make the thresholds relative to
/// the size of the font.
pub fn find_stem_terminals(
    glyph: char,
    paths: &[&BezPath],
    slant: f64,
    upem: f64,
) -> Vec<StemTerminal> {
    let shear = deslant_shear(slant);
    let mut terminals = Vec::new();
    for (contour, path) in paths.iter().enumerate() {
        let segments = path.segments().collect::<Vec<_>>();
        if segments.is_empty() {
            continue;
        }
        let samples = samples(&segments, shear);
        let runs = vertical_runs(&samples);
        log::debug!(
            "{glyph}: contour {contour} has {} samples and {} vertical wall runs",
            samples.len(),
            runs.len()
        );
        if runs.len() < 2 {
            continue;
        }
        for (start, end) in wall_pairs(&samples, &runs) {
            if let Some(terminal) = terminal_between(
                glyph,
                contour,
                &segments,
                &samples,
                &runs[start],
                &runs[end],
                paths,
                slant,
                upem,
            ) {
                terminals.push(terminal);
            }
        }
    }
    terminals
}

/// Find the stem terminals in the glyphs `l`, `m` and `n` of a font.
///
/// The slant is taken from the [`SLANT`] metric when it has already been
/// computed, and otherwise measured from the whole font.
pub fn detect_stem_terminals(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &crate::Results,
) -> Result<Vec<StemTerminal>, FontquantError> {
    let upem = font.head()?.units_per_em() as f64;
    let slant = slant_of(font, location, results)?;
    let mut terminals = Vec::new();
    for glyph in ['l', 'm', 'n'] {
        let Some(bezglyph) = font.bezglyph_for_char(location, None, glyph)? else {
            continue;
        };
        // Overlapping contours (a stem drawn on top of an arch, say) would
        // otherwise offer their internal edges as walls. A font that
        // linesweeper cannot handle is left as it is: the ink test will still
        // reject the internal edges of the overlap.
        let bezglyph = bezglyph
            .remove_overlaps()
            .unwrap_or_else(|_| bezglyph.clone());
        let paths = bezglyph.iter().collect::<Vec<_>>();
        terminals.extend(find_stem_terminals(glyph, &paths, slant, upem));
    }
    Ok(terminals)
}

/// The tangent of the slant shear, `x' = x - slant * y`.
fn slant_of(
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
struct Sample {
    point: Point,
    tangent: Vec2,
    segment: usize,
    t: f64,
}

#[derive(Debug, Clone, Copy)]
struct Run {
    start: usize,
    end: usize,
    /// +1 when the outline travels up this wall, -1 when it travels down.
    direction: i8,
}

fn samples(segments: &[PathSeg], shear: Affine) -> Vec<Sample> {
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

/// Pairs of wall runs that could be the two sides of a stem end, as indices
/// into `runs`.
///
/// A flared terminal often starts with a very short near-vertical piece where
/// the flare turns over. Such a piece belongs to the terminal rather than to a
/// wall, so a pair whose wall is shorter than [`MIN_WALL_RATIO`] of the stem
/// width is extended through the neighbouring run until it reaches a real
/// wall. Different runs can lead to the same pair this way; each pair is
/// returned once.
fn wall_pairs(samples: &[Sample], runs: &[Run]) -> Vec<(usize, usize)> {
    let count = runs.len();
    let mut pairs = Vec::new();
    let mut seen = HashSet::new();
    for i in 0..count {
        let mut start = i;
        let mut end = (i + 1) % count;
        if runs[start].direction == runs[end].direction {
            continue;
        }
        for _ in 0..count {
            let width = cap_width(samples, runs[start].end, runs[end].start);
            let mut extended = false;
            if run_length(samples, runs[start].start, runs[start].end) < MIN_WALL_RATIO * width {
                let previous = (start + count - 1) % count;
                if previous != end && runs[previous].direction != runs[end].direction {
                    start = previous;
                    extended = true;
                }
            }
            if run_length(samples, runs[end].start, runs[end].end) < MIN_WALL_RATIO * width {
                let next = (end + 1) % count;
                if next != start && runs[next].direction != runs[start].direction {
                    end = next;
                    extended = true;
                }
            }
            if !extended {
                break;
            }
        }
        if seen.insert((start, end)) {
            pairs.push((start, end));
        }
    }
    pairs
}

/// Maximal cyclic runs of samples whose tangent is close to vertical.
fn vertical_runs(samples: &[Sample]) -> Vec<Run> {
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

#[allow(clippy::too_many_arguments)]
fn terminal_between(
    glyph: char,
    contour: usize,
    segments: &[PathSeg],
    samples: &[Sample],
    wall_a: &Run,
    wall_b: &Run,
    paths: &[&BezPath],
    slant: f64,
    upem: f64,
) -> Option<StemTerminal> {
    let n = samples.len();
    let a = wall_a.end;
    let b = wall_b.start;

    // The samples strictly between the two walls, leaving out their own ends.
    let mut cap = Vec::new();
    let mut i = (a + 1) % n;
    while i != b && cap.len() < n {
        cap.push(i);
        i = (i + 1) % n;
    }
    if cap.is_empty() || cap.len() >= n {
        return None;
    }

    // Net turn from the first wall to the second.
    let mut turn = 0.0;
    let mut previous = samples[a].tangent.atan2();
    for &i in cap.iter().chain(std::iter::once(&b)) {
        let current = samples[i].tangent.atan2();
        turn += cap_step(turn, previous, current);
        previous = current;
    }
    let turn_degrees = turn.to_degrees();
    if !(MIN_TURN_DEGREES..=MAX_TURN_DEGREES).contains(&turn_degrees.abs()) {
        log::debug!(
            "{glyph}: rejected cap at ({:.0},{:.0}): turn {turn_degrees:.0} deg outside {}..{}",
            samples[a].point.x,
            samples[a].point.y,
            MIN_TURN_DEGREES,
            MAX_TURN_DEGREES
        );
        return None;
    }

    let junction_a = samples[a].point;
    let junction_b = samples[b].point;
    let width = (junction_b.x - junction_a.x).abs();
    if width <= 0.0 || !(MIN_WIDTH_EM..=MAX_WIDTH_EM).contains(&(width / upem)) {
        log::debug!(
            "{glyph}: rejected cap at ({:.0},{:.0}): width {:.0} outside {}..{} em",
            junction_a.x,
            junction_a.y,
            width,
            MIN_WIDTH_EM * upem,
            MAX_WIDTH_EM * upem
        );
        return None;
    }

    let top = wall_a.direction > 0;
    let depth = if top {
        cap.iter()
            .map(|&i| samples[i].point.y)
            .fold(f64::NEG_INFINITY, f64::max)
            - junction_a.y.max(junction_b.y)
    } else {
        junction_a.y.min(junction_b.y)
            - cap
                .iter()
                .map(|&i| samples[i].point.y)
                .fold(f64::INFINITY, f64::min)
    };
    if depth > MAX_DEPTH_RATIO * width || depth < MIN_DEPTH_RATIO * width {
        log::debug!(
            "{glyph}: rejected cap at ({:.0},{:.0}): depth {:.0} outside {}..{} for width {:.0}",
            junction_a.x,
            junction_a.y,
            depth,
            MIN_DEPTH_RATIO * width,
            MAX_DEPTH_RATIO * width,
            width
        );
        return None;
    }

    let wall_a_length = run_length(samples, wall_a.start, a);
    let wall_b_length = run_length(samples, b, wall_b.end);
    if wall_a_length.min(wall_b_length) < MIN_WALL_RATIO * width {
        log::debug!(
            "{glyph}: rejected cap at ({:.0},{:.0}): walls {:.0} and {:.0} shorter than {:.0}",
            junction_a.x,
            junction_a.y,
            wall_a_length,
            wall_b_length,
            MIN_WALL_RATIO * width
        );
        return None;
    }

    // There must be ink between the walls, just inside the terminal, and none
    // just outside it. This is what rules out arches, whose middle is counter.
    let middle_x = (junction_a.x + junction_b.x) / 2.0;
    let (inside_y, outside_y) = if top {
        (
            junction_a.y.min(junction_b.y) - width * 0.25,
            junction_a.y.max(junction_b.y) + depth + width * 0.25,
        )
    } else {
        (
            junction_a.y.max(junction_b.y) + width * 0.25,
            junction_a.y.min(junction_b.y) - depth - width * 0.25,
        )
    };
    let inside = unslant(Point::new(middle_x, inside_y), slant);
    let outside = unslant(Point::new(middle_x, outside_y), slant);
    if !is_ink(paths, inside) || is_ink(paths, outside) {
        log::debug!(
            "{glyph}: rejected cap at ({:.0},{:.0}): ink test failed (inside={}, outside={})",
            junction_a.x,
            junction_a.y,
            is_ink(paths, inside),
            is_ink(paths, outside)
        );
        return None;
    }

    Some(StemTerminal {
        glyph,
        top,
        width,
        depth,
        turn: turn_degrees,
        side_a: slices_between(segments, samples, wall_a.start, a),
        cap: slices_between(segments, samples, a, b),
        side_b: slices_between(segments, samples, b, wall_b.end),
        junction_a,
        junction_b,
        contour,
        slant,
    })
}

/// The turn from one tangent to the next, as part of a running turn. Where
/// the outline reverses (at the pointed tip of a flare, say) the direction of
/// the reversal is ambiguous; take the one that keeps the running turn small,
/// so that a doubling back cancels out.
fn cap_step(turn: f64, previous: f64, current: f64) -> f64 {
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

fn signed_angle_difference(from: f64, to: f64) -> f64 {
    let mut difference = to - from;
    while difference > PI {
        difference -= 2.0 * PI;
    }
    while difference <= -PI {
        difference += 2.0 * PI;
    }
    difference
}

fn cap_width(samples: &[Sample], a: usize, b: usize) -> f64 {
    (samples[b].point.x - samples[a].point.x).abs()
}

fn run_length(samples: &[Sample], start: usize, end: usize) -> f64 {
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
fn slices_between(
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

fn is_ink(paths: &[&BezPath], point: Point) -> bool {
    paths.iter().map(|path| path.winding(point)).sum::<i32>() != 0
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

fn unit_tangent(slice: &SegmentSlice, at_end: bool) -> Option<Vec2> {
    // `segment` is already the piece described by `t0..t1`, so its own
    // parameter range is always 0..1.
    let t = if at_end { 1.0 } else { 0.0 };
    let mut tangent = segment_tangent(&slice.segment, t);
    if tangent.hypot() < 1e-9 {
        tangent = slice.segment.end() - slice.segment.start();
    }
    (tangent.hypot() > 1e-9).then(|| tangent.normalize())
}

fn corner_turn(incoming: Option<Vec2>, outgoing: Option<Vec2>) -> Option<f64> {
    let (incoming, outgoing) = (incoming?, outgoing?);
    Some(
        signed_angle_difference(incoming.atan2(), outgoing.atan2())
            .abs()
            .to_degrees(),
    )
}

/// Measurements of a terminal's shape, used to categorise it.
///
/// All angles are in degrees and all lengths are in font units. The ratios are
/// dimensionless, so the measurements do not depend on the size of the font.
#[derive(Debug, Clone, Copy)]
pub struct TerminalFeatures {
    /// Turn from the first wall into the terminal.
    pub corner_a: f64,
    /// Turn from the terminal into the second wall.
    pub corner_b: f64,
    /// How far the terminal bulges away from the chord between the two walls,
    /// as a fraction of its width. Zero for a flat terminal, about 0.39 for a
    /// semicircle once the walls' own curvature is excluded.
    pub bulge_ratio: f64,
    /// Fraction of the terminal's length that is straight, by curvature. One
    /// for a flat terminal, zero for a semicircle.
    pub straight_ratio: f64,
    /// Width at the terminal divided by the width one stem width down the
    /// stem. More than one is flared, less than one is tapered. `None` when
    /// the walls are too short to measure a profile.
    pub flare_ratio: Option<f64>,
    /// Angle of the chord between the two wall junctions, in degrees away from
    /// horizontal, measured in the glyph's own coordinates. Non-zero when the
    /// terminal is cut at an angle to the baseline.
    pub chord_angle: f64,
    /// Radius of the circle through the middle of the terminal and the points
    /// a quarter of its width either side, divided by the width. About 0.5 for
    /// a semicircle, more for a shallower arc, and infinite for a flat top.
    pub apex_radius: f64,
    /// Length of the longest straight line segment in the terminal, divided by
    /// the terminal's width. A rounded-rectangle terminal usually has a flat
    /// top drawn as a line; a semicircle has none.
    pub line_ratio: f64,
}

pub fn terminal_features(terminal: &StemTerminal) -> Option<TerminalFeatures> {
    let corner_a = corner_turn(
        terminal
            .side_a
            .last()
            .and_then(|slice| unit_tangent(slice, true)),
        terminal
            .cap
            .first()
            .and_then(|slice| unit_tangent(slice, false)),
    )?;
    let corner_b = corner_turn(
        terminal
            .cap
            .last()
            .and_then(|slice| unit_tangent(slice, true)),
        terminal
            .side_b
            .first()
            .and_then(|slice| unit_tangent(slice, false)),
    )?;
    let (bulge, straight_ratio, apex_radius) = cap_metrics(terminal)?;
    let longest_line = terminal
        .cap
        .iter()
        .filter(|slice| matches!(slice.segment, PathSeg::Line(_)))
        .map(|slice| slice.segment.start().distance(slice.segment.end()))
        .fold(0.0, f64::max);
    // Measure the chord in the glyph's own frame: an italic terminal cut
    // parallel to the baseline is horizontal, however slanted its stem is.
    let a = unslant(terminal.junction_a, terminal.slant);
    let b = unslant(terminal.junction_b, terminal.slant);
    let chord = b - a;
    Some(TerminalFeatures {
        corner_a,
        corner_b,
        bulge_ratio: bulge / terminal.width,
        straight_ratio,
        flare_ratio: flare_ratio(terminal),
        chord_angle: chord.y.atan2(chord.x.abs()).abs().to_degrees(),
        apex_radius,
        line_ratio: longest_line / terminal.width,
    })
}

/// The shape of the cap at the end of a stem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalShape {
    /// Roughly 90 degree corners and a flat cap.
    Blunted,
    /// A flat cap cut at an angle, so its corners are not at 90 degrees.
    Angled,
    /// A semicircular, or nearly semicircular, cap.
    Rounded,
    /// A rounded-rectangle cap: a flat top with rounded corners.
    Soft,
}

impl std::fmt::Display for TerminalShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TerminalShape::Blunted => "blunted",
            TerminalShape::Angled => "angled",
            TerminalShape::Rounded => "rounded",
            TerminalShape::Soft => "soft",
        })
    }
}

/// The width profile of a stem, independent of the shape of its cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StemProfile {
    /// The stem is wider at the terminal than one stem width down.
    Flared,
    /// The stem does not change width appreciably towards the terminal.
    Straight,
    /// The stem is narrower at the terminal than one stem width down.
    Tapered,
}

impl std::fmt::Display for StemProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StemProfile::Flared => "flared",
            StemProfile::Straight => "straight",
            StemProfile::Tapered => "tapered",
        })
    }
}

/// Categorise the cap of a terminal from its measurements.
///
/// The cap is blunted or angled when it is flat, and rounded or soft when it
/// is curved. The width profile of the stem is reported separately by
/// [`classify_stem_profile`].
pub fn classify_terminal(terminal: &StemTerminal) -> Option<TerminalShape> {
    let features = terminal_features(terminal)?;
    let flat =
        features.straight_ratio >= FLAT_STRAIGHT_RATIO && features.bulge_ratio <= FLAT_BULGE_RATIO;
    if flat {
        return Some(if features.chord_angle > ANGLED_CHORD_DEGREES {
            TerminalShape::Angled
        } else {
            TerminalShape::Blunted
        });
    }
    let flat_topped =
        features.line_ratio >= SOFT_LINE_RATIO || features.apex_radius > SOFT_APEX_RATIO;
    Some(if flat_topped {
        TerminalShape::Soft
    } else {
        TerminalShape::Rounded
    })
}

/// Categorise the width profile of a stem: flared, tapered or straight.
///
/// `None` when the walls are too short to measure a profile.
pub fn classify_stem_profile(terminal: &StemTerminal) -> Option<StemProfile> {
    let flare = terminal_features(terminal)?.flare_ratio?;
    Some(if flare >= FLARED_RATIO {
        StemProfile::Flared
    } else if flare <= TAPERED_RATIO {
        StemProfile::Tapered
    } else {
        StemProfile::Straight
    })
}

fn most_common<T: Eq + std::hash::Hash + Copy>(items: impl IntoIterator<Item = T>) -> Option<T> {
    let mut counts = std::collections::HashMap::new();
    for item in items {
        *counts.entry(item).or_insert(0usize) += 1;
    }
    let mut best = None;
    let mut tied = false;
    for (item, count) in counts {
        match best {
            None => best = Some((item, count)),
            Some((_, best_count)) if count > best_count => {
                best = Some((item, count));
                tied = false;
            }
            Some((_, best_count)) if count == best_count => tied = true,
            Some(_) => {}
        }
    }
    if tied {
        None
    } else {
        best.map(|(item, _)| item)
    }
}

/// The cap shape that occurs most often, or `None` when there is a tie or
/// nothing to count.
pub fn most_common_shape(shapes: impl IntoIterator<Item = TerminalShape>) -> Option<TerminalShape> {
    most_common(shapes)
}

/// The stem profile that occurs most often among the terminals that have one,
/// or `None` when there is a tie or nothing to measure.
pub fn most_common_profile(terminals: &[StemTerminal]) -> Option<StemProfile> {
    most_common(terminals.iter().filter_map(classify_stem_profile))
}

/// The largest distance of the terminal from the chord between its two ends,
/// and the fraction of its length that is straight.
fn cap_metrics(terminal: &StemTerminal) -> Option<(f64, f64, f64)> {
    let start = terminal.junction_a;
    let chord = terminal.junction_b - start;
    let width = chord.hypot();
    if width <= 0.0 {
        return None;
    }
    let normal = Vec2::new(-chord.y, chord.x) / width;
    let points = flatten(&chain_path(&terminal.cap), width * 0.002);
    if points.len() < 2 {
        return None;
    }
    let bulge = points
        .iter()
        .map(|point| (*point - start).dot(normal).abs())
        .fold(0.0, f64::max);

    // Curvature at each interior vertex, in radians per font unit.
    let curvature = (1..points.len() - 1)
        .map(|i| {
            let incoming = points[i] - points[i - 1];
            let outgoing = points[i + 1] - points[i];
            let length = (incoming.hypot() + outgoing.hypot()) / 2.0;
            let turn = signed_angle_difference(incoming.atan2(), outgoing.atan2()).abs();
            if length > 0.0 { turn / length } else { 0.0 }
        })
        .collect::<Vec<_>>();
    let mut straight = 0.0;
    let mut total = 0.0;
    for (i, pair) in points.windows(2).enumerate() {
        let length = pair[1].distance(pair[0]);
        total += length;
        // A piece is straight when neither of its ends is on a curve.
        let mut piece_curvature = 0.0f64;
        if i > 0 {
            piece_curvature = piece_curvature.max(curvature[i - 1]);
        }
        if i + 1 < curvature.len() {
            piece_curvature = piece_curvature.max(curvature[i + 1]);
        }
        if piece_curvature < 0.3 / width {
            straight += length;
        }
    }
    if total <= 0.0 {
        return None;
    }
    let radius = apex_radius(&points, start, chord, width).unwrap_or(f64::INFINITY);
    Some((bulge, straight / total, radius / width))
}

/// Radius of the circle through the point furthest from the chord and the
/// points a quarter of the width either side of it.
fn apex_radius(points: &[Point], start: Point, chord: Vec2, width: f64) -> Option<f64> {
    let along = chord / width;
    let normal = Vec2::new(-along.y, along.x);
    let (mut apex, mut apex_distance) = (None, 0.0f64);
    for point in points {
        let distance = (*point - start).dot(normal).abs();
        if distance > apex_distance {
            apex = Some(*point);
            apex_distance = distance;
        }
    }
    if apex_distance <= 1e-9 {
        return None;
    }
    let apex = apex?;
    let apex_along = (apex - start).dot(along);
    let closest = |target: f64| {
        points.iter().copied().min_by(|a, b| {
            ((*a - start).dot(along) - target)
                .abs()
                .total_cmp(&((*b - start).dot(along) - target).abs())
        })
    };
    let low = closest(apex_along - width * 0.25)?;
    let high = closest(apex_along + width * 0.25)?;
    let (a, b, c) = (low.distance(apex), apex.distance(high), high.distance(low));
    let area = (apex - low).cross(high - low).abs() / 2.0;
    if area <= 1e-9 {
        return Some(f64::INFINITY);
    }
    Some(a * b * c / (4.0 * area))
}

/// Width at the terminal divided by the width further down the stem.
fn flare_ratio(terminal: &StemTerminal) -> Option<f64> {
    let side_a = wall_profile(&terminal.side_a, true);
    let side_b = wall_profile(&terminal.side_b, false);
    let reachable = side_a.last()?.0.min(side_b.last()?.0);
    let width = terminal.width;
    if reachable < 0.75 * width {
        return None;
    }
    let far = reachable.min(width);
    let far_width = width_between(&side_a, &side_b, far)?;
    (far_width > 0.0).then(|| width / far_width)
}

/// Distance between the two walls, `distance` font units back from the
/// terminal along the stem. `None` when either wall is too short.
pub fn width_at(terminal: &StemTerminal, distance: f64) -> Option<f64> {
    let side_a = wall_profile(&terminal.side_a, true);
    let side_b = wall_profile(&terminal.side_b, false);
    width_between(&side_a, &side_b, distance)
}

/// Points along a wall, with their distance from the terminal.
fn wall_profile(slices: &[SegmentSlice], from_junction: bool) -> Vec<(f64, Point)> {
    let mut points = flatten(&chain_path(slices), 0.5);
    if from_junction {
        points.reverse();
    }
    let mut distance = 0.0;
    let mut profile = Vec::with_capacity(points.len());
    for (i, point) in points.iter().enumerate() {
        if i > 0 {
            distance += point.distance(points[i - 1]);
        }
        profile.push((distance, *point));
    }
    profile
}

fn point_at(profile: &[(f64, Point)], distance: f64) -> Option<Point> {
    let last = profile.last()?;
    if distance > last.0 {
        return None;
    }
    let index = profile.partition_point(|(d, _)| *d < distance);
    if index == 0 {
        return Some(profile[0].1);
    }
    let (d0, p0) = profile[index - 1];
    let (d1, p1) = profile[index];
    let t = if d1 > d0 {
        (distance - d0) / (d1 - d0)
    } else {
        0.0
    };
    Some(p0.lerp(p1, t))
}

fn width_between(side_a: &[(f64, Point)], side_b: &[(f64, Point)], distance: f64) -> Option<f64> {
    let a = point_at(side_a, distance)?;
    let b = point_at(side_b, distance)?;
    Some((a.x - b.x).abs())
}

fn flatten(path: &BezPath, tolerance: f64) -> Vec<Point> {
    let mut points = Vec::new();
    kurbo::flatten(path.iter(), tolerance, |element| match element {
        kurbo::PathEl::MoveTo(point) | kurbo::PathEl::LineTo(point) => points.push(point),
        _ => {}
    });
    points
}

quantifier!(
    STEM_TERMINALS,
    "structure/stem_terminals",
    r#"The shape of the ends of the stems of the letters 'l', 'm' and 'n', as the
most common of blunted (roughly 90 degree corners), angled (a flat cap cut at
an angle), rounded (a semicircular cap) and soft (a rounded-rectangle cap).
Fonts whose terminals cannot be found, such as most serif designs, are not
given a value. Whether the stem widens or narrows towards the terminal is
reported separately by `structure/stem_profile`.
"#,
    MetricValue::String("blunted".to_string())
);

quantifier!(
    STEM_PROFILE,
    "structure/stem_profile",
    r#"The width profile of the ends of the stems of the letters 'l', 'm' and 'n',
as the most common of flared (the stem widens towards its end), straight and
tapered (the stem narrows towards its end). Fonts whose stem widths cannot be
measured, such as most serif designs, are not given a value.
"#,
    MetricValue::String("straight".to_string())
);

pub(crate) fn gather_stem_terminals(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let terminals = detect_stem_terminals(font, location, results)?;
    let shapes = terminals.iter().filter_map(classify_terminal);
    if let Some(shape) = most_common_shape(shapes) {
        results.add_metric(&STEM_TERMINALS, MetricValue::String(shape.to_string()));
    }
    if let Some(profile) = most_common_profile(&terminals) {
        results.add_metric(&STEM_PROFILE, MetricValue::String(profile.to_string()));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // it's a test suite
mod tests {
    use super::*;
    use kurbo::{Rect, RoundedRect};

    fn find(paths: &[BezPath], slant: f64, upem: f64) -> Vec<StemTerminal> {
        let paths = paths.iter().collect::<Vec<_>>();
        find_stem_terminals('l', &paths, slant, upem)
    }

    fn polygon(points: &[(f64, f64)]) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(points[0]);
        for &point in &points[1..] {
            path.line_to(point);
        }
        path.close_path();
        path
    }

    #[test]
    fn blunt_rectangle_has_two_terminals() {
        let glyph = Rect::new(0.0, 0.0, 100.0, 1000.0).to_path(0.01);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        for terminal in &terminals {
            assert!((terminal.width - 100.0).abs() < 1.0, "{terminal:?}");
            assert!(terminal.depth.abs() < 1.0, "{terminal:?}");
            assert!((terminal.turn.abs() - 180.0).abs() < 1.0, "{terminal:?}");
        }
        assert!(terminals.iter().any(|terminal| terminal.top));
        assert!(terminals.iter().any(|terminal| !terminal.top));
    }

    #[test]
    fn rounded_terminal_has_depth() {
        let glyph = RoundedRect::new(0.0, 0.0, 100.0, 1000.0, 50.0).to_path(0.01);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        for terminal in &terminals {
            // The walls continue a little way into the semicircle before the
            // tangent leaves the vertical tolerance, so the measured terminal
            // is slightly narrower and shallower than the full circle.
            assert!(
                terminal.width > 90.0 && terminal.width <= 100.0,
                "{terminal:?}"
            );
            assert!(terminal.depth > 0.5 * terminal.width / 2.0, "{terminal:?}");
            assert!(terminal.turn.abs() > 140.0, "{terminal:?}");
        }
    }

    #[test]
    fn angled_terminal_is_found() {
        // A stem cut off at 45 degrees.
        let glyph = polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 900.0), (0.0, 1000.0)]);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        let top = terminals.iter().find(|terminal| terminal.top).unwrap();
        assert!((top.width - 100.0).abs() < 1.0, "{top:?}");
        assert!(top.depth.abs() < 1.0, "{top:?}");
    }

    fn shapes(terminals: &[StemTerminal]) -> Vec<Option<TerminalShape>> {
        terminals.iter().map(classify_terminal).collect()
    }

    #[test]
    fn blunt_rectangle_is_blunted() {
        let glyph = Rect::new(0.0, 0.0, 100.0, 1000.0).to_path(0.01);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        assert!(
            shapes(&terminals)
                .iter()
                .all(|shape| *shape == Some(TerminalShape::Blunted))
        );
    }

    #[test]
    fn angled_cut_is_angled() {
        let glyph = polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 900.0), (0.0, 1000.0)]);
        let terminals = find(&[glyph], 0.0, 1000.0);
        let top = terminals.iter().find(|terminal| terminal.top).unwrap();
        assert_eq!(classify_terminal(top), Some(TerminalShape::Angled));
    }

    #[test]
    fn stadium_is_rounded() {
        let glyph = RoundedRect::new(0.0, 0.0, 100.0, 1000.0, 50.0).to_path(0.01);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        assert!(
            shapes(&terminals)
                .iter()
                .all(|shape| *shape == Some(TerminalShape::Rounded))
        );
    }

    #[test]
    fn rounded_rectangle_is_soft() {
        let glyph = RoundedRect::new(0.0, 0.0, 100.0, 1000.0, 20.0).to_path(0.01);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2);
        assert!(
            shapes(&terminals)
                .iter()
                .all(|shape| *shape == Some(TerminalShape::Soft))
        );
    }

    #[test]
    fn widened_stem_is_flared() {
        // A stem of constant width with a short flare at the top.
        let glyph = polygon(&[
            (-40.0, 1000.0),
            (140.0, 1000.0),
            (100.0, 800.0),
            (100.0, 0.0),
            (0.0, 0.0),
            (0.0, 800.0),
        ]);
        let terminals = find(&[glyph], 0.0, 1000.0);
        let top = terminals.iter().find(|terminal| terminal.top).unwrap();
        assert_eq!(classify_terminal(top), Some(TerminalShape::Blunted));
        assert_eq!(classify_stem_profile(top), Some(StemProfile::Flared));
    }

    #[test]
    fn narrowed_stem_is_tapered() {
        let glyph = polygon(&[
            (20.0, 1000.0),
            (80.0, 1000.0),
            (100.0, 800.0),
            (100.0, 0.0),
            (0.0, 0.0),
            (0.0, 800.0),
        ]);
        let terminals = find(&[glyph], 0.0, 1000.0);
        let top = terminals.iter().find(|terminal| terminal.top).unwrap();
        assert_eq!(classify_terminal(top), Some(TerminalShape::Blunted));
        assert_eq!(classify_stem_profile(top), Some(StemProfile::Tapered));
    }

    #[test]
    fn most_common_needs_a_winner() {
        use TerminalShape::*;
        assert_eq!(
            most_common_shape([Blunted, Blunted, Rounded]),
            Some(Blunted)
        );
        assert_eq!(most_common_shape([Blunted, Rounded]), None);
        assert_eq!(most_common_shape([]), None);
    }

    fn metric_value(font_data: &[u8], key: &str) -> Option<String> {
        let font = skrifa::FontRef::new(font_data).unwrap();
        let mut results = crate::Results::new();
        gather_stem_terminals(&font, &[], &mut results).unwrap();
        results.get(key).map(|metric| metric.1.to_string())
    }

    #[test]
    fn metric_shapes_for_test_fonts() {
        let open_sans = font("OpenSans-VariableFont_wdth,wght.ttf");
        assert_eq!(
            metric_value(&open_sans, &STEM_TERMINALS.name).as_deref(),
            Some("blunted")
        );
        assert_eq!(
            metric_value(&open_sans, &STEM_PROFILE.name).as_deref(),
            Some("straight")
        );
        let delius = font("DeliusUnicase-Regular.ttf");
        assert_eq!(
            metric_value(&delius, &STEM_TERMINALS.name).as_deref(),
            Some("rounded")
        );
        assert_eq!(
            metric_value(&delius, &STEM_PROFILE.name).as_deref(),
            Some("straight")
        );
        let farro = font("Farro-Regular.ttf");
        assert_eq!(
            metric_value(&farro, &STEM_TERMINALS.name).as_deref(),
            Some("blunted")
        );
        assert_eq!(
            metric_value(&farro, &STEM_PROFILE.name).as_deref(),
            Some("flared")
        );
    }

    #[test]
    fn italic_stem_is_deslanted() {
        let upright = Rect::new(0.0, 0.0, 100.0, 1000.0).to_path(0.01);
        let shear = 0.2;
        let italic = Affine::new([1.0, 0.0, shear, 1.0, 0.0, 0.0]) * &upright;
        let terminals = find(&[italic], shear, 1000.0);
        assert_eq!(terminals.len(), 2);
        for terminal in &terminals {
            // Width is measured perpendicular to the (deslanted) stem.
            assert!((terminal.width - 100.0).abs() < 1.0, "{terminal:?}");
        }
    }

    #[test]
    fn arch_is_not_a_terminal() {
        // An "n" with no separate stem tops: two feet, joined by a wide arch.
        // The arch turns through 180 degrees like a terminal, but it is far
        // wider than a stem.
        let glyph = polygon(&[
            (0.0, 0.0),
            (0.0, 600.0),
            (600.0, 600.0),
            (600.0, 0.0),
            (500.0, 0.0),
            (500.0, 500.0),
            (100.0, 500.0),
            (100.0, 0.0),
        ]);
        let terminals = find(&[glyph], 0.0, 1000.0);
        assert_eq!(terminals.len(), 2, "{terminals:#?}");
        assert!(terminals.iter().all(|terminal| !terminal.top));
    }

    fn font(name: &str) -> Vec<u8> {
        let path = format!("{}/../tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).unwrap()
    }

    fn counts(font_data: &[u8]) -> (usize, usize, usize) {
        let font = skrifa::FontRef::new(font_data).unwrap();
        let terminals = detect_stem_terminals(&font, &[], &crate::Results::new()).unwrap();
        ['l', 'm', 'n']
            .map(|glyph| {
                terminals
                    .iter()
                    .filter(|terminal| terminal.glyph == glyph)
                    .count()
            })
            .into()
    }

    #[test]
    fn open_sans_terminal_counts() {
        let data = font("OpenSans-VariableFont_wdth,wght.ttf");
        assert_eq!(counts(&data), (2, 4, 3));
    }

    #[test]
    fn roboto_flex_terminal_counts() {
        let data = font("RobotoFlex-Var.ttf");
        assert_eq!(counts(&data), (2, 4, 3));
    }

    /// These fonts flare out to a short, almost vertical piece at the very end
    /// of each stem, sometimes with a pointed tip where the outline doubles
    /// back. That piece is part of the terminal, not a wall.
    #[test]
    fn flared_terminal_counts() {
        for name in [
            "Asul-Regular.ttf",
            "Marcellus-Regular.ttf",
            "Smythe-Regular.ttf",
            "RedRose[wght].ttf",
        ] {
            let data = font(name);
            assert_eq!(counts(&data), (2, 4, 3), "{name}");
            assert_eq!(
                metric_value(&data, &STEM_PROFILE.name).as_deref(),
                Some("flared"),
                "{name}"
            );
        }
    }

    fn slices_to_path(slices: &[SegmentSlice]) -> BezPath {
        let mut path = BezPath::new();
        if let Some(first) = slices.first() {
            path.move_to(first.segment.start());
            for slice in slices {
                path.push(slice.segment.as_path_el());
            }
        }
        path
    }

    fn glyph_svg(font: &skrifa::FontRef, glyph: char) -> String {
        let Some(bezglyph) = font.bezglyph_for_char(&[], None, glyph).ok().flatten() else {
            return String::new();
        };
        let Ok(bezglyph) = bezglyph.remove_overlaps() else {
            return String::new();
        };
        let paths = bezglyph.iter().collect::<Vec<_>>();
        let upem = font
            .head()
            .map(|head| head.units_per_em() as f64)
            .unwrap_or(1000.0);
        let slant = slant_of(font, &[], &crate::Results::new()).unwrap_or(0.0);
        let terminals = find_stem_terminals(glyph, &paths, slant, upem);
        let bbox = bezglyph
            .bbox()
            .unwrap_or(kurbo::Rect::new(0.0, 0.0, upem, upem));
        let pad = bbox.height() * 0.05;
        let view = format!(
            "{} {} {} {}",
            bbox.x0 - pad,
            -bbox.y1 - pad,
            bbox.width() + 2.0 * pad,
            bbox.height() + 2.0 * pad
        );
        let mut svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{view}' width='240' height='240'>             <g transform='scale(1,-1)'>"
        );
        for path in &paths {
            let mut data = String::new();
            for element in path.iter() {
                match element {
                    kurbo::PathEl::MoveTo(p) => data.push_str(&format!("M{},{} ", p.x, p.y)),
                    kurbo::PathEl::LineTo(p) => data.push_str(&format!("L{},{} ", p.x, p.y)),
                    kurbo::PathEl::QuadTo(c, p) => {
                        data.push_str(&format!("Q{},{} {},{} ", c.x, c.y, p.x, p.y))
                    }
                    kurbo::PathEl::CurveTo(c1, c2, p) => data.push_str(&format!(
                        "C{},{} {},{} {},{} ",
                        c1.x, c1.y, c2.x, c2.y, p.x, p.y
                    )),
                    kurbo::PathEl::ClosePath => data.push_str("Z "),
                }
            }
            svg.push_str(&format!(
                "<path d='{data}' fill='none' stroke='#cccccc' stroke-width='{}'/>",
                upem / 120.0
            ));
        }
        for (i, terminal) in terminals.iter().enumerate() {
            for (slices, colour) in [
                (&terminal.side_a, "#3060d0"),
                (&terminal.cap, "#e04040"),
                (&terminal.side_b, "#30a060"),
            ] {
                let path = unslant_path(&slices_to_path(slices), slant);
                let mut data = String::new();
                for element in path.iter() {
                    match element {
                        kurbo::PathEl::MoveTo(p) => data.push_str(&format!("M{},{} ", p.x, p.y)),
                        kurbo::PathEl::LineTo(p) => data.push_str(&format!("L{},{} ", p.x, p.y)),
                        kurbo::PathEl::QuadTo(c, p) => {
                            data.push_str(&format!("Q{},{} {},{} ", c.x, c.y, p.x, p.y))
                        }
                        kurbo::PathEl::CurveTo(c1, c2, p) => data.push_str(&format!(
                            "C{},{} {},{} {},{} ",
                            c1.x, c1.y, c2.x, c2.y, p.x, p.y
                        )),
                        kurbo::PathEl::ClosePath => {}
                    }
                }
                svg.push_str(&format!(
                    "<path d='{data}' fill='none' stroke='{colour}' stroke-width='{}'/>",
                    upem / 40.0
                ));
            }
            let point = unslant(terminal.junction_a, slant);
            svg.push_str(&format!(
                "<circle cx='{}' cy='{}' r='{}' fill='#e04040'/>",
                point.x,
                point.y,
                upem / 90.0
            ));
            svg.push_str(&format!(
                "<text x='{}' y='{}' font-size='{}' fill='#444444' \
                 transform='translate({}, {}) scale(1,-1)'>\
                 {i}: {} {:.0}deg w{:.0}</text>",
                point.x,
                -point.y,
                upem / 45.0,
                point.x + upem / 60.0,
                -point.y,
                classify_terminal(terminal)
                    .map(|shape| shape.to_string())
                    .unwrap_or_else(|| "?".to_string()),
                terminal.turn,
                terminal.width
            ));
        }
        svg.push_str("</g></svg>");
        svg
    }

    /// Not a test of the quantifier: write the detected chains next to the
    /// glyphs so they can be inspected by eye. Run with
    /// `cargo test -p fontquant-lib --lib render_debug_svgs -- --ignored`
    /// and open `target/stem-terminals/index.html`.
    #[test]
    #[ignore = "writes debug SVGs instead of asserting anything"]
    fn render_debug_svgs() {
        let mut fonts = vec![
            font("OpenSans-VariableFont_wdth,wght.ttf"),
            font("RobotoFlex-Var.ttf"),
            font("Ysabeau[wght].ttf"),
            font("BodoniModa_18pt-Italic.ttf"),
        ];
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../output");
        for name in [
            "ABeeZee-Regular.ttf",
            "Asul-Regular.ttf",
            "Marcellus-Regular.ttf",
            "Smythe-Regular.ttf",
            "RedRose[wght].ttf",
            "Poppins-Regular.ttf",
            "Inter-Italic[opsz,wght].ttf",
            "Baloo2[wght].ttf",
            "Comfortaa[wght].ttf",
            "Fredoka[wdth,wght].ttf",
            "ArchivoNarrow-Italic[wght].ttf",
            "Asap[wdth,wght].ttf",
        ] {
            if let Ok(data) = std::fs::read(output.join(name)) {
                fonts.push(data);
            }
        }
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/stem-terminals");
        std::fs::create_dir_all(&directory).unwrap();
        let mut html = String::from(
            "<!doctype html><meta charset=utf-8><style>             body{font-family:sans-serif}.grid{display:flex;flex-wrap:wrap}             .cell{border:1px solid #eee;margin:4px;text-align:center}             svg{background:#fafafa}</style><div class=grid>",
        );
        for data in &fonts {
            let Ok(font) = skrifa::FontRef::new(data) else {
                continue;
            };
            for glyph in ['l', 'm', 'n'] {
                html.push_str(&format!(
                    "<div class=cell><div>{glyph}</div>{}</div>",
                    glyph_svg(&font, glyph)
                ));
            }
        }
        html.push_str("</div>");
        std::fs::write(directory.join("index.html"), html).unwrap();
    }
}
