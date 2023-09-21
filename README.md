# Fontquant

Fontquant looks into a font and quantifies what’s in it, creating a machine-readable representation of font features that it has _proven_ to work. It makes heavy use of the _Harfbuzz_ shaping engine to prove the functionality of font features, rather than just looking up the feature list in the font.

The purpose of Fontquant is to provide a high-level quantifiable overview of a font’s features and quality in order to make fonts’ technical quality comparable, to make their features searchable through a user interface as part of a font library, and for font quality assurance (QA).

Examples: 

* `smcp` and `c2sc` features are checked by applying the features to all encoded lowercase or uppercase characters in the font, using Python’s own `unicodetata` library to check for a character’s category to be `Ll` or `Lu` (lowercase letter or uppercase letter), and seeing whether the shaping changes after applying `smcp` or `c2sc`. The resulting value is a percentage expressed as a floating point number (0—1) that represents the total number of qualifying characters that got shaped successfully. A professional font should reach a value of `1.0` here (100%).
* Likewise, superior numbers (`¹`) are quantified individually. A value of `0.0` means no superior numbers are activatable through the `sups` feature. `0.4` means that the font contains only four superior numbers as required by a character set (often `¹²³⁴`), but the feature should not be advertized as fully supported until the value reaches `1.0` because a partial support is unusable in practice.

The individual values are described [in the documentation](Lib/fontquant/README.md) along with recommendations of how to interpret the values.

# Usage

Install tool with pip: `pip install .`, then invoke tool with: `fontquant font.ttf`.
Currently prints formatted JSON to the screen.
