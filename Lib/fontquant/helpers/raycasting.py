import re
from kurbopy import Point, Vec2, Line
import statistics
import math

# There is a lot of (potentially unnecessary) scaffolding here to hopefully
# make the final algorithms more readable.
angles = {
    "E": 0,
    "NE": 45,
    "N": 90,
    "NW": 135,
    "W": 180,
    "SW": 225,
    "S": 270,
    "SE": 315,
}

EPSILON = 5


def proportional_point_to_point(point, bbox, is_endpoint=False, direction=None):
    if isinstance(point, Point) or point is None:
        return point
    # Proportions of the bounding box
    if isinstance(point, tuple) and isinstance(point[0], (float, int)) and isinstance(point[1], (float, int)):
        return Point(
            bbox.min_x() + bbox.width() * point[0],
            bbox.min_y() + bbox.height() * point[1],
        )
    raise ValueError("Invalid point")


def cast_ray(paths, point, endpoint=None, direction=0, pairs=False, jittering=False):
    # Find overall bounding box
    bbox = paths[0].bounding_box()
    for path in paths[1:]:
        bbox = bbox.union(path.bounding_box())
    if direction in angles:
        direction = angles[direction]
    elif isinstance(direction, str):
        raise ValueError("Invalid angle")

    point = proportional_point_to_point(point, bbox, direction=direction)

    vector = Vec2.from_angle(math.radians(direction))

    if endpoint is not None:
        endpoint = proportional_point_to_point(endpoint, bbox, is_endpoint=True, direction=direction)
    else:
        endpoint = point + vector * 100.0
        while bbox.contains(endpoint):
            endpoint += vector * 100.0

    ray = Line(point, endpoint)

    intersections = []

    for glyph_path in paths:
        intersections.extend(glyph_path.intersections(ray))

    # Drop any intersections which touch the endpoints
    intersections = [i for i in intersections if not (i.distance(point) < EPSILON or i.distance(endpoint) < EPSILON)]
    # Sort the intersections based on their position along the ray
    intersections.sort(key=lambda x: x.distance(point))

    if pairs:
        if len(intersections) < 2:
            return []
        if len(intersections) == 2:
            return [(intersections[0], intersections[1])]

        # Find point-to-point distances
        distances = []
        for i in range(0, len(intersections) - 1):
            distance = intersections[i].distance(intersections[i + 1])
            if distance > 0:
                distances.append((i, i + 1, distance))

        # Find sets of adjacent points which are a "sensible" distance
        # away from each other. If we are jittering, don't do this here
        # but do it on the whole set of distances
        if not jittering:
            distances = do_drop_outliers(distances, key=lambda x: x[2])
        return [(intersections[d[0]], intersections[d[1]]) for d in distances]
    return intersections


def do_drop_outliers(distances, key=lambda x: x):
    if len(distances) < 3:
        return distances
    # If there is a single modal value, take it
    if len(statistics.multimode([key(d) for d in distances])) == 1:
        return [d for d in distances if key(d) == statistics.mode([key(d) for d in distances])]
    median = statistics.median([key(d) for d in distances])
    stdev = statistics.stdev([key(d) for d in distances])
    if stdev == 0:
        return distances
    return [d for d in distances if abs(key(d) - median) < 1 * stdev]


def paired_intersection_mean(intersections, drop_outliers=False):
    distances = [p[0].distance(p[1]) for p in intersections]
    if drop_outliers:
        distances = do_drop_outliers(distances)
    return statistics.mean(distances)


def jitter_point(point, direction, jittering=0.2, samples=10):
    if point is None:
        yield None
    if isinstance(point, tuple):
        if direction == "S" or direction == "N":
            start = point[0] * (1 - jittering)
            end = point[0] * (1 + jittering)
            step = (end - start) / samples
            for i in range(samples):
                yield (start + (i * step), point[1])
        else:
            start = point[1] * (1 - jittering)
            end = point[1] * (1 + jittering)
            step = (end - start) / samples
            for i in range(samples):
                yield (point[0], start + (i * step))
    return point


# Try to avoid the effect of serifs etc. by jittering the start and end points
def jittered_distance(paths, point, endpoint=None, direction=0, jittering=0.2, samples=10):
    means = []
    starts = jitter_point(point, direction=direction, jittering=jittering, samples=samples)
    if endpoint is None:
        endpoints = [None] * samples
    else:
        endpoints = jitter_point(endpoint, direction=direction, jittering=jittering, samples=samples)
    for start, end in zip(starts, endpoints):
        try:
            means.extend(cast_ray(paths, start, end, direction, pairs=True, jittering=True))
        except statistics.StatisticsError as e:
            pass
    return paired_intersection_mean(means, drop_outliers=True)


if __name__ == "__main__":

    from fontquant import CustomTTFont, CustomHarfbuzz
    import sys
    from kurbopy import BezPathCreatingPen
    import argparse

    parser = argparse.ArgumentParser(description="Raycasting test for fonts")
    parser.add_argument("--variations", nargs="?", help="Variation settings")
    parser.add_argument("font", help="Path to the font file")

    args = parser.parse_args()
    ttFont = CustomTTFont(args.font)
    vhb = CustomHarfbuzz(args.font)
    variations = {}
    if args.variations:
        for var in args.variations.split(","):
            k, v = var.split("=")
            variations[k] = float(v)

    def paths_for_glyph(char):
        buf = vhb.shape(char, {"variations": variations})
        return vhb.buf_to_bezierpaths(buf, penclass=BezPathCreatingPen)

    try:
        xopq = jittered_distance(paths_for_glyph("H"), point=(0, 0.25), direction="E")
        print("XOPQ: ", xopq)
    except statistics.StatisticsError as e:
        print("Could not determine XOPQ")

    try:
        xolc = jittered_distance(paths_for_glyph("n"), point=(0, 0.25), direction="E")
        print("XOLC: ", xolc)
    except statistics.StatisticsError as e:
        print("Could not determine XOLC")

    try:
        xofi = jittered_distance(paths_for_glyph("1"), point=(0, 0.5), direction="E")
        print("XOFI: ", xofi)
    except statistics.StatisticsError as e:
        print("Could not determine XOFI")

    try:
        # Sweep the whole glyph
        yopq = jittered_distance(
            paths_for_glyph("H"), (0.5, 0.25), endpoint=(0.5, 0.75), direction="N", jittering=1, samples=10
        )
        print("YOPQ: ", yopq)
    except statistics.StatisticsError as e:
        print("Could not determine YOPQ")

    try:
        yolc = jittered_distance(
            paths_for_glyph("f"), (0.75, 0.25), endpoint=(0.75, 0.8), direction="N", jittering=0.5, samples=10
        )
        print("YOLC: ", yolc)
    except statistics.StatisticsError as e:
        print("Could not determine YOLC")
