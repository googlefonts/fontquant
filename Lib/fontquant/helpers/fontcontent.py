siblings = (
    ("Kore", "Hang"),
    ("Jpan", "Hani", "Hant", "Hans"),
    ("Hira", "Kana"),
)


def is_sibling_script(target, guessed):
    for family in siblings:
        if guessed in family and target in family:
            return True


def get_sibling_scripts(target):
    for family in siblings:
        if target in family:
            return family


def get_primary_script(ttFont):
    from fontTools import unicodedata
    from collections import Counter

    script_count = Counter()
    for c in ttFont.getBestCmap().keys():
        for script in unicodedata.script_extension(chr(c)):
            if script not in ["Zinh", "Zyyy", "Zzzz"]:
                # Zinh: "Inherited"
                # Zyyy: "Common"
                # Zzzz: "Unknown"
                script_count[script] += 1
    most_common = script_count.most_common(1)
    if most_common:
        script = most_common[0][0]
        return script


def get_glyphs_for_script(ttFont, script):
    from fontTools import unicodedata

    cmap = ttFont.getBestCmap()
    glyphs = []
    for c in cmap.keys():
        # print(list(unicodedata.script_extension(chr(c)))[0])
        if script in unicodedata.script_extension(chr(c)):
            glyphs.append(cmap[c])

    return glyphs


# if __name__ == "__main__":
#     from fontquant import CustomTTFont

#     ttFont = CustomTTFont("tests/fonts/Farro-Regular.ttf")
#     print(get_primary_script(ttFont))
#     print(get_glyphs_for_script(ttFont, get_primary_script(ttFont)))
