# https://simoncozens.github.io/semiautomated-handwriting-fonts/

import pandas
import math
import matplotlib.pyplot as plt
import matplotlib.path as mpath
import matplotlib.patches as mpatches
import numpy as np
import seaborn as sns

from kurbopy import Point, Vec2, Line, Vec2


# Distance of measurement points.
# Smaller values lead to more segments and more accurate results, but is slower.
PRECISION = 0.1


def remove_outliers(array, column):
    """Detection"""
    # IQR
    # Calculate the upper and lower limits
    Q1 = array[column].quantile(0.25)
    Q3 = array[column].quantile(0.75)
    IQR = Q3 - Q1
    lower = Q1 - 1.1 * IQR
    upper = Q3 + 1.1 * IQR

    # Create arrays of Boolean values indicating the outlier rows
    upper_array = np.where(array[column] >= upper)[0]
    lower_array = np.where(array[column] <= lower)[0]

    # Removing the outliers
    array.drop(index=upper_array, inplace=True)
    array.drop(index=lower_array, inplace=True)


def Interpolate(a, b, p, limit=False):
    """
    Interpolate between values a and b at float position p (0-1)
    Limit: No extrapolation
    """
    i = a + (b - a) * p
    if limit and i < a:
        return a
    elif limit and i > b:
        return b
    else:
        return i


def interpolate_point(a, b, p):
    return Point(Interpolate(a.x, b.x, p), Interpolate(a.y, b.y, p))


def draw_point(point, ax):
    circle = plt.Circle((point.x, point.y), 2)
    ax.add_patch(circle)


def line(p1, p2, ax, color="gray", width=1):
    Path = mpath.Path
    pp = mpatches.PathPatch(Path([(p1.x, p1.y), (p2.x, p2.y)], [Path.MOVETO, Path.LINETO]), edgecolor=color, lw=width)
    ax.add_patch(pp)


def rotate(p, origin=(0, 0), degrees=0):
    angle = np.deg2rad(degrees)
    R = np.array([[np.cos(angle), -np.sin(angle)], [np.sin(angle), np.cos(angle)]])
    o = np.atleast_2d(origin)
    p = np.atleast_2d(p)
    return np.squeeze((R @ (p.T - o.T) + o.T).T)


def intersections(paths, line):
    intersections = []

    for glyph_path in paths:
        intersections.extend(glyph_path.intersections(line))

    return intersections


def stroke_contrast(paths, width, ascender, descender, show=False):
    """\
    Calculate ratio between thickest and thinnest stroke of a BezierPath
    """

    # Remove overlaps
    # TODO: Find a way to reactiveate this
    # for path in paths:
    #     path.removeOverlap()

    # Prepare matplotlib
    if show:
        f, (ax3, ax4, ax5) = plt.subplots(1, 3, figsize=(15, 15))

    height = ascender - descender

    # Analyze
    strokes = pandas.DataFrame(columns=["center", "p1", "p2", "thickness", "position"])

    # for skeleton_path in sk.paths_list():
    if True:
        i = 0

        # Sort paths by size
        paths = sorted(paths, key=lambda x: x.bounding_box().width(), reverse=True)
        skeleton_path = paths[0]

        # Scale up again
        skeleton_path = skeleton_path.flatten(PRECISION)

        for i, point in enumerate(skeleton_path):
            if i == 0:
                j = len(skeleton_path) - 1
            else:
                j = i - 1
            previous_point = skeleton_path[j]

            if point.x == previous_point.x and point.y == previous_point.y:
                continue

            # Find point halfway between node and previous node
            halfway_point = point.midpoint(previous_point)

            distance = point.distance(previous_point)

            if show:
                draw_point(halfway_point, ax3)

            # Rotate node 90° and variate angle to find the shortest distance at halfway_point
            angles = pandas.DataFrame(columns=["p1", "p2", "thickness"])

            for angle in range(-30, 31, 3):

                outside_point = Point(*rotate((point.x, point.y), (halfway_point.x, halfway_point.y), 90 + angle))
                distance = outside_point.distance(halfway_point)

                if distance:
                    # Create line with endpoints far outside glyph
                    # TODO: Find better way to calculate the 5000 distance, based on font's UPM
                    far_outside_point_1 = interpolate_point(halfway_point, outside_point, 5000 / distance)
                    far_outside_point_2 = interpolate_point(halfway_point, outside_point, -1 * (5000 / distance))
                    measurement_line = Line(far_outside_point_1, far_outside_point_2)
                    intersections_list = intersections(paths, measurement_line)

                    # Found 2 or more intersections
                    if intersections_list:
                        # Sort points by distance to node and shorten list
                        if len(intersections_list) > 2:
                            intersections_list.sort(key=lambda x: x.distance(halfway_point))
                            intersections_list = intersections_list[:2]

                        angles.loc[len(angles.index)] = [
                            intersections_list[0],
                            intersections_list[1],
                            intersections_list[0].distance(intersections_list[1]),
                        ]

            # Process different angles measured at halfway_point
            remove_outliers(angles, "thickness")
            min_angle = angles.loc[angles["thickness"].idxmin()]

            # Add shortest line to strokes list
            strokes.loc[len(strokes.index)] = [
                halfway_point,
                min_angle["p1"],
                min_angle["p2"],
                min_angle["thickness"],
                i / len(skeleton_path),
            ]
            if show:
                draw_point(min_angle["p1"], ax3)
                draw_point(min_angle["p2"], ax3)
                line(min_angle["p1"], min_angle["p2"], ax3)

    # https://www.geeksforgeeks.org/detect-and-remove-the-outliers-using-python/

    if show:
        sns.boxplot(strokes["thickness"], ax=ax4)

    # This caused faults in some cases, removing the maximum value
    # remove_outliers(strokes, "thickness")

    if show:
        sns.boxplot(strokes["thickness"], ax=ax5)

    # max
    max_row = strokes.loc[strokes["thickness"].idxmax()]
    min_row = strokes.loc[strokes["thickness"].idxmin()]

    # find other minimum
    strokes.sort_values(by=["thickness"], inplace=True)
    other_min = None
    for ind in strokes.index:
        # TODO:
        # The 0.75/0.25 approach is a crutch. Make sure this point can be found going around the path's origin
        # like an unsigned integer
        if 0.75 > abs(strokes["position"][ind] - min_row["position"]) > 0.25:
            other_min = strokes.loc[ind]
            break

    if show:
        line(max_row["p1"], max_row["p2"], ax3, color="red", width=4)
        line(min_row["p1"], min_row["p2"], ax3, color="blue", width=4)
        if other_min is not None:
            line(other_min["p1"], other_min["p2"], ax3, color="blue", width=4)

        ax3.set_aspect("equal")
        ax3.set_ylim(0, height)
        ax3.set_xlim(0, width)

    # Contrast angle
    contrast_angle = None
    if other_min is not None:
        if other_min["center"].y > min_row["center"].y:
            vector = Vec2(min_row["center"].x - other_min["center"].x, min_row["center"].y - other_min["center"].y)
        else:
            vector = Vec2(other_min["center"].x - min_row["center"].x, other_min["center"].y - min_row["center"].y)
        contrast_angle = math.degrees(vector.atan2()) + 90
        if show:
            line(other_min["center"], min_row["center"], ax3, color="gray", width=2)

    if show:
        plt.show()

    return round((min_row["thickness"] / max_row["thickness"]) * 100) / 100, round(contrast_angle * 100) / 100


if __name__ == "__main__":

    from fontquant import CustomTTFont, CustomHarfbuzz
    import sys
    from kurbopy import BezPathCreatingPen

    ttFont = CustomTTFont(sys.argv[-1])
    vhb = CustomHarfbuzz(sys.argv[-1])
    character = ttFont.glyphname_for_char("o")

    buf = vhb.shape(character)
    pen = BezPathCreatingPen()
    for info in buf.glyph_infos:
        vhb._hbfont.draw_glyph_with_pen(info.codepoint, pen)
    paths = pen.paths

    width = ttFont.getGlyphSet()[character].width
    descender = ttFont["hhea"].descender
    ascender = ttFont["hhea"].ascender
    assert descender <= 0

    measurement_line = Line(Point(51, 411), Point(534, 454))

    print(stroke_contrast(paths, width, ascender, descender, show=True))
    # print(intersections(paths, measurement_line))
