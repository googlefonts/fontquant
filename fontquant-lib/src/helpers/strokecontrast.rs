use std::io::Write;

use crate::{
    bezglyph::BezGlyph,
    helpers::{
        all_intersections,
        raycaster::{self, Raycaster},
        remove_outliers,
    },
};
use itertools::Itertools;
use kurbo::{flatten, Affine, BezPath, Line, Point, Shape};
use statistical::median;

const TOLERANCE: f64 = 0.1;

type ShowStrokesList<'a> = Box<dyn Fn(&[Stroke]) + 'a>;
type ShowMinSecondMax<'a> = Box<dyn Fn(&Stroke, Option<&Stroke>, &Stroke) + 'a>;
struct DebuggingHook<'a> {
    show_strokes_list: ShowStrokesList<'a>,
    show_min_second_max: ShowMinSecondMax<'a>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Stroke {
    half_way: Point,
    p1: Point,
    p2: Point,
    distance: f32,
    position: usize,
}

pub fn stroke_contrast_antiqua(glyph: &BezGlyph) -> Option<(f32, Option<f32>)> {
    _stroke_contrast_antiqua(glyph, None)
}

fn _stroke_contrast_antiqua(
    glyph: &BezGlyph,
    hook: Option<&DebuggingHook>,
) -> Option<(f32, Option<f32>)> {
    let mut paths: Vec<&BezPath> = glyph.0.iter().collect();
    // Sort by bbox width
    paths.sort_by(|a, b| {
        b.bounding_box()
            .width()
            .total_cmp(&a.bounding_box().width())
    });
    if paths.len() < 2 {
        return None;
    }
    let mut skeleton_points = vec![];
    flatten(paths[0].iter(), TOLERANCE, |path| {
        if let Some(e) = path.end_point() {
            skeleton_points.push(e)
        }
    });

    let mut strokes_list = vec![];
    for (prev_point, point) in skeleton_points.iter().circular_tuple_windows() {
        let distance = prev_point.distance(*point);
        let mut angles_list = vec![];
        if distance == 0.0 {
            continue;
        }
        let half_way = prev_point.lerp(*point, 0.5);
        for angle in -30..31 {
            let outside_point =
                Affine::rotate_about(((90 + angle) as f64).to_radians(), half_way) * *point;
            let distance = outside_point.distance(half_way);
            if distance != 0.0 {
                let far_outside_pt1 = half_way.lerp(outside_point, 5000.0 / distance);
                let far_outside_pt2 = half_way.lerp(outside_point, -5000.0 / distance);
                let measurement_line = Line::new(far_outside_pt1, far_outside_pt2);
                let mut intersections: Vec<Point> = all_intersections(&paths, &measurement_line);
                intersections.sort_by(|a, b| a.distance(half_way).total_cmp(&b.distance(half_way)));
                if intersections.len() > 1 {
                    let first = intersections[0];
                    let second = intersections[1];
                    let distance = first.distance(second);
                    if distance != 0.0 {
                        angles_list.push((first, second, distance));
                    }
                }
            }
            remove_outliers(&mut angles_list, |(_, _, d)| *d);
        }
        // Add shortest line to strokes list
        if let Some((first, second, distance)) = angles_list
            .iter()
            .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(d2))
        {
            strokes_list.push(Stroke {
                half_way,
                p1: *first,
                p2: *second,
                distance: *distance as f32,
                position: 0,
            });
        }
    }

    remove_outliers(&mut strokes_list, |s| s.distance as f64);
    if let Some(hook) = hook {
        (hook.show_strokes_list)(&strokes_list);
    }
    if strokes_list.len() < 2 {
        return None;
    }

    let mut indices = (0..strokes_list.len()).collect::<Vec<_>>();
    indices.sort_by(|a, b| {
        strokes_list[*a]
            .distance
            .total_cmp(&strokes_list[*b].distance)
    });
    let min_index = indices[0];
    let max_index = indices[indices.len() - 1];
    // Find the "second minimum": distance should be similar but index must be far away
    let mut second_min_index = None;
    for index in indices.into_iter().skip(1) {
        if (index as isize - min_index as isize) % (strokes_list.len() as isize)
            > (strokes_list.len() / 4) as isize
        {
            second_min_index = Some(index);
            break;
        }
    }

    if let Some(hook) = hook {
        (hook.show_min_second_max)(
            &strokes_list[min_index],
            second_min_index.and_then(|i| strokes_list.get(i)),
            &strokes_list[max_index],
        );
    }

    let angle = second_min_index.map(|second_min_index| {
        let min = strokes_list[min_index].half_way;
        let second_min = strokes_list[second_min_index].half_way;
        (min - second_min).angle() as f32
    });
    let contrast = strokes_list[max_index].distance / strokes_list[min_index].distance;
    Some((contrast, angle))
}

#[allow(clippy::unwrap_used)]
pub fn stroke_contrast_raycaster(
    raycaster_north: &mut Raycaster,
    raycaster_east: &mut Raycaster,
) -> Option<f32> {
    raycaster_north.winding(raycaster::Winding::Ink);
    raycaster_east.winding(raycaster::Winding::Ink);

    if log::log_enabled!(log::Level::Trace) {
        let data = raycaster_north.draw();
        let mut file = std::fs::File::create("rc-north.png").unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }

    let mut distances = raycaster_north.distances();
    // If any pairs have the same X coords as any others, keep the pair with the smallest distance
    distances.sort_by(|a, b| a.0.x.total_cmp(&b.0.x).then(a.2.total_cmp(&b.2)));
    distances.dedup_by(|a, b| a.0.x == b.0.x && a.1.x == b.1.x);
    let mut distances = distances
        .iter()
        .map(|(_, _, d)| *d as f32)
        .collect::<Vec<_>>();
    remove_outliers(&mut distances, |d| *d as f64);
    log::trace!("North distances (before outlier removal): {:?}", distances);
    log::trace!("North distances: {:?}", distances);
    if distances.is_empty() {
        return None;
    }
    let horizontal_thickness = median(&distances) as f64;
    log::trace!("Horizontal thickness: {:?}", horizontal_thickness);

    if log::log_enabled!(log::Level::Trace) {
        let data = raycaster_east.draw();
        let mut file = std::fs::File::create("rc-east.png").unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }

    let vertical_thickness = raycaster_east.median_pair_distance(true);

    if horizontal_thickness == 0.0 {
        return None;
    }
    let contrast = vertical_thickness / horizontal_thickness;
    log::trace!("Vertical thickness: {:?}", vertical_thickness);
    log::trace!("Contrast: {:?}", contrast);
    Some(contrast as f32)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use std::io::Write;

    use crate::helpers::raycaster::{ProportionalPoint, EAST, NORTH};
    use kurbo::{BezPath, Insets, SvgParseError};
    use skia_safe::{EncodedImageFormat, PaintStyle};

    use crate::{bezglyph::bezpaths_to_skpath, helpers::k2s};

    use super::*;

    // EB Garamond O. Thickest stroke: 84 units; thinnest stroke: 27 units. (3.11x)
    // Thickness at north: 30 units; thickness at west: 83 units. (2.77x)
    // Roughly: broad nib at width = 85, height = 25, pen angle = 15 degrees.
    const EBGARAMOND_O: &str = "M234 -14Q178 -14 133 12Q88 38 61 83Q35 129 35 187Q35 229 51 269Q68 309 98 342Q128 375 168 394Q208 414 254 414Q312 414 358 386Q405 358 432 312Q460 267 460 213Q460 155 434 103Q409 51 359 18Q309 -14 234 -14ZM255 16Q289 16 316 29Q343 42 358 71Q370 94 374 127Q378 160 378 189Q378 237 360 281Q343 326 311 354Q280 383 237 383Q210 383 188 374Q167 366 149 343Q129 318 123 282Q117 246 117 210Q117 161 135 116Q153 72 184 44Q215 16 255 16Z";

    // Lobster's O. This is a pain because it has a thin outstroke in the northeast which should be
    // ignored for contrast calculation. Thickest stroke: 150; thinnest (real) stroke: 61 (2.46x)
    // Thickness at north: 62; thickness at west: 146 (2.35x)
    const LOBSTER_O: &str = "M29 34Q-14 72 -14 154Q-14 222 14 305Q42 388 104 448Q166 509 260 509Q388 509 388 349L388 348Q391 347 399 347Q432 347 475 364Q518 382 553 407L562 380Q533 349 486 327Q440 306 385 297Q377 206 344 138Q311 70 259 33Q207 -4 146 -4Q72 -4 29 34ZM234 113Q258 141 276 188Q294 236 302 294Q275 300 275 336Q275 377 306 390Q304 423 296 435Q288 448 268 448Q235 448 204 400Q173 352 154 285Q135 218 135 167Q135 119 146 102Q157 86 186 86Q210 86 234 113Z";

    fn svg_to_bezglyph(svg: &str) -> Result<BezGlyph, SvgParseError> {
        // Split at Zs
        let mut bezpaths = Vec::new();
        let mut current_path = String::new();
        for c in svg.chars() {
            current_path.push(c);
            if c == 'Z' {
                bezpaths.push(BezPath::from_svg(&current_path)?);
            }
        }
        if !current_path.is_empty() {
            bezpaths.push(BezPath::from_svg(&current_path)?);
        }
        Ok(BezGlyph(bezpaths))
    }

    #[test]
    fn test_contrast() {
        let glyph = svg_to_bezglyph(LOBSTER_O).unwrap();
        let bbox = glyph.bbox().unwrap().inset(Insets::uniform(20.0));
        let mut surface =
            skia_safe::surfaces::raster_n32_premul((bbox.width() as i32, bbox.height() as i32))
                .expect("surface");
        let canvas = surface.canvas();
        canvas.clear(skia_safe::Color::WHITE);
        let mut transform = skia_safe::M44::scale(1.0, -1.0, 1.0);
        transform.post_translate(
            -bbox.min_x() as f32,
            bbox.min_y() as f32 + bbox.height() as f32,
            0.0,
        );
        canvas.set_matrix(&transform);
        let skia_path = bezpaths_to_skpath(&glyph.0);
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.3, 0.3, 0.3, 1.0), None);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke(true);
        paint.set_stroke_width(2.0);
        canvas.draw_path(&skia_path, &paint);

        let debughooks = DebuggingHook {
            show_strokes_list: (Box::new(|strokes| {
                let mut paint =
                    skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.5), None);
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(1.0);
                for stroke in strokes {
                    canvas.draw_line(k2s(stroke.p1), k2s(stroke.p2), &paint);
                }
            })),
            show_min_second_max: (Box::new(|min, second_min, max| {
                let mut red =
                    skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                red.set_style(PaintStyle::Stroke);
                red.set_stroke_width(4.0);
                canvas.draw_line(k2s(max.p1), k2s(max.p2), &red);
                if let Some(second_min) = second_min {
                    let mut grey =
                        skia_safe::Paint::new(skia_safe::Color4f::new(0.5, 0.5, 0.5, 1.0), None);
                    grey.set_style(PaintStyle::Stroke);
                    grey.set_stroke_width(4.0);
                    canvas.draw_line(k2s(second_min.p1), k2s(second_min.p2), &grey);
                    grey.set_stroke_width(1.0);
                    canvas.draw_line(k2s(min.half_way), k2s(second_min.half_way), &grey);
                }
                let mut green =
                    skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                green.set_style(PaintStyle::Stroke);
                green.set_stroke_width(4.0);
                canvas.draw_line(k2s(min.p1), k2s(min.p2), &green);
            })),
        };

        let (contrast, angle) = _stroke_contrast_antiqua(&glyph, Some(&debughooks)).unwrap();

        std::mem::forget(debughooks);

        let image = surface.image_snapshot();
        let mut context = surface.direct_context();
        let data = image
            .encode(context.as_mut(), EncodedImageFormat::PNG, None)
            .unwrap();
        let mut file = std::fs::File::create("test.png").unwrap();
        file.write_all(&data).expect("write to file");

        let mut raycaster_north = Raycaster::new(&glyph, ProportionalPoint::new(0.5, 0.0), NORTH);
        raycaster_north.jitter(0.2, 20);
        let mut raycaster_east = Raycaster::new(&glyph, ProportionalPoint::new(0.0, 0.5), EAST);
        raycaster_east.jitter(0.2, 50);
        let contrast =
            stroke_contrast_raycaster(&mut raycaster_north, &mut raycaster_east).unwrap();

        assert!(contrast > 2.0);
        assert!(contrast < 3.5);
        assert!(
            angle.unwrap().to_degrees() > 10.0 && angle.unwrap().to_degrees() < 20.0,
            "Contrast angle: {:?}",
            angle.unwrap()
        );
    }
}
