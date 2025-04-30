use kurbo::BezPath;

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
