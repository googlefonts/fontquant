use kurbo::{BezPath, Insets, Line, ParamCurve, Point, Rect, Shape, Vec2};
use skia_safe::{EncodedImageFormat, PaintStyle};

use crate::bezglyph::{BezGlyph, bezpaths_to_skpath};

pub const EAST: Direction = Direction::Angle(0.0);
// pub const NORTHEAST: Direction = Direction::Angle(45.0);
pub const NORTH: Direction = Direction::Angle(90.0);
// pub const NORTHWEST: Direction = Direction::Angle(135.0);
// pub const WEST: Direction = Direction::Angle(180.0);
// pub const SOUTHWEST: Direction = Direction::Angle(225.0);
// pub const SOUTH: Direction = Direction::Angle(270.0);
// pub const SOUTHEAST: Direction = Direction::Angle(315.0);

const EPSILON: f64 = 5.0;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Angle(f64),
    EndPoint(ProportionalPoint),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Winding {
    Ink,
    Transparent,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProportionalPoint {
    x: f64,
    y: f64,
}
impl ProportionalPoint {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn to_point(self, bbox: &Rect) -> Point {
        Point::new(
            self.x * bbox.width() + bbox.min_x(),
            self.y * bbox.height() + bbox.min_y(),
        )
    }
}

fn drop_outliers(
    references: Vec<(Point, Point, f64)>,
    deviations: Option<f64>,
) -> Vec<(Point, Point, f64)> {
    let mut deviations = deviations.unwrap_or(0.1);
    if references.len() < 3 {
        return references;
    }
    // Otherwise, drop anything n standard deviations away from the median
    let just_distances: Vec<f64> = references.iter().map(|(_, _, d)| *d).collect();
    if just_distances.is_empty() {
        return vec![];
    }
    let median_dist = statistical::median(&just_distances);
    let stdev = statistical::standard_deviation(&just_distances, Some(median_dist));
    if stdev == 0.0 {
        return references;
    }

    while deviations < 2.0 {
        let mut value = references.clone();
        value.retain(|&d| (d.2 - median_dist).abs() < stdev * deviations);
        if value.is_empty() {
            deviations += 0.1;
            continue;
        }
        return value;
    }
    references
}

fn t_of_point(line: Line, point: &Point) -> f64 {
    if !isclose(line.end().x, line.start().x) {
        (point.x - line.start().x) / (line.end().x - line.start().x)
    } else if !isclose(line.end().y, line.start().y) {
        (point.y - line.start().y) / (line.end().y - line.start().y)
    } else {
        0.0
    }
}

fn isclose(x_1: f64, x_2: f64) -> bool {
    (x_1 - x_2).abs() < f64::EPSILON
}

pub struct Raycaster<'a> {
    paths: &'a Vec<BezPath>,
    start: ProportionalPoint,
    end: Option<ProportionalPoint>,
    direction: Vec2,
    bbox: Rect,
    winding: Option<Winding>,
    jittering: Option<f32>,
    samples: usize,
    _intersections: Vec<Vec<Point>>,
    _pairs: Vec<Vec<(Point, Point)>>,
}

impl<'a> Raycaster<'a> {
    pub fn new(glyph: &'a BezGlyph, start: ProportionalPoint, direction: Direction) -> Self {
        let paths = &glyph.0;
        let winding = None;
        // Sensible defaults
        let jittering = Some(0.1);
        let samples = 12;
        let _intersections = vec![];
        let _pairs = vec![];
        let bbox = glyph
            .bbox()
            .unwrap_or_default()
            .inset(Insets::uniform(10.0));
        let (direction, endpoint) = match direction {
            Direction::Angle(angle) => {
                let angle = angle.to_radians();
                (Vec2::from_angle(angle), None)
            }
            Direction::EndPoint(end) => {
                let direction = Vec2::new(end.x - start.x, end.y - start.y);
                (direction, Some(end))
            }
        };

        Self {
            paths,
            start,
            end: endpoint,
            direction,
            bbox,
            winding,
            jittering,
            samples,
            _intersections,
            _pairs,
        }
    }

    pub fn winding(&mut self, winding: Winding) -> &mut Self {
        self.winding = Some(winding);
        self
    }

    pub fn jitter(&mut self, jittering: f32, samples: usize) -> &mut Self {
        self.jittering = Some(jittering);
        self.samples = samples;
        self
    }

    fn _create_ray(
        &self,
        start_pt: ProportionalPoint,
        end_pt: Option<ProportionalPoint>,
    ) -> (Line, Line) {
        let mut s = start_pt.to_point(&self.bbox);
        let mut e = match end_pt {
            Some(end) => end.to_point(&self.bbox),
            None => {
                let mut tmp_e = s + self.direction;
                while self.bbox.contains(tmp_e) {
                    tmp_e += self.direction;
                }
                tmp_e
            }
        };
        s -= self.direction * EPSILON;
        e += self.direction * EPSILON;
        let original_line = Line::new(s, e);
        s -= self.direction * 300.0;
        while self.bbox.contains(s) {
            s -= self.direction * 300.0;
        }
        e += self.direction * 300.0;
        while self.bbox.contains(e) {
            e += self.direction * 300.0;
        }
        (Line::new(s, e), original_line)
    }

    fn _create_starts_ends(&self) -> Vec<(ProportionalPoint, Option<ProportionalPoint>)> {
        let mut starts_ends = vec![];
        if let Some(jittering) = self.jittering {
            let perp = Vec2::new(-self.direction.y, self.direction.x);
            for i in 0..self.samples {
                let jitter_ix = 2.0 * (i as f32 - self.samples as f32 / 2.0) * jittering
                    / (self.samples as f32);
                let jitter = perp * jitter_ix.into();
                let start =
                    ProportionalPoint::new(self.start.x + jitter.x, self.start.y + jitter.y);
                let end = self
                    .end
                    .map(|end| ProportionalPoint::new(end.x + jitter.x, end.y + jitter.y));
                starts_ends.push((start, end));
            }
        } else {
            starts_ends.push((self.start, self.end));
        }
        starts_ends
    }

    fn _cast_ray(&mut self) {
        self._intersections.clear();
        let rays = self
            ._create_starts_ends()
            .into_iter()
            .map(|(start, end)| self._create_ray(start, end))
            .collect::<Vec<_>>();
        for (ray, short_ray) in rays.into_iter() {
            let mut intersections = self
                .paths
                .iter()
                .flat_map(|p| p.segments().map(|s| s.intersect_line(ray).into_iter()))
                .flatten()
                .collect::<Vec<_>>();
            intersections.sort_by(|a, b| a.line_t.total_cmp(&b.line_t));
            // Uniquify intersection to within distance EPSILON
            let mut intersection_points = intersections
                .iter()
                .map(|i| ray.eval(i.line_t))
                .collect::<Vec<_>>();
            intersection_points.dedup_by(|a, b| a.distance(*b) < EPSILON);
            // Bound it to short ray
            intersection_points.retain(|pt| {
                let short_t = t_of_point(short_ray, pt);
                0.0 < short_t && short_t < 1.0 && pt.distance(short_ray.start()) > EPSILON
            });
            self._intersections.push(intersection_points);
        }
    }

    pub fn pairs(&mut self) -> &Vec<Vec<(Point, Point)>> {
        self._pairs.clear();
        if self._intersections.is_empty() {
            self._cast_ray();
        }
        for intersections in self._intersections.iter() {
            if intersections.len() < 2 {
                self._pairs.push(vec![]);
                continue;
            }
            if intersections.len() == 2
                && (self.winding.is_none() || self.winding == Some(Winding::Ink))
            {
                let pair = vec![(intersections[0], intersections[1])];
                self._pairs.push(pair);
                continue;
            }
            // Find point-to-point distances
            let mut distances = vec![];
            for i in 0..intersections.len() - 1 {
                if self.winding == Some(Winding::Ink) && i % 2 == 1 {
                    continue;
                }
                if self.winding == Some(Winding::Transparent) && i % 2 == 0 {
                    continue;
                }
                let distance = intersections[i].distance(intersections[i + 1]);
                if distance > 0.0 {
                    distances.push((intersections[i], intersections[i + 1], distance));
                }
            }
            self._pairs.push(
                distances
                    .iter()
                    .map(|(start, end, _)| (*start, *end))
                    .collect(),
            );
        }
        &self._pairs
    }

    pub(crate) fn distances(&mut self) -> Vec<(Point, Point, f64)> {
        if self._pairs.is_empty() {
            self.pairs();
        }
        self._pairs
            .iter()
            .filter(|ray_pairs| !ray_pairs.is_empty())
            .flatten()
            .map(|(start, end)| (*start, *end, start.distance(*end)))
            .collect::<Vec<_>>()
    }

    pub fn median_pair_distance(&mut self, remove_outliers: bool) -> f64 {
        let mut distances = self.distances();
        if remove_outliers {
            let new_distances = drop_outliers(distances.clone(), None);
            // Log outliers here
            distances = new_distances;
        }
        let distances = distances.iter().map(|(_, _, d)| *d).collect::<Vec<_>>();
        if distances.is_empty() {
            0.0
        } else {
            // Return the median distance
            statistical::median(&distances)
        }
    }

    // Debugging draw tool. If it breaks, you get to keep the pieces.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) fn draw(&mut self) -> skia_safe::Data {
        #[allow(clippy::expect_used)]
        let mut surface = skia_safe::surfaces::raster_n32_premul((
            self.bbox.width() as i32,
            self.bbox.height() as i32,
        ))
        .expect("surface");
        #[allow(clippy::unwrap_used)]
        let tf = skia_safe::FontMgr::new()
            .legacy_make_typeface(None, skia_safe::FontStyle::normal())
            .unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::Color::WHITE);
        // Draw the glyph. Upside-down, because Y=0 is top, but this is just a debugging tool.
        let skia_path = bezpaths_to_skpath(self.paths);
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.3, 0.3, 0.3, 1.0), None);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke(true);
        paint.set_stroke_width(2.0);

        canvas.draw_path(&skia_path, &paint);
        // Draw the rays
        let rays = self
            ._create_starts_ends()
            .into_iter()
            .map(|(start, end)| self._create_ray(start, end))
            .collect::<Vec<_>>();
        let short_ray_bbox = rays
            .iter()
            .map(|(_, short_ray)| short_ray.bounding_box())
            .reduce(|acc, bbox| acc.union(bbox))
            .unwrap_or_default();
        let fill_color = skia_safe::Color4f::new(1.0, 0.66, 0.66, 0.27); // "#ffaaaa44"
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
        paint.set_style(PaintStyle::StrokeAndFill);
        paint.set_color4f(fill_color, None);
        canvas.draw_rect(
            skia_safe::Rect {
                left: short_ray_bbox.min_x() as f32,
                top: short_ray_bbox.min_y() as f32,
                right: short_ray_bbox.max_x() as f32,
                bottom: short_ray_bbox.max_y() as f32,
            },
            &paint,
        );
        for (long_ray, _short_ray) in rays.iter() {
            let mut paint =
                skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 0.13, 0.13, 0.26), None);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(1.0);
            let start = skia_safe::Point::new(long_ray.start().x as f32, long_ray.start().y as f32);
            let end = skia_safe::Point::new(long_ray.end().x as f32, long_ray.end().y as f32);
            canvas.draw_line(start, end, &paint);
        }

        for intersections in self._intersections.iter() {
            for pt in intersections.iter() {
                let mut paint =
                    skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 0.0, 1.0, 1.0), None);
                paint.set_style(PaintStyle::Fill);
                let point = skia_safe::Point::new(pt.x as f32, pt.y as f32);
                canvas.draw_circle(point, 5.0, &paint);
            }
        }

        let distances = self.distances();
        for (start, end, distance) in distances.iter() {
            let mut paint =
                skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
            paint.set_style(PaintStyle::Fill);
            let normal = Vec2::new(end.y - start.y, start.x - end.x).normalize();
            let start = skia_safe::Point::new(start.x as f32, start.y as f32);
            let end = skia_safe::Point::new(end.x as f32, end.y as f32);
            let mut midpoint =
                skia_safe::Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
            // Add 10 units in direction of normal
            midpoint += skia_safe::Point::new(normal.x as f32, normal.y as f32) * 10.0;
            canvas.draw_line(start, end, &paint);
            let text = format!("{:.0}", distance);
            paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
            canvas.draw_str_align(
                &text,
                midpoint,
                &skia_safe::Font::from_typeface_with_params(&tf, 20.0, 1.0, 0.0),
                &paint,
                skia_safe::utils::text_utils::Align::Center,
            );
        }
        // Drop outliers and do it again
        let non_outliers = drop_outliers(distances.clone(), None);
        for (start, end, distance) in non_outliers.iter() {
            let mut paint =
                skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
            paint.set_style(PaintStyle::Fill);
            let normal = Vec2::new(end.y - start.y, start.x - end.x).normalize();
            let start = skia_safe::Point::new(start.x as f32, start.y as f32);
            let end = skia_safe::Point::new(end.x as f32, end.y as f32);
            let mut midpoint =
                skia_safe::Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
            // Add 10 units in direction of normal
            midpoint += skia_safe::Point::new(normal.x as f32, normal.y as f32) * 10.0;
            canvas.draw_line(start, end, &paint);
            let text = format!("{:.0}", distance);
            paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 1.0, 1.0), None);
            canvas.draw_str_align(
                &text,
                midpoint,
                &skia_safe::Font::from_typeface_with_params(&tf, 20.0, 1.0, 0.0),
                &paint,
                skia_safe::utils::text_utils::Align::Center,
            );
        }

        let image = surface.image_snapshot();
        let mut context = surface.direct_context();
        #[allow(clippy::unwrap_used)]
        image
            .encode(context.as_mut(), EncodedImageFormat::PNG, None)
            .unwrap()
    }
}
