use crate::bezglyph::BezGlyph;
use kurbo::{PathEl, Point, Shape};

const TARGET_RES: usize = 1024;
const SS: usize = 2;
const MARGIN: usize = 16;
const INF: f32 = 1e30;

/// Estimate the stroke contrast of a glyph by rasterizing it, computing a
/// distance transform, extracting the medial axis (skeleton), pruning the
/// skeleton's dead-end branches, and taking the ratio of the 90th to the 10th
/// percentile of the stroke half-widths along the remaining skeleton.
///
/// Returns `(contrast, Some(stress_angle_in_degrees))`, or `None` if the glyph
/// is too far from a simple ring (outer contour plus a counter) to be measured.
pub fn stroke_contrast_medial(glyph: &BezGlyph) -> Option<(f32, Option<f32>)> {
    let cleaned = glyph.remove_overlaps().ok().unwrap_or_else(|| glyph.clone());
    let contours = flatten_glyph(&cleaned);
    if contours.len() < 2 || contours.len() > 8 {
        // A clean 'o' is a single outer contour with a counter; anything with
        // many contours (e.g. pixel fonts built from dozens of rectangles) is
        // too unusual to measure.
        return None;
    }

    let bbox = contours
        .iter()
        .map(|c| poly_bbox(c))
        .reduce(|a, b| a.union(b))?;
    let gsize = bbox.width().max(bbox.height());
    if gsize <= 1.0 {
        return None;
    }

    // Outer contour has the largest area; the counter is the next largest and
    // must lie inside it.
    let mut order = (0..contours.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| {
        polygon_area(&contours[b])
            .abs()
            .total_cmp(&polygon_area(&contours[a]).abs())
    });
    let outer = &contours[order[0]];
    let inner = &contours[order[1]];
    let inner_centroid = poly_centroid(inner);
    if !point_in_polygon(inner_centroid, outer) {
        return None;
    }

    let spacing = (gsize / 1000.0).clamp(0.5, 2.0);
    let outer_raw = resample_poly(outer, spacing);
    let inner_raw = resample_poly(inner, spacing);

    // The smoothing window is a small fraction of the glyph size, but never
    // more than half the minimum counter clearance, so we don't close the
    // counter of high-contrast fonts whose hairlines are a fraction of a unit.
    let clearance = min_distance_to_poly(&inner_raw, &outer_raw)?;
    if clearance <= 0.0 {
        return None;
    }
    let window = (0.02 * gsize).min(0.5 * clearance);
    let outer_smooth = smooth_poly(&outer_raw, window);
    let inner_smooth = smooth_poly(&inner_raw, window);

    let (mask, w, h, scale) = rasterize(&[outer_smooth, inner_smooth], gsize, &bbox);
    let mask = binary_closing(&mask, w, h);
    if count_holes(&mask, w, h) != 1 {
        return None;
    }

    let dt = chamfer_distance(&mask, w, h);
    let mut skeleton = zhang_suen(&mask, w, h);
    prune_endpoints(&mut skeleton, w, h);

    let mut positions = Vec::new();
    let mut widths = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if skeleton[idx] != 0 {
                let dist = dt[idx] / 3.0 / scale as f32;
                if dist > 0.0 {
                    positions.push((x, y, dist));
                    widths.push(dist);
                }
            }
        }
    }
    if widths.len() < 30 {
        return None;
    }

    widths.sort_by(|a, b| a.total_cmp(b));
    let p10 = percentile(&widths, 0.10);
    let p90 = percentile(&widths, 0.90);
    if p10 <= 0.0 {
        return None;
    }
    let contrast = p90 / p10;

    let thin_positions: Vec<_> = positions
        .into_iter()
        .filter(|&(_, _, dist)| dist <= p10)
        .collect();
    let angle = stress_angle(&thin_positions);
    Some((contrast, angle))
}

fn flatten_glyph(glyph: &BezGlyph) -> Vec<Vec<Point>> {
    let mut contours = Vec::new();
    for path in glyph.iter() {
        let bbox = path.bounding_box();
        let tol = (bbox.width().max(bbox.height()) * 0.0005).max(0.05);
        let mut current = Vec::new();
        kurbo::flatten(path.iter(), tol, |el| match el {
            PathEl::MoveTo(p) => {
                if !current.is_empty() {
                    contours.push(std::mem::take(&mut current));
                }
                current.push(p);
            }
            PathEl::LineTo(p) => current.push(p),
            PathEl::ClosePath if !current.is_empty() => {
                contours.push(std::mem::take(&mut current));
            }
            _ => {}
        });
        if !current.is_empty() {
            contours.push(current);
        }
    }
    contours.retain(|c| c.len() >= 3);
    contours
}

fn poly_bbox(poly: &[Point]) -> kurbo::Rect {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in poly {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    kurbo::Rect::new(min_x, min_y, max_x, max_y)
}

fn polygon_area(poly: &[Point]) -> f64 {
    let mut area = 0.0;
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        area += a.x * b.y - b.x * a.y;
    }
    area / 2.0
}

fn poly_centroid(poly: &[Point]) -> Point {
    let mut cx = 0.0;
    let mut cy = 0.0;
    for p in poly {
        cx += p.x;
        cy += p.y;
    }
    let n = poly.len().max(1) as f64;
    Point::new(cx / n, cy / n)
}

fn point_in_polygon(p: Point, poly: &[Point]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (poly[i].x, poly[i].y);
        let (xj, yj) = (poly[j].x, poly[j].y);
        if (yi > p.y) != (yj > p.y) && p.x < (xj - xi) * (p.y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn resample_poly(poly: &[Point], spacing: f64) -> Vec<Point> {
    let mut out = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let len = a.distance(b);
        let steps = ((len / spacing).round() as usize).max(1);
        for k in 0..steps {
            let t = k as f64 / steps as f64;
            out.push(a.lerp(b, t));
        }
    }
    if out.len() < 3 {
        out = poly.to_vec();
    }
    out
}

fn smooth_poly(poly: &[Point], window: f64) -> Vec<Point> {
    let n = poly.len();
    if n < 5 || window <= 0.0 {
        return poly.to_vec();
    }
    // Cumulative arc length
    let mut cum = Vec::with_capacity(n + 1);
    cum.push(0.0);
    for i in 0..n {
        cum.push(cum[i] + poly[i].distance(poly[(i + 1) % n]));
    }
    let total = cum[n];
    if total <= 0.0 {
        return poly.to_vec();
    }
    let samples = 9;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let center = (cum[i] + cum[i + 1]) / 2.0;
        let mut sx = 0.0;
        let mut sy = 0.0;
        for k in 0..samples {
            let s = center + window * (k as f64 / (samples - 1) as f64 - 0.5);
            let p = point_at_arc(poly, &cum, total, s);
            sx += p.x;
            sy += p.y;
        }
        out.push(Point::new(sx / samples as f64, sy / samples as f64));
    }
    out
}

fn point_at_arc(poly: &[Point], cum: &[f64], total: f64, s: f64) -> Point {
    let s = s.rem_euclid(total);
    // Binary search for the segment containing s
    let mut lo = 0usize;
    let mut hi = poly.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if cum[mid] <= s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let a = poly[lo];
    let b = poly[(lo + 1) % poly.len()];
    let seg = cum[lo + 1] - cum[lo];
    let t = if seg > 0.0 { (s - cum[lo]) / seg } else { 0.0 };
    a.lerp(b, t)
}

fn min_distance_to_poly(points: &[Point], poly: &[Point]) -> Option<f64> {
    let mut best = f64::INFINITY;
    let n = poly.len();
    for p in points {
        for i in 0..n {
            let d = point_segment_distance(*p, poly[i], poly[(i + 1) % n]);
            best = best.min(d);
        }
    }
    (best < f64::INFINITY / 2.0).then_some(best)
}

fn point_segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    if len2 == 0.0 {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

#[allow(clippy::needless_range_loop)]
fn rasterize(contours: &[Vec<Point>], gsize: f64, bbox: &kurbo::Rect) -> (Vec<u8>, usize, usize, f64) {
    let scale = (TARGET_RES * SS) as f64 / gsize;
    let w = ((bbox.width() * scale).ceil() as usize + 2 * MARGIN).max(1);
    let h = ((bbox.height() * scale).ceil() as usize + 2 * MARGIN).max(1);
    let min_x = bbox.min_x();
    let min_y = bbox.min_y();
    let map_x = |x: f64| (x - min_x) * scale + MARGIN as f64;
    let map_y = |y: f64| (y - min_y) * scale + MARGIN as f64;

    let mut crossings: Vec<Vec<(f64, f64)>> = vec![Vec::new(); h];
    for poly in contours {
        let n = poly.len();
        for i in 0..n {
            let x1 = map_x(poly[i].x);
            let y1 = map_y(poly[i].y);
            let x2 = map_x(poly[(i + 1) % n].x);
            let y2 = map_y(poly[(i + 1) % n].y);
            if (y2 - y1).abs() < 1e-9 {
                continue;
            }
            let sign = if y2 > y1 { 1.0 } else { -1.0 };
            let (ylo, yhi) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            let yy0 = ((ylo - 0.5).ceil() as isize).max(0) as usize;
            let yy1 = (((yhi - 0.5).ceil() as isize).max(0) as usize).min(h);
            for yy in yy0..yy1 {
                let cy = yy as f64 + 0.5;
                let t = (cy - y1) / (y2 - y1);
                let xx = x1 + t * (x2 - x1);
                crossings[yy].push((xx, sign));
            }
        }
    }

    let mut hi = vec![0u8; w * h];
    for (yy, row) in crossings.iter_mut().enumerate() {
        row.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut winding = 0.0f64;
        for k in 0..row.len().saturating_sub(1) {
            winding += row[k].1;
            if winding.abs() > 1e-9 {
                let xa = ((row[k].0 - 0.5).ceil() as isize).max(0) as usize;
                let xb = ((row[k + 1].0 - 0.5).floor() as isize).max(0) as usize;
                if xb >= xa {
                    let xb = xb.min(w - 1);
                    hi[yy * w + xa..=yy * w + xb].fill(1);
                }
            }
        }
    }

    // Supersample down with an "any" reduction so thin strokes stay connected.
    let w2 = w / SS;
    let h2 = h / SS;
    let mut mask = vec![0u8; w2 * h2];
    for yy in 0..h2 {
        for xx in 0..w2 {
            let mut v = 0u8;
            for sy in 0..SS {
                for sx in 0..SS {
                    v |= hi[(yy * SS + sy) * w + (xx * SS + sx)];
                }
            }
            mask[yy * w2 + xx] = v;
        }
    }
    let final_scale = scale / SS as f64;
    (mask, w2, h2, final_scale)
}

fn binary_closing(mask: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut dilated = mask.to_vec();
    for y in 0..h {
        for x in 0..w {
            if mask[y * w + x] != 0 {
                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0 && ny < h as isize && nx >= 0 && nx < w as isize {
                            dilated[ny as usize * w + nx as usize] = 1;
                        }
                    }
                }
            }
        }
    }
    let mut eroded = dilated.clone();
    for y in 0..h {
        for x in 0..w {
            let mut all = true;
            'outer: for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let ny = y as isize + dy;
                    let nx = x as isize + dx;
                    if ny < 0 || ny >= h as isize || nx < 0 || nx >= w as isize {
                        all = false;
                        break 'outer;
                    }
                    if dilated[ny as usize * w + nx as usize] == 0 {
                        all = false;
                        break 'outer;
                    }
                }
            }
            eroded[y * w + x] = if all { 1 } else { 0 };
        }
    }
    eroded
}

fn count_holes(mask: &[u8], w: usize, h: usize) -> usize {
    let mut visited = vec![0u8; w * h];
    let mut holes = 0usize;
    let mut queue = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if mask[idx] == 0 && visited[idx] == 0 {
                // New background component; flood fill and check whether it
                // touches the border.
                queue.clear();
                queue.push(idx);
                visited[idx] = 1;
                let mut touches_border = false;
                let mut qi = 0usize;
                while qi < queue.len() {
                    let cur = queue[qi];
                    qi += 1;
                    let cx = cur % w;
                    let cy = cur / w;
                    if cx == 0 || cy == 0 || cx + 1 == w || cy + 1 == h {
                        touches_border = true;
                    }
                    let cx = cx as isize;
                    let cy = cy as isize;
                    for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                        let nx = cx + dx;
                        let ny = cy + dy;
                        if nx >= 0 && nx < w as isize && ny >= 0 && ny < h as isize {
                            let nidx = ny as usize * w + nx as usize;
                            if mask[nidx] == 0 && visited[nidx] == 0 {
                                visited[nidx] = 1;
                                queue.push(nidx);
                            }
                        }
                    }
                }
                if !touches_border {
                    holes += 1;
                }
            }
        }
    }
    holes
}

fn chamfer_distance(mask: &[u8], w: usize, h: usize) -> Vec<f32> {
    let mut dt = vec![INF; w * h];
    for (i, &m) in mask.iter().enumerate() {
        if m == 0 {
            dt[i] = 0.0;
        }
    }
    // Forward pass
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if dt[i] == 0.0 {
                continue;
            }
            let mut v = dt[i];
            if x > 0 {
                v = v.min(dt[i - 1] + 3.0);
            }
            if y > 0 {
                v = v.min(dt[i - w] + 3.0);
                if x > 0 {
                    v = v.min(dt[i - w - 1] + 4.0);
                }
                if x + 1 < w {
                    v = v.min(dt[i - w + 1] + 4.0);
                }
            }
            dt[i] = v;
        }
    }
    // Backward pass
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if dt[i] == 0.0 {
                continue;
            }
            let mut v = dt[i];
            if x + 1 < w {
                v = v.min(dt[i + 1] + 3.0);
            }
            if y + 1 < h {
                v = v.min(dt[i + w] + 3.0);
                if x + 1 < w {
                    v = v.min(dt[i + w + 1] + 4.0);
                }
                if x > 0 {
                    v = v.min(dt[i + w - 1] + 4.0);
                }
            }
            dt[i] = v;
        }
    }
    dt
}

fn zhang_suen(mask: &[u8], w: usize, h: usize) -> Vec<u8> {
    let (g123, g123p) = thinning_luts();
    let mut img = mask.to_vec();
    let mut changed = true;
    let mut marker = vec![false; w * h];
    while changed {
        changed = false;
        for lut in [&g123, &g123p] {
            marker.fill(false);
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    if img[i] == 0 {
                        continue;
                    }
                    let idx = (u8::from(img[i - w] != 0))
                        | (u8::from(img[i - w + 1] != 0) << 1)
                        | (u8::from(img[i + 1] != 0) << 2)
                        | (u8::from(img[i + w + 1] != 0) << 3)
                        | (u8::from(img[i + w] != 0) << 4)
                        | (u8::from(img[i + w - 1] != 0) << 5)
                        | (u8::from(img[i - 1] != 0) << 6)
                        | (u8::from(img[i - w - 1] != 0) << 7);
                    if lut[idx as usize] {
                        marker[i] = true;
                    }
                }
            }
            for (i, &m) in marker.iter().enumerate() {
                if m {
                    img[i] = 0;
                    changed = true;
                }
            }
        }
    }
    img
}

fn thinning_luts() -> (Vec<bool>, Vec<bool>) {
    let mut g123 = vec![false; 256];
    let mut g123p = vec![false; 256];
    for n in 0..256usize {
        let bits: [bool; 9] = std::array::from_fn(|i| n >> i & 1 != 0);
        let mut s = 0;
        for i in [0usize, 2, 4, 6] {
            if !bits[i] && (bits[i + 1] || bits[(i + 2) % 8]) {
                s += 1;
            }
        }
        let g1 = s == 1;
        let (mut n1, mut n2) = (0, 0);
        for k in [1usize, 3, 5, 7] {
            if bits[k] || bits[k - 1] {
                n1 += 1;
            }
            if bits[k] || bits[(k + 1) % 8] {
                n2 += 1;
            }
        }
        let g2 = matches!(n1.min(n2), 2 | 3);
        let g3 = !((bits[1] || bits[2] || !bits[7]) && bits[0]);
        let g3p = !((bits[5] || bits[6] || !bits[3]) && bits[4]);
        g123[n] = g1 && g2 && g3;
        g123p[n] = g1 && g2 && g3p;
    }
    (g123, g123p)
}

fn prune_endpoints(img: &mut [u8], w: usize, h: usize) {
    loop {
        let mut removed = false;
        let mut endpoints = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if img[i] == 0 {
                    continue;
                }
                let mut count = 0;
                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0 && ny < h as isize && nx >= 0 && nx < w as isize
                            && img[ny as usize * w + nx as usize] != 0
                        {
                            count += 1;
                        }
                    }
                }
                if count == 1 {
                    endpoints.push(i);
                }
            }
        }
        for i in endpoints {
            img[i] = 0;
            removed = true;
        }
        if !removed {
            break;
        }
    }
}

fn percentile(sorted: &[f32], q: f64) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn stress_angle(thin: &[(usize, usize, f32)]) -> Option<f32> {
    if thin.len() < 4 {
        return None;
    }
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    for &(x, y, _) in thin {
        cx += x as f64;
        cy += y as f64;
    }
    let n = thin.len() as f64;
    cx /= n;
    cy /= n;
    let mut doubled = Vec::with_capacity(thin.len());
    for &(x, y, _) in thin {
        let dx = x as f64 - cx;
        let dy = y as f64 - cy;
        if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
            continue;
        }
        let theta = dy.atan2(dx).to_degrees(); // -180..180
        let theta = if theta < 0.0 { theta + 180.0 } else { theta }; // 0..180
        doubled.push(2.0 * theta);
    }
    if doubled.is_empty() {
        return None;
    }
    doubled.sort_by(|a, b| a.total_cmp(b));
    let median = doubled[doubled.len() / 2] / 2.0;
    Some(median as f32)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::bezglyph::BezGlyph;
    use kurbo::BezPath;

    fn svg_to_bezglyph(svg: &str) -> BezGlyph {
        let mut bezpaths = Vec::new();
        for chunk in svg.split_inclusive('Z') {
            if !chunk.trim().is_empty() {
                bezpaths.push(BezPath::from_svg(chunk).expect("valid svg path"));
            }
        }
        BezGlyph(bezpaths)
    }

    #[test]
    fn uniform_ring_is_monoline() {
        // A circular ring: outer radius 500, inner radius 300, so the stroke
        // is a uniform 200 units everywhere.
        let svg = "M -500 0 C -500 -276.142 -276.142 -500 0 -500 C 276.142 -500 500 -276.142 500 0 C 500 276.142 276.142 500 0 500 C -276.142 500 -500 276.142 -500 0 Z M -300 0 C -300 165.685 -165.685 300 0 300 C 165.685 300 300 165.685 300 0 C 300 -165.685 165.685 -300 0 -300 C -165.685 -300 -300 -165.685 -300 0 Z";
        let glyph = svg_to_bezglyph(svg);
        let (contrast, _) = stroke_contrast_medial(&glyph).expect("measurable ring");
        assert!(contrast > 0.9 && contrast < 1.2, "contrast {contrast}");
    }

    #[test]
    fn stressed_ring_is_detected() {
        // A circle (rx=500) with a vertically-squeezed counter (rx=300,
        // ry=250): vertical strokes are 250 thick and horizontal strokes 200
        // thick, so the contrast is around 1.25.
        let svg = "M -500 0 C -500 -276.142 -276.142 -500 0 -500 C 276.142 -500 500 -276.142 500 0 C 500 276.142 276.142 500 0 500 C -276.142 500 -500 276.142 -500 0 Z M -300 0 C -300 138.071 -138.071 250 0 250 C 138.071 250 300 138.071 300 0 C 300 -138.071 138.071 -250 0 -250 C -138.071 -250 -300 -138.071 -300 0 Z";
        let glyph = svg_to_bezglyph(svg);
        let (contrast, _) = stroke_contrast_medial(&glyph).expect("measurable ring");
        assert!(contrast > 1.05 && contrast < 1.6, "contrast {contrast}");
    }
}
