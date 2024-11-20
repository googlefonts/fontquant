from dataclasses import dataclass
import math
import statistics
from typing import List, Optional, Tuple, Union

from kurbopy import Line, Point, Vec2, BezPath, Rect, Insets

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


@dataclass
class ProportionalPoint:
    x: float
    y: float

    def to_point(self, bbox) -> Point:
        return Point(
            bbox.min_x() + bbox.width() * self.x,
            bbox.min_y() + bbox.height() * self.y,
        )


# We'll be using the median a lot instead of the mean because it's
# more robust to outliers.
def drop_outliers(distances, key=lambda x: x, deviations=0.1):
    if len(distances) < 3:
        return distances
    # Otherwise, drop anything n standard deviations away from the median
    median = statistics.median([round(key(d)) for d in distances])
    stdev = statistics.stdev([round(key(d)) for d in distances])
    if stdev == 0:
        return distances

    while deviations < 2:
        value = [d for d in distances if abs(round(key(d)) - median) < stdev * deviations]
        if len(value) == 0:
            deviations += 0.1
            continue
        return value
    return distances


def t_of_point(line: Line, pt: Point) -> float:
    if not math.isclose(line.end().x, line.start().x):
        t = (pt.x - line.start().x) / (line.end().x - line.start().x)
    elif not math.isclose(line.end().y, line.start().y):
        t = (pt.y - line.start().y) / (line.end().y - line.start().y)
    else:
        t = 0
    return t


class Raycaster:
    start_pt: Point
    end_pt: Point
    direction: Vec2
    bbox: Rect
    winding: Optional[str]
    results: List[float]
    jittering: Optional[float]
    samples: int
    _intersections: List[List[Point]]  # One list per ray
    _pairs: List[List[Tuple[Point, Point]]]

    # We have a bunch of largely similar but subtly different things we want to do
    # with this class, so this is a rare appropriate use of the Builder Pattern!

    def __init__(
        self,
        paths: List[BezPath],
        startpoint: Tuple[float, float],
        endpoint: Optional[Tuple[float, float]] = None,
        direction: Optional[Union[str, int, float]] = None,
    ):
        self.paths = paths
        self.jittering = None
        self.samples = 1
        self._intersections = None
        self._pairs = None
        self._debuglog = {}
        # Find overall bounding box
        self.bbox = paths[0].bounding_box()
        for path in paths[1:]:
            self.bbox = self.bbox.union(path.bounding_box())
        self.bbox = self.bbox.inset(Insets.uniform(10))

        if direction in angles:
            direction = angles[direction]
        elif isinstance(direction, str):
            raise ValueError("Invalid angle")

        # Resolve startpoint, endpoint, direction
        if endpoint is not None and direction is not None:
            raise ValueError("Both endpoint and direction given")
        # Determine start and end points
        self.start_pt = ProportionalPoint(*startpoint)
        if endpoint is not None:
            self.end_pt = ProportionalPoint(*endpoint)
            self.direction = (
                self.end_pt.to_point(self.bbox).to_vec2() - self.start_pt.to_point(self.bbox).to_vec2()
            ).normalize()
        elif direction is not None:
            self.end_pt = None
            self.direction = Vec2.from_angle(math.radians(direction))
        else:
            raise ValueError("Either endpoint or direction must be given")

    def winding(self, winding: str) -> "Raycaster":
        if winding is not None and winding not in ("ink", "transparent"):
            raise ValueError("Invalid winding: must be either 'ink' or 'transparent'")
        self.winding = winding
        return self

    def jitter(self, jittering: float = 0.2, samples: int = 10) -> "Raycaster":
        self.samples = samples
        self.jittering = jittering
        return self

    def _create_ray(self, start_pt: ProportionalPoint, end_pt: Optional[ProportionalPoint]) -> Tuple[Line, Line]:
        # We're actually going to lie here, and create a ray which is entirely outside
        # the bounding box, so that the winding test always works correctly.
        s = start_pt.to_point(self.bbox)
        if end_pt is not None:
            e = end_pt.to_point(self.bbox)
        else:
            e = s + self.direction
            # This could be made faster by computing the intersection with the bounding box
            # of an unbounded ray.
            while self.bbox.contains(e):
                e += self.direction

        # Move start back and end forward by epsilon to avoid nibbling at the edges
        s -= self.direction * EPSILON
        e += self.direction * EPSILON
        original_start, original_end = s, e

        s -= self.direction * 300.0
        while self.bbox.contains(s):
            s -= self.direction * 300.0
        e += self.direction * 300.0
        while self.bbox.contains(e):
            e += self.direction * 300.0
        # But we also return the original start and end points so that we can
        # bound the intersections to them
        return Line(s, e), Line(original_start, original_end)

    def _create_starts_ends(self) -> List[Tuple[ProportionalPoint, Optional[ProportionalPoint]]]:
        if self.jittering is None:
            return [(self.start_pt, self.end_pt)]
        starts = []
        ends = []
        start = ProportionalPoint(self.start_pt.x, self.start_pt.y)
        # Rotate the direction vector 90 degrees
        perp = Vec2(-self.direction.y, self.direction.x)
        for i in range(self.samples):
            jitter = perp * (2 * (i - self.samples // 2) * self.jittering / self.samples)
            starts.append(ProportionalPoint(start.x + jitter.x, start.y + jitter.y))
            if self.end_pt is not None:
                end = ProportionalPoint(self.end_pt.x, self.end_pt.y)
                ends.append(ProportionalPoint(end.x + jitter.x, end.y + jitter.y))
            else:
                ends.append(None)
        return list(zip(starts, ends))

    def cast_ray(self) -> "Raycaster":
        # May cast multiple rays if jittering is enabled
        self._intersections = []
        rays = [self._create_ray(start, end) for start, end in self._create_starts_ends()]
        self._debuglog["rays"] = rays
        for ray, short_ray in rays:
            this_intersections = []
            for glyph_path in self.paths:
                this_intersections.extend(glyph_path.intersections(ray))
            # Sort the intersections based on their position along the ray
            this_intersections.sort(key=lambda x: x.distance(ray.start()))
            # Uniquify the intersections
            if this_intersections:
                this_intersections = [this_intersections[0]] + [
                    this_intersections[i]
                    for i in range(1, len(this_intersections))
                    if this_intersections[i].distance(this_intersections[i - 1]) > EPSILON
                ]
            # Bound the intersections to the original start and end points
            this_intersections = [
                i
                for i in this_intersections
                if 0 < t_of_point(short_ray, i) < 1 and i.distance(short_ray.start()) > EPSILON
            ]
            self._intersections.append(this_intersections)
        self._debuglog["intersections"] = self._intersections
        return self

    def pairs(self) -> "Raycaster":
        if self._intersections is None:
            self.cast_ray()
        self._pairs = []
        for intersections in self._intersections:
            if len(intersections) < 2:
                self._pairs.append([])
                continue
            if len(intersections) == 2 and (self.winding is None or self.winding == "ink"):
                self._pairs.append([(intersections[0], intersections[1])])
                continue
            # Find point-to-point distances
            distances = []
            for i in range(0, len(intersections) - 1):
                if self.winding == "ink" and i % 2 == 1:
                    continue
                if self.winding == "transparent" and i % 2 == 0:
                    continue
                distance = intersections[i].distance(intersections[i + 1])
                if distance > 0:
                    distances.append((i, i + 1, distance))

            self._pairs.append([(intersections[d[0]], intersections[d[1]]) for d in distances])
        return self

    def median_pair_distance(self, remove_outliers=True):
        if not self._pairs:
            self.pairs()

        distances = []
        for ray_pairs in self._pairs:
            if not ray_pairs:
                continue
            # We keep the pair for now so that when we drop outliers, we can
            # trace back to the original points
            distances.extend([(p, p[0].distance(p[1])) for p in ray_pairs])
        self._debuglog["distances"] = distances
        if drop_outliers:
            new_distances = drop_outliers(distances, key=lambda x: x[1])
            # The outliers were the ones we dropped...
            outliers = [p for p, d in distances if (p, d) not in new_distances]
            self._debuglog["outliers"] = outliers
            distances = new_distances
        return statistics.median([d for p, d in distances])

    def draw(self, ax, title=None, value=None):
        from matplotlib.patches import Rectangle

        ax.set_aspect("equal")
        if title:
            if value:
                ax.set_title(f"{title} = {value}")
            else:
                ax.set_title(title)
        for path in self.paths:
            path.plot(ax, color="#aaaaaa", linewidth=1)
        if "rays" in self._debuglog:
            all_short_rays = [short_ray for long_ray, short_ray in self._debuglog["rays"]]
            min_x = min(r.start().x for r in all_short_rays)
            min_y = min(r.start().y for r in all_short_rays)
            max_x = max(r.end().x for r in all_short_rays)
            max_y = max(r.end().y for r in all_short_rays)
            tall = max_x - min_x > max_y - min_y
            window = Rectangle(
                (min_x, min_y),
                abs(max_x - min_x),
                abs(max_y - min_y),
                fill=True,
                facecolor="#ffaaaa44",
                edgecolor="#44111144",
                linewidth=1,
                rotation_point="center",
                angle=math.degrees(self.direction.atan2()) + (0 if tall else 90),
            )
            ax.add_patch(window)
            for long_ray, short_ray in self._debuglog["rays"]:
                ax.plot(
                    [long_ray.start().x, long_ray.end().x],
                    [long_ray.start().y, long_ray.end().y],
                    "-",
                    linewidth=0.5,
                    color="#ff222233",
                )

        if "intersections" in self._debuglog:
            for intersections in self._debuglog["intersections"]:
                for intersection in intersections:
                    ax.plot(intersection.x, intersection.y, "o", color="#cccccc")
        if "distances" in self._debuglog:
            for ix, (pair, distance) in enumerate(self._debuglog["distances"]):
                is_outlier = pair in self._debuglog.get("outliers", {})
                midpoint = pair[0].lerp(pair[1], 0.5)
                # Add 10 units in the direction of the normal
                direction = pair[1].to_vec2() - pair[0].to_vec2()
                normal = Vec2(-direction.y, direction.x).normalize()
                midpoint = midpoint + normal * 10
                if is_outlier:
                    nodecolor = "#33337766"
                    edgecolor = "#8888ff66"
                    textcolor = "#33337766"
                    textsize = 6
                    markersize = 0
                else:
                    nodecolor = "r"
                    edgecolor = "r"
                    textcolor = "#000000"
                    textsize = 8
                    markersize = 5

                ax.plot([pair[0].x, pair[1].x], [pair[0].y, pair[1].y], "-", color=edgecolor)
                ax.plot(pair[0].x, pair[0].y, "o", color=nodecolor, markersize=markersize)
                ax.plot(pair[1].x, pair[1].y, "o", color=nodecolor, markersize=markersize)
                ax.text(
                    midpoint.x,
                    midpoint.y,
                    round(distance),
                    verticalalignment="center",
                    horizontalalignment="center",
                    fontsize=textsize,
                    color=textcolor,
                    rotation=math.degrees(math.atan2(direction.y, direction.x)),
                )
        return ax


if __name__ == "__main__":
    import argparse

    from kurbopy import BezPathCreatingPen

    from fontquant import CustomHarfbuzz, CustomTTFont
    from matplotlib import pyplot as plt

    parser = argparse.ArgumentParser(description="Raycasting test for fonts")
    parser.add_argument("--variations", nargs="?", help="Variation settings")
    parser.add_argument("--draw", help="Draw the results", default="")
    parser.add_argument("fonts", help="Path to the font file", nargs="+")

    args = parser.parse_args()

    def find_parametric_values(font):
        ttFont = CustomTTFont(font)
        vhb = CustomHarfbuzz(font)
        variations = {}
        if args.variations:
            for var in args.variations.split(","):
                k, v = var.split("=")
                variations[k] = float(v)

        def paths_for_glyph(char):
            buf = vhb.shape(char, {"variations": variations})
            return vhb.buf_to_bezierpaths(buf, penclass=BezPathCreatingPen)

        many_samples = 12

        parama = {
            "XOPQ": dict(glyph="H", start_point=(0.0, 0.5), direction="E", winding="ink", samples=many_samples),
            "XOLC": dict(glyph="n", start_point=(0.0, 0.4), direction="E", winding="ink", samples=many_samples),
            "XOFI": dict(glyph="1", start_point=(0.0, 0.5), direction="E", winding="ink"),
            "XTRA": dict(
                glyph="H", start_point=(0.0, 0.5), direction="E", winding="transparent", samples=many_samples
            ),
            "XTLC": dict(
                glyph="n", start_point=(0.0, 0.4), direction="E", winding="transparent", samples=many_samples
            ),
            "XTFI": dict(glyph="0", start_point=(0.0, 0.5), jittering=0.05, direction="E", winding="transparent"),
            "YOPQ": dict(
                glyph="H", start_point=(0.5, 0.25), direction="N", samples=int(many_samples * 0.8), winding="ink"
            ),
            "YOLC": dict(
                glyph="f",
                start_point=(0.75, 0.5),
                end_point=(0.75, 0.9),
                jittering=0.1,
                samples=many_samples // 2,
                winding="ink",
            ),
            "YOFI": dict(
                glyph="0", start_point=(0.5, 0), direction="N", samples=many_samples, jittering=0.05, winding="ink"
            ),
        }
        plots = len(parama) if args.draw == "*" else len([p for p in parama.keys() if p in (args.draw or "")])
        rows = max(math.ceil(plots / 3), 1)
        cols = max(min(plots, 3), 1)
        if plots > 0:
            fig, axes = plt.subplots(nrows=rows, ncols=cols)
        if plots == 0:
            axes = []
        elif plots == 1:
            axes = [axes]
        else:
            axes = axes.flat
        axes_iter = iter(axes)
        for parameter, settings in parama.items():
            # print("Calculating " + parameter)
            paths = paths_for_glyph(settings["glyph"])
            if not paths:
                print(f"Could not find glyph {settings['glyph']}")
                continue
            caster = Raycaster(
                paths,
                settings["start_point"],
                endpoint=settings.get("end_point"),
                direction=settings.get("direction"),
            )
            if "winding" in settings:
                caster = caster.winding(settings["winding"])
            caster = caster.jitter(settings.get("jittering", 0.2), settings.get("samples", 10))
            pairs = caster.pairs()
            try:
                value = round(pairs.median_pair_distance())
                # Upem scale
                value = round(value * 1000 / ttFont["head"].unitsPerEm)
            except statistics.StatisticsError:
                print(f"Could not determine {parameter}")
                value = None
            print(f"{parameter}: ", value)
            drawing = None
            if parameter in args.draw or args.draw == "*":
                drawing = pairs.draw(next(axes_iter), parameter, value)
        if plots > 0:
            plt.show()

    for font in args.fonts:
        print(font)
        find_parametric_values(font)
