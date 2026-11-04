//! Detection and classification of serifs.
//!
//! A serif is the widening at the end of a stem that a serif typeface adds to
//! finish it off. This module finds the serifs on the letters `I` and `H` and
//! describes their shape.
//!
//! # How it works
//!
//! Serifs are found from the stem rather than from the outline alone. The
//! outline of a serif font is full of vertical pieces — the ends of the serifs
//! themselves look exactly like the walls of a stem — so pairing walls without
//! knowing where the stem is gives answers that mean different things in
//! different fonts.
//!
//! The stems are found first, by scanning across the glyph at a range of
//! heights. The narrowest interval that the scan lines keep meeting is the
//! thickness of a stem, and the intervals of about that width, taken together,
//! are the stems. `I` has one; `H` has two, and its crossbar shows up as an
//! interval that is too wide to be a stem.
//!
//! With a stem in hand, each end of it is a serif. The outline from the point
//! where the left wall leaves the vertical, around the end of the stem, to the
//! same point on the right is the **serif chain**: a contiguous slice of the
//! contour, just as a stem terminal's chain is. Its two ends are the serif's
//! **shoulders**.
//!
//! # Classification
//!
//! The shapes that type designers distinguish come down to two measurements:
//! how **thick** the serif is, and what the outline does **at the shoulder**,
//! where the serif joins the stem.
//!
//! A serif that is only a sliver of its stem's thickness is a **hairline**,
//! whatever its shoulder looks like: whether a serif is a hairline is a matter
//! of height, not of shape.
//!
//! Otherwise the shoulder decides:
//!
//! * a serif that sits flat, so that the outline runs along the top of the
//!   serif without dropping, is a **slab** (the corner may be rounded);
//! * an outline that leaves the stem at an angle but stays straight is a
//!   **wedge**;
//! * an outline that curves away from the stem over a distance is
//!   **bracketed**.
//!
//! Two further questions are asked separately, because they cut across the
//! shape:
//!
//! * is the serif **asymmetric**, i.e. does it have a different profile on the
//!   left and the right? and
//! * is it **bifurcated**, i.e. is the middle of its outer edge raised, so that
//!   the serif has two lobes?
//!
//! They fill in the `structure/serif_shape`, `structure/serif_asymmetric` and
//! `structure/serif_bifurcated` metrics.

use kurbo::{Affine, BezPath, PathSeg, Point, Rect, Vec2};
use read_fonts::TableProvider as _;

use crate::{MetricValue, error::FontquantError, monkeypatching::MakeBezGlyphs as _, quantifier};

use super::outline::{
    Run, Sample, SegmentSlice, cap_step, chain_path, deslant_shear, flatten, is_ink, samples,
    slant_of, slices_between, unslant, vertical_runs,
};

/// The glyphs used to measure serifs.
///
/// `I` is the cleanest reference for a serif in any font: one stem, a serif at
/// each end and nothing else. `H` adds four more serifs, and a crossbar that
/// the stem finder has to see past.
const PROBE_GLYPHS: [char; 2] = ['I', 'H'];

/// How many horizontal lines are scanned across the glyph.
const SCAN_STEPS: usize = 48;

/// An ink interval no wider than this many stem widths is a cross section of a
/// stem. Anything wider is a serif, a counter or the whole glyph.
const STEM_INTERVAL_RATIO: f64 = 1.2;

/// A serif sticks out at least this far past the stem, as a fraction of the
/// stem width. Anything less is a stem that merely widens.
const MIN_OVERHANG: f64 = 0.06;

/// Serifs turn by about 180 degrees, exactly as the cap of a stem terminal
/// does.
const MIN_TURN_DEGREES: f64 = 140.0;
const MAX_TURN_DEGREES: f64 = 220.0;

/// How close to the edge of the glyph a serif has to reach, as a fraction of
/// the glyph height. Serifs make the cap height and the baseline, so a run that
/// stops well short of them is a crossbar or the top of a counter.
const EDGE_TOLERANCE: f64 = 0.02;

/// The fraction of a serif's overhang over which its shoulder is measured.
const TRANSITION_SPAN: f64 = 0.5;

/// A serif whose outer edge is raised in the middle by this fraction of the
/// serif's own thickness is bifurcated.
const BIFURCATION_RATIO: f64 = 0.15;

/// A serif thinner than this fraction of the stem width is a hairline.
const HAIRLINE_RATIO: f64 = 0.25;

/// How far the shoulder may bow away from a straight line before it counts as
/// a fillet rather than a straight edge.
const CURVED_SHOULDER: f64 = 0.1;

/// A serif whose shoulder drops less than this fraction of the serif's height,
/// or which leaves the stem within [`FLAT_SHOULDER_DEGREES`] of horizontal, is
/// sitting flat on the stem, which makes it a slab.
const FLAT_SHOULDER_DROP: f64 = 0.15;
const FLAT_SHOULDER_DEGREES: f64 = 5.0;

/// Two sides of a serif whose shoulders bow away from a straight line by
/// amounts differing more than this are asymmetric. A serif that is bracketed
/// on one side and flat on the other is the clearest case: one shoulder
/// deviates and the other does not.
const ASYMMETRY_DEVIATION: f64 = 0.05;

/// One serif: the outline from where one stem wall leaves the vertical, around
/// the end of the stem, to the same point on the other side.
///
/// `chain` is in deslanted coordinates and always runs from the left shoulder
/// to the right one, whichever end of the stem the serif is on.
#[derive(Debug, Clone)]
pub struct Serif {
    pub glyph: char,
    /// Whether this is a serif at the top of the stem (rather than the bottom).
    pub top: bool,
    /// Index of the contour the serif belongs to.
    pub contour: usize,
    /// Distance between the stem walls that the serif joins.
    pub stem_width: f64,
    /// The outer edge of each wall of the stem, in deslanted coordinates.
    pub stem_left: f64,
    pub stem_right: f64,
    /// Distance between the outermost points of the serif.
    pub width: f64,
    /// Thickness of the part of the serif that sticks out past the stem,
    /// measured along the stem.
    pub height: f64,
    /// Thickness of the serif at its outermost points, which is what makes a
    /// serif a hairline.
    pub tip_thickness: f64,
    /// Width of the gap in the middle of the outer edge, as a fraction of
    /// `width`, or zero when the outer edge is continuous.
    pub bifurcation: f64,
    /// Where the left wall of the stem leaves the vertical.
    pub junction_a: Point,
    /// Where the right wall of the stem leaves the vertical.
    pub junction_b: Point,
    pub chain: Vec<SegmentSlice>,
    /// The shear that deslants the glyph, as used for the chain.
    pub slant: f64,
}

/// Find the serifs in a single glyph.
pub fn find_serifs(glyph: char, paths: &[&BezPath], slant: f64, upem: f64) -> Vec<Serif> {
    let shear = deslant_shear(slant);
    let outline = contours(paths, shear, upem / 4000.0);
    let Some(bbox) = outline_bbox(&outline) else {
        return Vec::new();
    };
    let stems = find_stems(&outline, bbox, upem);
    if stems.is_empty() {
        return Vec::new();
    }
    let mut serifs = Vec::new();
    for (contour, path) in paths.iter().enumerate() {
        let segments = path.segments().collect::<Vec<_>>();
        if segments.is_empty() {
            continue;
        }
        let samples = samples(&segments, shear);
        let runs = vertical_runs(&samples);
        if runs.is_empty() {
            continue;
        }
        for stem in &stems {
            for top in [true, false] {
                if let Some(serif) = serif_at(
                    glyph, contour, &segments, &samples, &runs, stem, top, bbox, paths, slant,
                ) {
                    serifs.push(serif);
                }
            }
        }
    }
    serifs
}

/// Find the serifs of a font.
pub fn detect_serif_list(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &crate::Results,
) -> Result<Vec<Serif>, FontquantError> {
    let upem = font.head()?.units_per_em() as f64;
    let slant = slant_of(font, location, results)?;
    let mut serifs = Vec::new();
    for glyph in PROBE_GLYPHS {
        let Some(bezglyph) = font.bezglyph_for_char(location, None, glyph)? else {
            continue;
        };
        let bezglyph = bezglyph
            .remove_overlaps()
            .unwrap_or_else(|_| bezglyph.clone());
        let paths = bezglyph.iter().collect::<Vec<_>>();
        serifs.extend(find_serifs(glyph, &paths, slant, upem));
    }
    Ok(serifs)
}

/// Find the serifs of a font and fill in the serif metrics.
pub fn detect_serifs(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    results: &mut crate::Results,
) -> Result<(), FontquantError> {
    let serifs = detect_serif_list(font, location, results)?;
    results.add_metric(
        &SERIF_SHAPE,
        MetricValue::String(
            most_common_shape(serifs.iter().filter_map(classify_serif))
                .map(|shape| shape.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
    );
    results.add_metric(
        &SERIF_ASYMMETRIC,
        MetricValue::Boolean(
            most_common_of(serifs.iter().filter_map(is_asymmetric)).unwrap_or(false),
        ),
    );
    results.add_metric(
        &SERIF_BIFURCATED,
        MetricValue::Boolean(
            most_common_of(serifs.iter().filter_map(is_bifurcated)).unwrap_or(false),
        ),
    );
    Ok(())
}

/// A vertical stroke of the glyph, in deslanted coordinates.
#[derive(Debug, Clone, Copy)]
struct Stem {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

/// The glyph's contours as closed polylines, in deslanted coordinates.
fn contours(paths: &[&BezPath], shear: Affine, tolerance: f64) -> Vec<Vec<Point>> {
    paths
        .iter()
        .map(|path| {
            let mut points = flatten(&(shear * *path), tolerance);
            if let (Some(first), Some(last)) = (points.first().copied(), points.last().copied())
                && first != last
            {
                points.push(first);
            }
            points
        })
        .filter(|points| points.len() > 1)
        .collect()
}

fn outline_bbox(outline: &[Vec<Point>]) -> Option<Rect> {
    let mut bbox: Option<Rect> = None;
    for point in outline.iter().flatten() {
        let corner = Rect::new(point.x, point.y, point.x, point.y);
        bbox = Some(bbox.map_or(corner, |rect| rect.union(corner)));
    }
    bbox
}

/// The ink of the glyph along a horizontal line, as `(left, right)` pairs.
fn intervals_at(outline: &[Vec<Point>], y: f64) -> Vec<(f64, f64)> {
    let mut crossings = Vec::new();
    for contour in outline {
        for pair in contour.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if (a.y <= y) == (b.y <= y) {
                continue;
            }
            let t = (y - a.y) / (b.y - a.y);
            crossings.push(a.x + t * (b.x - a.x));
        }
    }
    crossings.sort_by(f64::total_cmp);
    (0..crossings.len() / 2)
        .map(|pair| (crossings[pair * 2], crossings[pair * 2 + 1]))
        .filter(|(left, right)| right > left)
        .collect()
}

fn median(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// The stems of a glyph, found by scanning across it.
///
/// The narrowest interval that the scan lines keep meeting is the thickness of
/// a stem; the intervals of about that width are its cross sections, and
/// grouping those by position gives the stems themselves.
fn find_stems(outline: &[Vec<Point>], bbox: Rect, upem: f64) -> Vec<Stem> {
    let height = bbox.height();
    if height <= 0.0 {
        return Vec::new();
    }
    let mut scans = Vec::with_capacity(SCAN_STEPS + 1);
    for step in 0..=SCAN_STEPS {
        // Stay clear of the very top and bottom, where a serif tapers away.
        let y = bbox.y0 + height * (0.06 + 0.88 * step as f64 / SCAN_STEPS as f64);
        scans.push((y, intervals_at(outline, y)));
    }
    let mut widths = scans
        .iter()
        .flat_map(|(_, intervals)| intervals.iter().map(|(left, right)| right - left))
        .collect::<Vec<_>>();
    if widths.is_empty() {
        return Vec::new();
    }
    widths.sort_by(f64::total_cmp);
    // A low percentile rather than the minimum, so that a stray hairline at one
    // height does not decide the answer.
    let stem_width = widths[widths.len() / 5];
    if stem_width < upem * 0.004 {
        return Vec::new();
    }

    let mut sections = Vec::new();
    for (y, intervals) in &scans {
        for (left, right) in intervals {
            if right - left <= stem_width * STEM_INTERVAL_RATIO {
                sections.push((*y, *left, *right));
            }
        }
    }
    sections.sort_by(|a, b| a.1.total_cmp(&b.1));

    let mut columns: Vec<Vec<(f64, f64, f64)>> = Vec::new();
    for section in sections {
        let belongs = columns.last().is_some_and(|column| {
            let left = median(column.iter().map(|section| section.1));
            (section.1 - left).abs() <= stem_width * 0.5
        });
        match columns.last_mut() {
            Some(column) if belongs => column.push(section),
            _ => columns.push(vec![section]),
        }
    }

    let mut stems = Vec::new();
    for column in columns
        .into_iter()
        .filter(|column| column.len() >= SCAN_STEPS / 4)
    {
        let top = column
            .iter()
            .map(|section| section.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let bottom = column
            .iter()
            .map(|section| section.0)
            .fold(f64::INFINITY, f64::min);
        // The narrowest cross section is the stem at its plainest, away from
        // the flare of a bracket.
        let narrowest = column
            .iter()
            .map(|section| section.2 - section.1)
            .fold(f64::INFINITY, f64::min);
        let plain = column
            .iter()
            .filter(|section| section.2 - section.1 <= narrowest * 1.05)
            .collect::<Vec<_>>();
        let stem = Stem {
            left: median(plain.iter().map(|section| section.1)),
            right: median(plain.iter().map(|section| section.2)),
            top,
            bottom,
        };
        log::debug!(
            "stem {:.0}..{:.0} over {:.0}..{:.0}",
            stem.left,
            stem.right,
            stem.bottom,
            stem.top
        );
        stems.push(stem);
    }
    stems
}

/// The wall of a stem on one side, as an index into `runs`.
///
/// Several runs can sit near the same x. A serif end can be as vertical as the
/// wall it joins, and a bracket can have a nearly vertical piece of its own
/// above the stem, so the run is chosen by how much of the stem it covers: the
/// wall is the run that runs the height of the stem, not the one that happens
/// to reach furthest towards the edge of the glyph.
fn wall_run(
    samples: &[Sample],
    runs: &[Run],
    stem: &Stem,
    x: f64,
    top: bool,
    direction: i8,
    tolerance: f64,
) -> Option<usize> {
    let n = samples.len();
    let mut best: Option<((f64, f64), usize)> = None;
    for (index, run) in runs.iter().enumerate() {
        if run.direction != direction {
            continue;
        }
        let mut sum = 0.0;
        let mut count = 0;
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        let mut i = run.start;
        loop {
            sum += samples[i].point.x;
            low = low.min(samples[i].point.y);
            high = high.max(samples[i].point.y);
            count += 1;
            if i == run.end {
                break;
            }
            i = (i + 1) % n;
        }
        if (sum / count as f64 - x).abs() > tolerance {
            continue;
        }
        // A piece of a bracket that happens to be nearly vertical is not the
        // wall: of the runs at about the right x, the wall is the one that
        // ends where the stem does.
        let overlap = (high.min(stem.top) - low.max(stem.bottom)).max(0.0);
        if overlap <= 0.0 {
            continue;
        }
        let across = (sum / count as f64 - x).abs();
        if across > tolerance * 0.7 {
            return None;
        }
        let along = if top {
            (high - stem.top).abs()
        } else {
            (low - stem.bottom).abs()
        };
        let score = (along, across);
        if best.is_none_or(|(best_score, _)| score < best_score) {
            best = Some((score, index));
        }
    }
    best.map(|(_, index)| index)
}

/// The serif at one end of one stem, if the outline there has the shape of one.
#[allow(clippy::too_many_arguments)]
fn serif_at(
    glyph: char,
    contour: usize,
    segments: &[PathSeg],
    samples: &[Sample],
    runs: &[Run],
    stem: &Stem,
    top: bool,
    bbox: Rect,
    paths: &[&BezPath],
    slant: f64,
) -> Option<Serif> {
    let n = samples.len();
    let stem_width = (stem.right - stem.left).abs();
    // Going up, the left wall ends at the serif and the right wall starts
    // there; going down it is the other way round.
    let left = wall_run(samples, runs, stem, stem.left, top, 1, stem_width * 0.5)
        .inspect(|_| {})
        .or_else(|| {
            log::debug!("{glyph} {top}: no left wall");
            None
        })?;
    let right =
        wall_run(samples, runs, stem, stem.right, top, -1, stem_width * 0.5).or_else(|| {
            log::debug!("{glyph} {top}: no right wall");
            None
        })?;
    log::debug!(
        "{glyph} stem {:.0}..{:.0} y {:.0}..{:.0} {}: left wall run {:?} right wall run {:?}",
        stem.left,
        stem.right,
        stem.bottom,
        stem.top,
        if top { "top" } else { "bottom" },
        (
            samples[runs[left].start].point,
            samples[runs[left].end].point
        ),
        (
            samples[runs[right].start].point,
            samples[runs[right].end].point
        )
    );
    let (a, b) = if top {
        (runs[left].end, runs[right].start)
    } else {
        (runs[right].end, runs[left].start)
    };

    let mut indices = Vec::new();
    let mut i = (a + 1) % n;
    while i != b && indices.len() < n {
        indices.push(i);
        i = (i + 1) % n;
    }
    if indices.is_empty() || indices.len() >= n {
        return None;
    }

    let mut turn = 0.0;
    let mut previous = samples[a].tangent.atan2();
    for &i in indices.iter().chain(std::iter::once(&b)) {
        let current = samples[i].tangent.atan2();
        turn += cap_step(turn, previous, current);
        previous = current;
    }
    let turn = turn.to_degrees();
    if !(MIN_TURN_DEGREES..=MAX_TURN_DEGREES).contains(&turn.abs()) {
        log::debug!("{glyph} {top}: turn {turn:.0} out of range");
        return None;
    }

    let mut chain = slices_between(segments, samples, a, b);
    let (mut junction_a, mut junction_b) = (samples[a].point, samples[b].point);
    if junction_a.x > junction_b.x {
        std::mem::swap(&mut junction_a, &mut junction_b);
        chain = reverse_chain(&chain);
    }
    let points = resample(&flatten(&chain_path(&chain), stem_width * 0.004), 256);
    if points.len() < 4 {
        return None;
    }

    // A serif finishes the stem at the edge of the glyph. Anything that stops
    // short of that edge is a crossbar or the top of a counter.
    let (extremum, edge) = if top {
        (
            points
                .iter()
                .map(|point| point.y)
                .fold(f64::NEG_INFINITY, f64::max),
            bbox.y1,
        )
    } else {
        (
            points
                .iter()
                .map(|point| point.y)
                .fold(f64::INFINITY, f64::min),
            bbox.y0,
        )
    };
    if (edge - extremum).abs() > bbox.height() * EDGE_TOLERANCE {
        log::debug!("{glyph} {top}: reaches {extremum:.0}, not the edge at {edge:.0}");
        return None;
    }

    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let overhang = (junction_a.x - min_x).max(0.0) + (max_x - junction_b.x).max(0.0);
    if overhang < MIN_OVERHANG * stem_width {
        log::debug!("{glyph} {top}: overhang {overhang:.0} too small for stem {stem_width:.0}");
        return None;
    }

    // The serif's thickness is the depth of the part that sticks out past the
    // stem, so it is measured beyond the walls rather than between them.
    let buckets = 32;
    let span = (max_x - min_x).max(f64::MIN_POSITIVE);
    let mut low = vec![f64::INFINITY; buckets];
    let mut high = vec![f64::NEG_INFINITY; buckets];
    for point in &points {
        let bucket = (((point.x - min_x) / span) * (buckets - 1) as f64).round() as usize;
        low[bucket] = low[bucket].min(point.y);
        high[bucket] = high[bucket].max(point.y);
    }
    let mut height: f64 = 0.0;
    for bucket in 0..buckets {
        let x = min_x + span * bucket as f64 / (buckets - 1) as f64;
        if x > junction_a.x && x < junction_b.x {
            continue;
        }
        if low[bucket].is_finite() && high[bucket].is_finite() {
            height = height.max(high[bucket] - low[bucket]);
        }
    }
    if height <= 0.0 {
        log::debug!("{glyph} {top}: no protrusion");
        return None;
    }

    // Outside the serif there must be no ink; this rules out a chain that runs
    // along the inside of a stroke rather than around its end.
    // The serif's own thickness at its tips, which is not its overall height:
    // a bracket makes it much deeper where it meets the stem.
    let reach = span * 0.15;
    let mut tip_low = f64::INFINITY;
    let mut tip_high = f64::NEG_INFINITY;
    for point in &points {
        if point.x - min_x <= reach || max_x - point.x <= reach {
            tip_low = tip_low.min(point.y);
            tip_high = tip_high.max(point.y);
        }
    }
    let tip_thickness = (tip_high - tip_low).max(f64::MIN_POSITIVE);

    let away = if top { 1.0 } else { -1.0 };
    let outside = Point::new((min_x + max_x) / 2.0, extremum + away * height * 0.1);
    if is_ink(paths, unslant(outside, slant)) {
        log::debug!(
            "{glyph} {top}: ink outside the serif at ({:.0},{:.0})",
            outside.x,
            outside.y
        );
        return None;
    }

    // A bifurcated serif has its outer edge raised in the middle, so the
    // middle of that edge sits closer to the stem than both of its ends. A
    // serif whose outer edge merely slopes is not bifurcated, which is why the
    // middle is compared with the lower of the two ends.
    let outward = (0..buckets)
        .filter(|&bucket| low[bucket].is_finite() && high[bucket].is_finite())
        .map(|bucket| {
            let value = if top { high[bucket] } else { -low[bucket] };
            (bucket, value)
        })
        .collect::<Vec<_>>();
    let last = buckets - 1;
    let end = |from_left: bool| {
        let edge = outward
            .iter()
            .filter(|(bucket, _)| {
                if from_left {
                    *bucket * 5 < last
                } else {
                    *bucket * 5 > last * 4
                }
            })
            .map(|(_, value)| *value)
            .collect::<Vec<_>>();
        (!edge.is_empty()).then(|| median(edge))
    };
    let (Some(left), Some(right)) = (end(true), end(false)) else {
        return None;
    };
    let middle = outward
        .iter()
        .filter(|(bucket, _)| *bucket * 5 > last * 2 && *bucket * 5 < last * 3)
        .map(|(_, value)| *value)
        .fold(f64::INFINITY, f64::min);
    let dip = if middle.is_finite() {
        (left.min(right) - middle).max(0.0)
    } else {
        0.0
    };
    let bifurcation = dip / tip_thickness;

    Some(Serif {
        glyph,
        top,
        contour,
        stem_width,
        stem_left: stem.left,
        stem_right: stem.right,
        width: max_x - min_x,
        height,
        tip_thickness,
        bifurcation,
        junction_a,
        junction_b,
        chain,
        slant,
    })
}

/// Points at even distances along a polyline.
///
/// Flattening a chain leaves the corners and little else, so a straight edge
/// contributes two points however long it is. Spacing the points by distance
/// instead keeps every part of the serif represented when the chain is sampled
/// along an axis.
fn resample(points: &[Point], count: usize) -> Vec<Point> {
    if points.len() < 2 || count < 2 {
        return points.to_vec();
    }
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    let mut total = 0.0;
    for pair in points.windows(2) {
        total += pair[0].distance(pair[1]);
        lengths.push(total);
    }
    if total <= 0.0 {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(count);
    let mut index = 0;
    for step in 0..count {
        let target = total * step as f64 / (count - 1) as f64;
        while index + 2 < lengths.len() && lengths[index + 1] < target {
            index += 1;
        }
        let length = lengths[index + 1] - lengths[index];
        let t = if length > 0.0 {
            ((target - lengths[index]) / length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(points[index].lerp(points[index + 1], t));
    }
    out
}

fn reverse_chain(chain: &[SegmentSlice]) -> Vec<SegmentSlice> {
    chain
        .iter()
        .rev()
        .map(|slice| SegmentSlice {
            index: slice.index,
            segment: slice.segment.reverse(),
            t0: 1.0 - slice.t1,
            t1: 1.0 - slice.t0,
        })
        .collect()
}

/// Measurements of one side of a serif: the shoulder where it leaves the stem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shoulder {
    /// How far the serif sticks out past the stem, in font units.
    pub overhang: f64,
    /// How far the outline drops over the whole overhang, as a fraction of the
    /// serif's height. A serif that sits flat stays near zero.
    pub drop: f64,
    /// Angle of the shoulder away from horizontal, in degrees. A slab's
    /// shoulder is horizontal; a wedge's is not.
    pub slope: f64,
    /// Total turning over the shoulder, in degrees. A straight shoulder does
    /// not turn at all.
    pub turn: f64,
    /// How far the shoulder bows away from the straight line between its ends,
    /// relative to the length of that line. A straight shoulder stays at zero.
    pub deviation: f64,
    /// Thickness of the serif at its tip, relative to the serif's height.
    pub thickness: f64,
}

/// Measurements of a serif, used to categorise it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SerifFeatures {
    /// Thickness of the serif, relative to the stem width.
    pub height_ratio: f64,
    /// Thickness of the serif at its tips, relative to the stem width. A
    /// bracket makes the serif deeper where it meets the stem, so this is the
    /// measurement that says whether a serif is drawn with a hairline.
    pub tip_ratio: f64,
    /// Width of the serif, relative to the stem width.
    pub width_ratio: f64,
    pub a: Shoulder,
    pub b: Shoulder,
    /// Difference between the profiles of the two sides; zero when they match.
    pub asymmetry: f64,
    /// Width of the gap in the middle of the outer edge, relative to the serif
    /// width.
    pub bifurcation: f64,
}

impl SerifFeatures {
    /// The shoulder on the side of the serif that sticks out furthest: the one
    /// that says most about the design.
    fn dominant(&self) -> Shoulder {
        if self.a.overhang > self.b.overhang {
            self.a
        } else {
            self.b
        }
    }
}

pub fn serif_features(serif: &Serif) -> Option<SerifFeatures> {
    let points = resample(
        &flatten(&chain_path(&serif.chain), serif.stem_width * 0.004),
        256,
    );
    if points.len() < 4 || serif.height <= 0.0 {
        return None;
    }
    let a = measure_shoulder(&points, true, serif.stem_left, serif.height)?;
    let b = measure_shoulder(&points, false, serif.stem_right, serif.height)?;
    let asymmetry = (a.deviation - b.deviation).abs();
    Some(SerifFeatures {
        height_ratio: serif.height / serif.stem_width,
        tip_ratio: serif.tip_thickness / serif.stem_width,
        width_ratio: serif.width / serif.stem_width,
        a,
        b,
        asymmetry,
        bifurcation: serif.bifurcation,
    })
}

/// Measure one side of a serif, walking from its shoulder towards its tip.
fn measure_shoulder(
    points: &[Point],
    from_start: bool,
    stem_x: f64,
    height: f64,
) -> Option<Shoulder> {
    let side = if from_start {
        points.to_vec()
    } else {
        points.iter().rev().copied().collect()
    };
    let tip_x = if from_start {
        side.iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min)
    } else {
        side.iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let overhang = (stem_x - tip_x).abs();
    if overhang <= 0.0 {
        return None;
    }
    let direction = if tip_x < stem_x { -1.0 } else { 1.0 };
    let reached =
        |point: &Point, fraction: f64| (point.x - stem_x) * direction >= fraction * overhang;

    let transition = side
        .iter()
        .position(|point| reached(point, TRANSITION_SPAN))
        .unwrap_or(side.len() - 1)
        .max(1);
    let tip = side
        .iter()
        .position(|point| reached(point, 0.95))
        .unwrap_or(side.len() - 1)
        .max(1);

    let mut turn = 0.0;
    for window in side[..=transition].windows(3) {
        let incoming = window[1] - window[0];
        let outgoing = window[2] - window[1];
        if incoming.hypot() > 1e-9 && outgoing.hypot() > 1e-9 {
            turn += signed_angle(incoming, outgoing).abs();
        }
    }

    let mut low: f64 = f64::INFINITY;
    let mut high: f64 = f64::NEG_INFINITY;
    for point in &side {
        if (point.x - stem_x).abs() >= 0.85 * overhang {
            low = low.min(point.y);
            high = high.max(point.y);
        }
    }
    let thickness = if low.is_finite() && high.is_finite() {
        (high - low) / height
    } else {
        0.0
    };

    let chord = side[transition] - side[0];
    let length = chord.hypot();
    let normal = if length > 0.0 {
        Vec2::new(-chord.y, chord.x) / length
    } else {
        Vec2::ZERO
    };
    let deviation = if length > 0.0 {
        side[..=transition]
            .iter()
            .map(|point| (*point - side[0]).dot(normal).abs())
            .fold(0.0, f64::max)
            / length
    } else {
        0.0
    };
    Some(Shoulder {
        overhang,
        drop: (side[tip].y - side[0].y).abs() / height,
        slope: chord.y.abs().atan2(chord.x.abs()).to_degrees(),
        deviation,
        turn: turn.to_degrees(),
        thickness,
    })
}

fn signed_angle(from: Vec2, to: Vec2) -> f64 {
    let mut difference = to.atan2() - from.atan2();
    while difference > std::f64::consts::PI {
        difference -= std::f64::consts::TAU;
    }
    while difference <= -std::f64::consts::PI {
        difference += std::f64::consts::TAU;
    }
    difference
}

/// The shape of a serif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerifShape {
    /// The outline curves away from the stem: a fillet.
    Bracketed,
    /// The outline leaves the stem at an angle, in a straight line.
    Wedge,
    /// The serif sits flat against the stem, with a corner at the shoulder.
    Slab,
    /// A thin serif, whatever the shape of its shoulder.
    Hairline,
}

impl std::fmt::Display for SerifShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SerifShape::Bracketed => "bracketed",
            SerifShape::Wedge => "wedge",
            SerifShape::Slab => "slab",
            SerifShape::Hairline => "hairline",
        })
    }
}

/// The shape of a serif, or `None` if there is nothing to measure.
pub fn classify_serif(serif: &Serif) -> Option<SerifShape> {
    serif_features(serif).map(|features| classify_features(&features))
}

fn classify_features(features: &SerifFeatures) -> SerifShape {
    if features.height_ratio < HAIRLINE_RATIO {
        return SerifShape::Hairline;
    }
    let shoulder = features.dominant();
    if shoulder.deviation > CURVED_SHOULDER {
        SerifShape::Bracketed
    } else if shoulder.drop < FLAT_SHOULDER_DROP || shoulder.slope < FLAT_SHOULDER_DEGREES {
        SerifShape::Slab
    } else {
        SerifShape::Wedge
    }
}

/// Whether the two sides of the serif differ.
pub fn is_asymmetric(serif: &Serif) -> Option<bool> {
    serif_features(serif).map(|features| features.asymmetry > ASYMMETRY_DEVIATION)
}

/// Whether the middle of the serif's outer edge is raised.
pub fn is_bifurcated(serif: &Serif) -> Option<bool> {
    Some(serif.bifurcation > BIFURCATION_RATIO)
}

fn most_common_of<T: Eq + std::hash::Hash + Copy>(items: impl IntoIterator<Item = T>) -> Option<T> {
    let mut counts: Vec<(T, usize)> = Vec::new();
    for item in items {
        match counts.iter_mut().find(|(existing, _)| *existing == item) {
            Some((_, count)) => *count += 1,
            None => counts.push((item, 1)),
        }
    }
    let highest = counts.iter().map(|(_, count)| *count).max()?;
    let winners = counts
        .iter()
        .filter(|(_, count)| *count == highest)
        .collect::<Vec<_>>();
    (winners.len() == 1).then(|| winners[0].0)
}

/// The shape found on most of a font's serifs, or `None` on a tie.
pub fn most_common_shape(shapes: impl IntoIterator<Item = SerifShape>) -> Option<SerifShape> {
    most_common_of(shapes)
}

quantifier!(
    SERIF_SHAPE,
    "structure/serif_shape",
    r#"The shape of the serifs at the ends of the stems of the letters 'I' and 'H',
    one of `bracketed`, `wedge`, `slab` or `hairline`.

    * **bracketed**: the outline curves away from the stem, joining the serif to it with a fillet;
    * **wedge**: the outline leaves the stem at an angle, in a straight line;
    * **slab**: the serif sits flat against the stem, with a (possibly rounded) corner at the shoulder;
    * **hairline**: a thin serif, whatever the shape of its shoulder. Whether a serif is a hairline
      is a matter of height rather than of shape, so a wedge serif that is only a few units high
      is still a hairline.

    Reports `none` when no serifs can be found, which is the case for sans serif fonts."#,
    MetricValue::String("none".to_string())
);

quantifier!(
    SERIF_ASYMMETRIC,
    "structure/serif_asymmetric",
    "Reports whether the serifs of a font have a different profile on their left and right sides.",
    MetricValue::Boolean(false)
);

quantifier!(
    SERIF_BIFURCATED,
    "structure/serif_bifurcated",
    "Reports whether the serifs of a font are bifurcated, that is, whether the middle of the serif's outer edge is raised so that the serif has two lobes.",
    MetricValue::Boolean(false)
);

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // it's a test suite
mod tests {
    use super::*;

    /// An `I` with serifs of a simple shape: `bracket` is the height over which
    /// the serif curves into the stem, and `notch` lifts the middle of the
    /// bottom edge.
    fn probe(width: f64, stem: f64, height: f64, serif: f64, bracket: f64, notch: f64) -> BezPath {
        let (outer, inner) = (-width / 2.0, width / 2.0);
        let (left, right) = (-stem / 2.0, stem / 2.0);
        let (shoulder, top) = (serif + bracket, height - serif - bracket);
        let mut path = BezPath::new();
        path.move_to((outer, 0.0));
        path.line_to((outer, serif));
        if bracket > 0.0 {
            path.quad_to((left, serif), (left, shoulder));
        } else {
            path.line_to((left, serif));
        }
        path.line_to((left, top));
        if bracket > 0.0 {
            path.quad_to((left, height - serif), (outer, height - serif));
        } else {
            path.line_to((left, height - serif));
        }
        path.line_to((outer, height));
        path.line_to((inner, height));
        path.line_to((inner, height - serif));
        if bracket > 0.0 {
            path.quad_to((right, height - serif), (right, top));
        } else {
            path.line_to((right, height - serif));
        }
        path.line_to((right, shoulder));
        if bracket > 0.0 {
            path.quad_to((right, serif), (inner, serif));
        } else {
            path.line_to((inner, serif));
        }
        path.line_to((inner, 0.0));
        if notch > 0.0 {
            path.line_to((0.0, notch));
        }
        path.line_to((outer, 0.0));
        path.close_path();
        path
    }

    fn serifs_of(path: &BezPath) -> Vec<Serif> {
        find_serifs('I', &[path], 0.0, 1000.0)
    }

    #[test]
    fn square_serifs_are_slabs() {
        let path = probe(600.0, 200.0, 700.0, 120.0, 0.0, 0.0);
        let serifs = serifs_of(&path);
        assert_eq!(serifs.len(), 2, "{serifs:#?}");
        for serif in &serifs {
            assert_eq!(classify_serif(serif), Some(SerifShape::Slab));
            assert_eq!(is_asymmetric(serif), Some(false));
            assert_eq!(is_bifurcated(serif), Some(false));
        }
    }

    #[test]
    fn curved_serifs_are_bracketed() {
        let path = probe(600.0, 200.0, 700.0, 120.0, 100.0, 0.0);
        let serifs = serifs_of(&path);
        assert_eq!(serifs.len(), 2, "{serifs:#?}");
        for serif in &serifs {
            assert_eq!(classify_serif(serif), Some(SerifShape::Bracketed));
        }
    }

    #[test]
    fn thin_serifs_are_hairlines() {
        let path = probe(600.0, 200.0, 700.0, 30.0, 0.0, 0.0);
        let serifs = serifs_of(&path);
        assert_eq!(serifs.len(), 2, "{serifs:#?}");
        for serif in &serifs {
            assert_eq!(classify_serif(serif), Some(SerifShape::Hairline));
        }
    }

    #[test]
    fn notched_serif_is_bifurcated() {
        let path = probe(600.0, 200.0, 700.0, 120.0, 0.0, 60.0);
        let serifs = serifs_of(&path);
        assert_eq!(serifs.len(), 2, "{serifs:#?}");
        let bottom = serifs.iter().find(|serif| !serif.top).unwrap();
        assert_eq!(is_bifurcated(bottom), Some(true));
        let top = serifs.iter().find(|serif| serif.top).unwrap();
        assert_eq!(is_bifurcated(top), Some(false));
    }

    #[test]
    fn sans_font_has_no_serifs() {
        let path = probe(600.0, 200.0, 700.0, 0.0, 0.0, 0.0);
        assert!(serifs_of(&path).is_empty());
    }

    fn font(name: &str) -> Vec<u8> {
        let path = format!("{}/../tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).unwrap()
    }

    fn metric(font_data: &[u8], name: &str) -> Option<String> {
        let font = skrifa::FontRef::new(font_data).unwrap();
        let mut results = crate::Results::new();
        detect_serifs(&font, &[], &mut results).unwrap();
        results.get(name).map(|(_, value)| value.to_string())
    }

    #[test]
    fn serif_shapes_for_test_fonts() {
        for (name, shape) in [
            ("YoungSerif-Regular.ttf", "bracketed"),
            ("AlfaSlabOne-Regular.ttf", "slab"),
            ("Alike-Regular.ttf", "wedge"),
            ("BodoniModa_18pt-Italic.ttf", "hairline"),
        ] {
            assert_eq!(
                metric(&font(name), &SERIF_SHAPE.name).as_deref(),
                Some(shape),
                "{name}"
            );
        }
        assert_eq!(
            metric(
                &font("OpenSans-VariableFont_wdth,wght.ttf"),
                &SERIF_SHAPE.name
            )
            .as_deref(),
            Some("none")
        );
    }
}
