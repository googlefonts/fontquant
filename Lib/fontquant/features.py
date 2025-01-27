from fontquant import Metric, List, Dictionary


class Features(Metric):
    """\
    Returns a list of all registed OpenType features in the font from both GSUB and GPOS tables.
    """

    name = "List of OpenType Features"
    keyword = "feature_list"
    data_type = List
    example_value = ["aalt", "liga", "kern"]

    def value(self, includes=None, excludes=None):

        features = sorted(
            list(
                set(
                    [fr.FeatureTag for fr in self.ttFont["GSUB"].table.FeatureList.FeatureRecord]
                    + [fr.FeatureTag for fr in self.ttFont["GPOS"].table.FeatureList.FeatureRecord]
                )
            )
        )

        return {"value": features}


class StylisticSets(Metric):
    """\
    Returns a dictionary with the names of the stylistic sets in the font.
    """

    name = "Stylistic Set Names"
    keyword = "stylistic_sets"
    data_type = Dictionary
    example_value = {"ss01": "Alternate a shape"}

    def value(self, includes=None, excludes=None):

        def get_name_id(name_id):
            for name in self.ttFont["name"].names:
                if name.nameID == name_id:
                    return name.toUnicode()

        # read stylistic set names from ttFont
        features = {}
        for fr in self.ttFont["GSUB"].table.FeatureList.FeatureRecord:
            if fr.FeatureTag.startswith("ss") and hasattr(fr.Feature.FeatureParams, "UINameID"):
                features[fr.FeatureTag] = get_name_id(fr.Feature.FeatureParams.UINameID)

        return {"value": features}


class Features(Metric):
    name = "Features"
    keyword = "features"
    children = [Features, StylisticSets]
