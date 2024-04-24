# https://simoncozens.github.io/semiautomated-handwriting-fonts/

import pandas
import math
import matplotlib.pyplot as plt
import matplotlib.path as mpath
import matplotlib.patches as mpatches
import numpy as np
import seaborn as sns

from skimage.morphology import skeletonize
from skan import Skeleton
from beziers.path import BezierPath
from beziers.point import Point
from beziers.path.representations.Nodelist import Node
from PIL import ImageOps
from fontquant.helpers.images import svg_to_img

from fontquant import CustomTTFont


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


def point(point, ax):
    circle = plt.Circle((point.x, point.y), 2)
    ax.add_patch(circle)


def line(p1, p2, ax, color="gray", width=1):
    Path = mpath.Path
    pp = mpatches.PathPatch(Path([(p1.x, p1.y), (p2.x, p2.y)], [Path.MOVETO, Path.LINETO]), edgecolor=color, lw=width)
    ax.add_patch(pp)


def stroke_contrast(paths, width, ascender, descender, show=False):
    """\
    Calculate ratio between thickest and thinnest stroke of a BezierPath
    """

    # Settings
    margin = 50

    show_skeleton = True
    show_measurements = True

    # Remove overlaps
    for path in paths:
        path.removeOverlap()

    # Smooth
    # for path in paths:
    #     path.smooth(maxCollectionSize=1, lengthLimit=20)

    # Prepare matplotlib
    if show:
        f, (ax3, ax4, ax5) = plt.subplots(1, 3, figsize=(15, 15))

    # Adjust path position from bounds calculations
    x_shift = 0
    for path in paths:
        bounds = path.bounds()
        if bounds.bl.x < margin:
            x_shift = max(x_shift, margin - bounds.bl.x)
        if bounds.bl.y < descender + margin:
            descender = bounds.bl.y - margin
        if bounds.tr.y > ascender - margin:
            ascender = bounds.tr.y + margin
        if bounds.tr.x > width - margin:
            width = bounds.tr.x + margin
    width += x_shift
    for path in paths:
        path.translate(Point(x_shift, 0))
    height = ascender - descender
    for path in paths:
        path.translate(Point(0, -descender))

    via_image = True

    scale = 1.0
    if via_image:
        # Write SVG
        svg = f'<svg viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg"><path d="'
        for path in paths:
            svg += path.asSVGPath()
        svg += '" /></svg>'

        img = svg_to_img(svg)
        img = ImageOps.invert(img)

        # Scale down image
        scale = 2.5
        width, height = img.size
        img = img.resize((int(width / scale), int(height / scale)))

        # Skeletonize
        skeleton_lee = skeletonize(np.array(img), method="lee")
        sk = Skeleton(skeleton_lee)

    # Analyze
    strokes = pandas.DataFrame(columns=["center", "p1", "p2", "thickness", "position"])

    # for skeleton_path in sk.paths_list():
    if True:
        i = 0

        if via_image:
            skeleton_path = BezierPath.fromPoints([Point(x, y) for y, x, _ in sk.path_coordinates(i)], error=200)
        else:
            skeleton_path = paths[1]

        # Scale up again
        if via_image:
            skeleton_path.scale(scale)
        skeleton_path.tidy(maxCollectionSize=0, lengthLimit=50)
        # lower number here leads to more segments
        skeleton_path = skeleton_path.flatten(5)

        nodeList = skeleton_path.asNodelist()
        for i, node in enumerate(nodeList):
            if i == 0:
                j = len(nodeList) - 1
            else:
                j = i - 1
            previousNode = nodeList[j]

            if node.point == previousNode.point:
                continue

            # Find point halfway between node and previous node
            halfway_point = Interpolate(node.point, previousNode.point, 0.5)

            distance = node.point.distanceFrom(previousNode.point)

            if show and show_skeleton:
                point(halfway_point, ax3)

            # Rotate node 90° and variate angle to find the shortest distance at halfway_point
            angles = pandas.DataFrame(columns=["p1", "p2", "thickness"])

            for angle in range(-30, 31):
                outside_point = node.point.rotated(halfway_point, math.radians(90 + angle))

                # distance
                distance = outside_point.distanceFrom(halfway_point)

                if distance:
                    # Create line with endpoints far outside glyph
                    # TODO: Find better way to calculate the 5000 distance, based on font's UPM
                    far_outside_point_1 = Interpolate(halfway_point, outside_point, 5000 / distance)
                    far_outside_point_2 = Interpolate(halfway_point, outside_point, -1 * (5000 / distance))
                    measurement_line_path = BezierPath.fromNodelist(
                        [
                            Node(far_outside_point_1.x, far_outside_point_1.y, "line"),
                            Node(far_outside_point_2.x, far_outside_point_2.y, "line"),
                        ]
                    )
                    measurement_line_segments = measurement_line_path.asSegments()
                    # line(far_outside_point_1, far_outside_point_2, ax3)

                    # Iterate over glyph segments
                    measurement_line_segment = measurement_line_segments[0]
                    intersections_list = []
                    for glyph_path in paths:
                        glyph_segments = glyph_path.asSegments()
                        for glyph_segment in glyph_segments:
                            intersections = measurement_line_segment.intersections(glyph_segment)
                            intersections_list.extend(intersections)

                    # Found 2 or more intersections
                    #  and len(intersections_list) >= 2
                    if intersections_list:
                        # Sort points by distance to node and shorten list
                        if len(intersections_list) > 2:
                            intersections_list.sort(key=lambda x: x.point.distanceFrom(halfway_point))
                            intersections_list = intersections_list[:2]

                        if len(intersections_list) >= 2:
                            angles.loc[len(angles.index)] = [
                                intersections_list[0].point,
                                intersections_list[1].point,
                                intersections_list[0].point.distanceFrom(intersections_list[1].point),
                            ]

            # Process different angles measured at halfway_point
            remove_outliers(angles, "thickness")
            min_angle = angles.loc[angles["thickness"].idxmin()]

            # Add shortest line to stokes list
            strokes.loc[len(strokes.index)] = [
                halfway_point,
                min_angle["p1"],
                min_angle["p2"],
                min_angle["thickness"],
                i / len(nodeList),
            ]

            if show and show_measurements:
                point(min_angle["p1"], ax3)
                point(min_angle["p2"], ax3)
                line(min_angle["p1"], min_angle["p2"], ax3)

    # https://www.geeksforgeeks.org/detect-and-remove-the-outliers-using-python/

    if show:
        sns.boxplot(strokes["thickness"], ax=ax4)

    remove_outliers(strokes, "thickness")

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
        line(max_row["p1"], max_row["p2"], ax3, color="red", width=2)
        line(min_row["p1"], min_row["p2"], ax3, color="blue", width=2)
        if other_min is not None:
            line(other_min["p1"], other_min["p2"], ax3, color="blue", width=2)

        ax3.set_aspect("equal")
        ax3.set_ylim(0, height)
        ax3.set_xlim(0, width)

    # Contrast angle
    contrast_angle = None
    if other_min is not None:
        if other_min["center"].y > min_row["center"].y:
            vector = other_min["center"] - min_row["center"]
        else:
            vector = min_row["center"] - other_min["center"]
        contrast_angle = math.degrees(vector.angle) - 90
        if show:
            line(other_min["center"], min_row["center"], ax3, color="gray", width=2)

    if show:
        plt.show()

    return round((min_row["thickness"] / max_row["thickness"]) * 100) / 100, round(contrast_angle * 100) / 100


if __name__ == "__main__":
    import sys

    ttFont = CustomTTFont(sys.argv[-1])
    character = ttFont.glyphname_for_char("o")
    width = ttFont.getGlyphSet()[character].width
    paths = BezierPath.fromFonttoolsGlyph(ttFont, character)
    descender = ttFont["hhea"].descender
    ascender = ttFont["hhea"].ascender
    assert descender <= 0

    print(stroke_contrast(paths, width, ascender, descender, show=True))
