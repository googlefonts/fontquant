from beziers.path.representations.fontparts import FontParts
from booleanOperations.booleanGlyph import BooleanGlyph
from fontParts.fontshell import RGlyph
from beziers.path import BezierPath


def removeOverlaps(paths):
    g = RGlyph()
    for path in paths:
        path = path.quadraticsToCubics()
        if path.length:
            FontParts.drawToFontpartsGlyph(g, path)
    bg = BooleanGlyph(g)
    bg = bg.removeOverlap()
    paths = BezierPath.fromDrawable(bg)
    return paths
