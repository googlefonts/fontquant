# Fontquant Documentation



* Casing:  
  * [SmallCaps](#smallcaps-casingsmallcaps)  
  * [Caps-To-SmallCaps](#caps-to-smallcaps-casingcaps-to-smallcaps)  
  * [Case-Sensitive Punctuation](#case-sensitive-punctuation-casingcase_sensitive_punctuation)  
  * [Unicase](#unicase-casingunicase)  
  * [Lowercase Shapes](#lowercase-shapes-casinglowercase_shapes)  
* Numerals:  
  * [Proportional Oldstyle Numerals](#proportional-oldstyle-numerals-numeralsproportional_oldstyle)  
  * [Tabular Oldstyle Numerals](#tabular-oldstyle-numerals-numeralstabular_oldstyle)  
  * [Proportional Lining Numerals](#proportional-lining-numerals-numeralsproportional_lining)  
  * [Tabular Lining Numerals](#tabular-lining-numerals-numeralstabular_lining)  
  * [Default Numerals](#default-numerals-numeralsdefault_numerals)  
  * [Superior Numerals](#superior-numerals-numeralssuperior_numerals)  
  * [Inferior Numerals](#inferior-numerals-numeralsinferior_numerals)  
  * [Encoded Fractions](#encoded-fractions-numeralsencoded_fractions)  
  * [Arbitrary Fractions](#arbitrary-fractions-numeralsarbitrary_fractions)  
  * [Slashed Zero](#slashed-zero-numeralsslashed_zero)  
* Appearance:  
  * [Stroke Contrast Ratio 🎛️](#stroke-contrast-ratio-appearancestroke_contrast_ratio)  
  * [Stroke Contrast Angle 🎛️](#stroke-contrast-angle-appearancestroke_contrast_angle)  
  * [Weight 🎛️](#weight-appearanceweight)  
  * [Width 🎛️](#width-appearancewidth)  
  * [Slant 🎛️](#slant-appearanceslant)  
  * [Lowercase a style](#lowercase-a-style-appearancelowercase_a_style)  
  * [Lowercase g style](#lowercase-g-style-appearancelowercase_g_style)  
  * [Stencil](#stencil-appearancestencil)  
  * [x-Height 🎛️](#x-height-appearancex_height)  
  * [Cap-Height 🎛️](#cap-height-appearancecap_height)  
  * [Ascender 🎛️](#ascender-appearanceascender)  
  * [Descender 🎛️](#descender-appearancedescender)  
  * [XOPQ 🎛️](#xopq-appearanceXOPQ)  
  * [XOLC 🎛️](#xolc-appearanceXOLC)  
  * [XOFI 🎛️](#xofi-appearanceXOFI)  
  * [XTRA 🎛️](#xtra-appearanceXTRA)  
  * [XTLC 🎛️](#xtlc-appearanceXTLC)  
  * [XTFI 🎛️](#xtfi-appearanceXTFI)  
  * [YOPQ 🎛️](#yopq-appearanceYOPQ)  
  * [YOLC 🎛️](#yolc-appearanceYOLC)  
  * [YOFI 🎛️](#yofi-appearanceYOFI)  
* Features:  
  * [List of OpenType Features](#list-of-opentype-features-featuresfeature_list)  
  * [Stylistic Set Names](#stylistic-set-names-featuresstylistic_sets)

## Casing

### SmallCaps (`casing/smallcaps`)



Returns the percentage of characters that are lowercase letters (`Ll`) and get shaped by the `smcp` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `smcp` feature if the value is above `0.95` (95%), as there are some characters that are lowercase letters but don't get shaped by the `smcp` feature, e.g. `florin`. Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.py` to see your values rise.


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["smallcaps"]["value"]
print(value)
>>> 0.5
```

### Caps-To-SmallCaps (`casing/caps-to-smallcaps`)



Returns the percentage of characters that are uppercase letters (`Lu`) and get shaped by the `c2sc` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `c2sc` feature if the value is above `0.95` (95%), as there are some characters that are uppercase letters but don't typically get shaped by the `c2sc` feature, e.g. `Ohm`. Alternatively, consider contributing exceptions to the `exceptions_c2sc` variable in `casing.py` to see your values rise.


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["caps-to-smallcaps"]["value"]
print(value)
>>> 0.5
```

### Case-Sensitive Punctuation (`casing/case_sensitive_punctuation`)



Returns the percentage of characters that are punctuation (`P*`) and get shaped by the `case` feature. 

_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["case_sensitive_punctuation"]["value"]
print(value)
>>> 0.5
```

### Unicase (`casing/unicase`)



Reports whether or not a font is unicase (lowercase and uppercase letters being of the same height). To check for different shapes of lowercase letters compared to uppercase, use the `Lowercase Shapes` metric. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["unicase"]["value"]
print(value)
>>> True
```

### Lowercase Shapes (`casing/lowercase_shapes`)



Returns the shapes of lowercase-codepoint characters. Possible values are `uppercase`, `lowercase`, and `smallcaps`. This check compares the contour count (and the average height) of uppercase and lowercase letters, so it compares actual outline construction. In that sense it's different from the `Unicase` metric which only looks at dimensions and allows upper/lowercase shapes to be different as long as they are of similar height. 

_Return Value:_ String

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["lowercase_shapes"]["value"]
print(value)
>>> abc...
```

## Numerals

### Proportional Oldstyle Numerals (`numerals/proportional_oldstyle`)



Returns a boolean of whether or not the font has functioning set of _proportional oldstyle_ numerals, either by default or activatable by the `onum`/`pnum` features. This check also performs heuristics to see whether the activated numeral set matches the common expectations on width/height variance and returns `False` if it doesn't. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["proportional_oldstyle"]["value"]
print(value)
>>> True
```

### Tabular Oldstyle Numerals (`numerals/tabular_oldstyle`)



Returns a boolean of whether or not the font has functioning set of _tabular oldstyle_ numerals, either by default or activatable by the `onum`/`tnum` features. This check also performs heuristics to see whether the activated numeral set matches the common expectations on width/height variance and returns `False` if it doesn't. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["tabular_oldstyle"]["value"]
print(value)
>>> True
```

### Proportional Lining Numerals (`numerals/proportional_lining`)



Returns a boolean of whether or not the font has functioning set of _proportional lining_ numerals, either by default or activatable by the `lnum`/`pnum` features. This check also performs heuristics to see whether the activated numeral set matches the common expectations on width/height variance and returns `False` if it doesn't. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["proportional_lining"]["value"]
print(value)
>>> True
```

### Tabular Lining Numerals (`numerals/tabular_lining`)



Returns a boolean of whether or not the font has functioning set of _tabular lining_ numerals, either by default or activatable by the `lnum`/`tnum` features. This check also performs heuristics to see whether the activated numeral set matches the common expectations on width/height variance and returns `False` if it doesn't. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["tabular_lining"]["value"]
print(value)
>>> True
```

### Default Numerals (`numerals/default_numerals`)



Returns the default numeral set (out of `proportional_oldstyle`, `tabular_oldstyle`, `proportional_lining`, `tabular_lining`). 

_Return Value:_ String

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["default_numerals"]["value"]
print(value)
>>> proportional_lining
```

### Superior Numerals (`numerals/superior_numerals`)



Returns the percentage of numerals that get shaped by the `sups` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%). _A partial support is useless in practice._


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["superior_numerals"]["value"]
print(value)
>>> 0.5
```

### Inferior Numerals (`numerals/inferior_numerals`)



Returns the percentage of numerals that get shaped by the `sinf` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%). _A partial support is useless in practice._


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["inferior_numerals"]["value"]
print(value)
>>> 0.5
```

### Encoded Fractions (`numerals/encoded_fractions`)



Returns percentage of encoded default fractions (e.g. ½) that are shaped by the `frac` feature. 

_Interpretation Hint:_ Consider encoded fractions to be _inferior_ to arbitrary fractions as checked by the `numerals/arbitrary_fractions` check. For a professional font, ignore this check.


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["encoded_fractions"]["value"]
print(value)
>>> 0.5
```

### Arbitrary Fractions (`numerals/arbitrary_fractions`)



Returns boolean of whether or not arbitrary fractions (e.g. 12/99) can be shaped by the `frac` feature. 

_Interpretation Hint:_ Consider arbitrary fractions to be _superior_ to encoded fractions as checked by the `numerals/encoded_fractions` check.


_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["arbitrary_fractions"]["value"]
print(value)
>>> True
```

### Slashed Zero (`numerals/slashed_zero`)



Returns percentage of feature combinations that shape the slashed zero. Here, the `zero` feature is used alone and in combination with other numeral-related features, if supported by the font, currently `sups`, `sinf`, `frac`. If so, the additional features are listed in the `checked_additional_features` key. 

_Interpretation Hint:_ A professional font should reach a value of `1.0` here.


_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["slashed_zero"]["value"]
print(value)
>>> 0.5
```

## Appearance

### Stroke Contrast Ratio (`appearance/stroke_contrast_ratio`)

🎛️ _This metric is variable-aware_

Calculates the ratio of the stroke contrast, calculated in thinnest/thickest stroke.  One representative character is measured for the font's primary script, such as the "o" for Latin.  Note that the two stroke contrast metrics (ratio and angle) are calculated in the same function. For efficiency, query both metrics at during the same call. 

_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["stroke_contrast_ratio"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 0.5, "wdth=100.0,wght=500.0": 0.5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stroke_contrast_ratio"]["value"]
print(value)
>>> 0.5
```

### Stroke Contrast Angle (`appearance/stroke_contrast_angle`)

🎛️ _This metric is variable-aware_

Calculates the angle of the stroke contrast. An angle of 0° means vertical contrast, with positive angles being counter-clockwise.  One representative character is measured for the font's primary script, such as the "o" for Latin.  Note that the two stroke contrast metrics (ratio and angle) are calculated in the same function. For efficiency, query both metrics at during the same call. 

_Return Value:_ Angle as floating point number (e.g. `-12.5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["stroke_contrast_angle"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": -12.5, "wdth=100.0,wght=500.0": -12.5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stroke_contrast_angle"]["value"]
print(value)
>>> -12.5
```

### Weight (`appearance/weight`)

🎛️ _This metric is variable-aware_

Measures the weight of encoded characters of the font as the amount of ink per glyph as a percentage of an em square and returns the average of all glyphs measured. Based on `fontTools.pens.statisticsPen.StatisticsPen` 

_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["weight"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 0.5, "wdth=100.0,wght=500.0": 0.5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["weight"]["value"]
print(value)
>>> 0.5
```

### Width (`appearance/width`)

🎛️ _This metric is variable-aware_

Measures the width of encoded characters of the font as a percentage of the UPM and returns the average of all glyphs measured. Based on `fontTools.pens.statisticsPen.StatisticsPen` 

_Return Value:_ Percentage as floating point number 0—1 (e.g. `0.5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["width"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 0.5, "wdth=100.0,wght=500.0": 0.5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["width"]["value"]
print(value)
>>> 0.5
```

### Slant (`appearance/slant`)

🎛️ _This metric is variable-aware_

Measures the slante angle of encoded characters of the font in degrees and returns the average of all glyphs measured. Right-leaning shapes have negative numbers. Based on `fontTools.pens.statisticsPen.StatisticsPen` 

_Return Value:_ Angle as floating point number (e.g. `-12.5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["slant"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": -12.5, "wdth=100.0,wght=500.0": -12.5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["slant"]["value"]
print(value)
>>> -12.5
```

### Lowercase a style (`appearance/lowercase_a_style`)



Attempts to determine the style of the lowercase "a" to be single or double story.  Only the most sturdy routines are used here. If the results are not conclusive, the metric will return None and you need to determine it manually.  The error margin for recognizing the single story "a" is 0-2%, and for the double story "a" 4-7%. 

_Return Value:_ String

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["lowercase_a_style"]["value"]
print(value)
>>> single_story
```

### Lowercase g style (`appearance/lowercase_g_style`)



Attempts to determine the style of the lowercase "g" to be single or double story.  Only the most sturdy routines are used here. If the results are not conclusive, the metric will return None and you need to determine it manually.  This metric is based on contour counting, which is not very reliable. A "g" which generally looks like a double story "g" but has an open lower bowl will be counted as single story, and a "g" in cursive writing that looks like a single story "g" but has a closed loop as part of an upstroke will be counted as double story. 

_Return Value:_ String

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["lowercase_g_style"]["value"]
print(value)
>>> single_story
```

### Stencil (`appearance/stencil`)



Reports whether or not a font is a stencil font.  It recognizes a stencil font correctly, but may sometimes mis-report non-stencil fonts as stencil fonts because it only looks at a limited set of characters for speed optimization. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stencil"]["value"]
print(value)
>>> True
```

### x-Height (`appearance/x_height`)

🎛️ _This metric is variable-aware_

Reports x-height of `x`. 

_Return Value:_ Per-Mille of UPM (e.g. `1000`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["x_height"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 1000, "wdth=100.0,wght=500.0": 1000}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["x_height"]["value"]
print(value)
>>> 1000
```

### Cap-Height (`appearance/cap_height`)

🎛️ _This metric is variable-aware_

Reports cap-height of `H`. 

_Return Value:_ Per-Mille of UPM (e.g. `1000`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["cap_height"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 1000, "wdth=100.0,wght=500.0": 1000}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["cap_height"]["value"]
print(value)
>>> 1000
```

### Ascender (`appearance/ascender`)

🎛️ _This metric is variable-aware_

Reports ascender of `h`. 

_Return Value:_ Per-Mille of UPM (e.g. `1000`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["ascender"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 1000, "wdth=100.0,wght=500.0": 1000}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["ascender"]["value"]
print(value)
>>> 1000
```

### Descender (`appearance/descender`)

🎛️ _This metric is variable-aware_

Reports descender of `y`. 

_Return Value:_ Per-Mille of UPM (e.g. `1000`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["descender"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 1000, "wdth=100.0,wght=500.0": 1000}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["descender"]["value"]
print(value)
>>> 1000
```

### XOPQ (`appearance/XOPQ`)

🎛️ _This metric is variable-aware_

Reports parametric axis XOPQ. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XOPQ"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XOPQ"]["value"]
print(value)
>>> 5
```

### XOLC (`appearance/XOLC`)

🎛️ _This metric is variable-aware_

Reports parametric axis XOLC. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XOLC"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XOLC"]["value"]
print(value)
>>> 5
```

### XOFI (`appearance/XOFI`)

🎛️ _This metric is variable-aware_

Reports parametric axis XOFI. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XOFI"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XOFI"]["value"]
print(value)
>>> 5
```

### XTRA (`appearance/XTRA`)

🎛️ _This metric is variable-aware_

Reports parametric axis XTRA. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XTRA"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XTRA"]["value"]
print(value)
>>> 5
```

### XTLC (`appearance/XTLC`)

🎛️ _This metric is variable-aware_

Reports parametric axis XTLC. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XTLC"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XTLC"]["value"]
print(value)
>>> 5
```

### XTFI (`appearance/XTFI`)

🎛️ _This metric is variable-aware_

Reports parametric axis XTFI. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["XTFI"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["XTFI"]["value"]
print(value)
>>> 5
```

### YOPQ (`appearance/YOPQ`)

🎛️ _This metric is variable-aware_

Reports parametric axis YOPQ. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["YOPQ"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["YOPQ"]["value"]
print(value)
>>> 5
```

### YOLC (`appearance/YOLC`)

🎛️ _This metric is variable-aware_

Reports parametric axis YOLC. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["YOLC"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["YOLC"]["value"]
print(value)
>>> 5
```

### YOFI (`appearance/YOFI`)

🎛️ _This metric is variable-aware_

Reports parametric axis YOFI. 

_Return Value:_ Integer number (e.g. `5`)

_Example with **variable locations**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf", locations="wght=400,wdth=100;wght=500,wdth=100")
value = results["appearance"]["YOFI"]["value"]
print(value)
>>> {"wdth=100.0,wght=400.0": 5, "wdth=100.0,wght=500.0": 5}
```

**Note:** The axes per instance used in the _return value keys_ will be **sorted alphabetically**
and the _return values_ will be **float** _regardless of your input_.
To identify them in your results, you should also sort and format your input instances accordingly.
You may use `fontquant.helpers.var.sort_instance()` (per instance) or `.sort_instances()` (whole list at once)
for this purpose.

_Example with **origin location**:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["YOFI"]["value"]
print(value)
>>> 5
```

## Features

### List of OpenType Features (`features/feature_list`)



Returns a list of all registed OpenType features in the font from both GSUB and GPOS tables. 

_Return Value:_ List of values (e.g. `[0, 1, 2]`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["features"]["feature_list"]["value"]
print(value)
>>> ['aalt', 'liga', 'kern']
```

### Stylistic Set Names (`features/stylistic_sets`)



Returns a dictionary with the names of the stylistic sets in the font. 

_Return Value:_ Dictionary of values (e.g. `{"key": 0, "key2": 1, "key3": 2}`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["features"]["stylistic_sets"]["value"]
print(value)
>>> {'ss01': 'Alternate a shape'}
```

