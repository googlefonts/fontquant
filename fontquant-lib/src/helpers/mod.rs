pub mod raycaster;
pub mod strokecontrast;

pub(crate) fn remove_outliers<T, F>(list: &mut Vec<T>, f: F)
where
    T: Clone,
    F: Fn(&T) -> f64,
{
    if list.len() < 3 {
        return;
    }
    let distances = list.iter().map(f).collect::<Vec<_>>();
    // Get sort order
    let mut indices = (0..distances.len()).collect::<Vec<_>>();
    indices.sort_by(|a, b| distances[*a].total_cmp(&distances[*b]));

    let q1 = distances[indices[distances.len() / 4]];
    let q3 = distances[indices[distances.len() * 3 / 4]];
    let iqr = q3 - q1;
    let lower_bound = q1 - iqr * 1.5;
    let upper_bound = q3 + iqr * 1.5;
    let mut keep = distances
        .iter()
        .map(|&d| d >= lower_bound && d <= upper_bound);
    #[allow(clippy::unwrap_used)]
    list.retain(|_| keep.next().unwrap());
}

#[allow(dead_code)] // Used in tests
fn k2s(pt: kurbo::Point) -> skia_safe::Point {
    skia_safe::Point::new(pt.x as f32, pt.y as f32)
}
#[allow(dead_code)] // Used in tests
fn s2k(pt: skia_safe::Point) -> kurbo::Point {
    kurbo::Point::new(pt.x as f64, pt.y as f64)
}
