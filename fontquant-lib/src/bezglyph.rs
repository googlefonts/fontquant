use fontations::skrifa;
use kurbo::BezPath;

use crate::error::FontquantError;

#[derive(Default, Debug)]
pub struct BezGlyph(pub(crate) Vec<BezPath>);

impl BezGlyph {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> &mut BezPath {
        self.0.push(BezPath::new());
        #[allow(clippy::unwrap_used)] // We just added it
        self.0.last_mut().unwrap()
    }
    pub fn current(&mut self) -> &mut BezPath {
        if self.0.is_empty() {
            self.0.push(BezPath::new());
        }
        #[allow(clippy::unwrap_used)] // We know it's not empty
        self.0.last_mut().unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &BezPath> {
        self.0.iter()
    }

    pub fn remove_overlaps(&self) -> Result<BezGlyph, FontquantError> {
        use skia_safe::simplify;
        let path = bezpaths_to_skpath(self.iter());
        let newpath = simplify(&path).ok_or(FontquantError::SkiaError)?;
        Ok(skpath_to_bezglyph(newpath))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl skrifa::outline::OutlinePen for BezGlyph {
    fn move_to(&mut self, x: f32, y: f32) {
        self.next().move_to((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current().line_to((x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.current().quad_to((cx0, cy0), (x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.current().curve_to((cx0, cy0), (cx1, cy1), (x, y));
    }

    fn close(&mut self) {
        self.current().close_path();
    }
}

pub struct ScalerPen<'a> {
    pub pen: &'a mut dyn skrifa::outline::OutlinePen,
    scale: f32,
}

impl ScalerPen<'_> {
    pub fn new(pen: &mut dyn skrifa::outline::OutlinePen, scale: f32) -> ScalerPen {
        ScalerPen { pen, scale }
    }
}

impl skrifa::outline::OutlinePen for ScalerPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pen.move_to(x * self.scale, y * self.scale);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.pen.line_to(x * self.scale, y * self.scale);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.pen.quad_to(
            cx0 * self.scale,
            cy0 * self.scale,
            x * self.scale,
            y * self.scale,
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.pen.curve_to(
            cx0 * self.scale,
            cy0 * self.scale,
            cx1 * self.scale,
            cy1 * self.scale,
            x * self.scale,
            y * self.scale,
        );
    }

    fn close(&mut self) {
        self.pen.close();
    }
}

// Yes, this is a horrible piece of code. But it should hopefully go away soon
// when kurbo gets native path operations support.

pub(crate) fn bezpaths_to_skpath<'a, I: IntoIterator<Item = &'a BezPath>>(
    bezpaths: I,
) -> skia_safe::Path {
    let mut path = skia_safe::Path::default();
    for pathel in bezpaths.into_iter().flatten() {
        match pathel {
            kurbo::PathEl::MoveTo(point) => path.move_to((point.x as i32, point.y as i32)),
            kurbo::PathEl::LineTo(point) => path.line_to((point.x as i32, point.y as i32)),
            kurbo::PathEl::QuadTo(point, point1) => path.quad_to(
                (point.x as i32, point.y as i32),
                (point1.x as i32, point1.y as i32),
            ),
            kurbo::PathEl::CurveTo(point, point1, point2) => path.cubic_to(
                (point.x as i32, point.y as i32),
                (point1.x as i32, point1.y as i32),
                (point2.x as i32, point2.y as i32),
            ),
            kurbo::PathEl::ClosePath => path.close(),
        };
    }
    path
}

pub(crate) fn skpath_to_bezglyph(path: skia_safe::Path) -> BezGlyph {
    let points_count = path.count_points();
    let mut points = vec![skia_safe::Point::default(); points_count];
    let _count_returned = path.get_points(&mut points);

    let verb_count = path.count_verbs();
    let mut verbs = vec![0_u8; verb_count];
    let _count_returned_verbs = path.get_verbs(&mut verbs);
    let mut newglyph = BezGlyph::default();
    let mut cur_pt = 0;
    for verb in verbs {
        let verb: skia_safe::PathVerb = unsafe { std::mem::transmute(verb as u32) };
        match verb {
            skia_safe::PathVerb::Move => {
                <BezGlyph as skrifa::outline::OutlinePen>::move_to(
                    &mut newglyph,
                    points[cur_pt].x,
                    points[cur_pt].y,
                );
                cur_pt += 1;
            }
            skia_safe::PathVerb::Line => {
                <BezGlyph as skrifa::outline::OutlinePen>::line_to(
                    &mut newglyph,
                    points[cur_pt].x,
                    points[cur_pt].y,
                );
                cur_pt += 1;
            }
            skia_safe::PathVerb::Quad => {
                <BezGlyph as skrifa::outline::OutlinePen>::quad_to(
                    &mut newglyph,
                    points[cur_pt].x,
                    points[cur_pt].y,
                    points[cur_pt + 1].x,
                    points[cur_pt + 1].y,
                );
                cur_pt += 2;
            }
            skia_safe::PathVerb::Conic => {
                panic!("You got a conic curve into a TrueType font, clever you.")
            }
            skia_safe::PathVerb::Cubic => {
                <BezGlyph as skrifa::outline::OutlinePen>::curve_to(
                    &mut newglyph,
                    points[cur_pt].x,
                    points[cur_pt].y,
                    points[cur_pt + 1].x,
                    points[cur_pt + 1].y,
                    points[cur_pt + 2].x,
                    points[cur_pt + 2].y,
                );
                cur_pt += 3;
            }
            skia_safe::PathVerb::Close => {
                <BezGlyph as skrifa::outline::OutlinePen>::close(&mut newglyph);
            }
        }
    }
    newglyph
}
