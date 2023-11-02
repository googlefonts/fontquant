# Fontquant Documentation



* Casing:  
  * [SmallCaps](#smallcaps-casingsmallcaps)  
  * [Caps-To-SmallCaps](#caps-to-smallcaps-casingcaps-to-smallcaps)  
  * [Case-Sensitive Punctuation](#case-sensitive-punctuation-casingcase_sensitive_punctuation)  
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
  * [Stroke Contrast Ratio](#stroke-contrast-ratio-appearancestroke_contrast_ratio)  
  * [Stroke Contrast Angle](#stroke-contrast-angle-appearancestroke_contrast_angle)  
  * [Weight](#weight-appearanceweight)  
  * [Width](#width-appearancewidth)  
  * [Lowercase a style](#lowercase-a-style-appearancelowercase_a_style)  
  * [Lowercase g style](#lowercase-g-style-appearancelowercase_g_style)  
  * [Stencil](#stencil-appearancestencil)

## Casing

### SmallCaps (`casing/smallcaps`)

Returns the percentage of characters that are lowercase letters (`Ll`) and get shaped by the `smcp` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `smcp` feature if the value is above `0.95` (95%), as there are some characters that are lowercase letters but don't get shaped by the `smcp` feature, e.g. `florin`. Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.py` to see your values rise.


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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

_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["case_sensitive_punctuation"]["value"]
print(value)
>>> 0.5
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


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

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

Calculates the ratio of the stroke contrast, calculated in thinnest/thickest stroke.  One representative character is measured for the font's primary script, such as the "o" for Latin. 

_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stroke_contrast_ratio"]["value"]
print(value)
>>> 0.5
```

### Stroke Contrast Angle (`appearance/stroke_contrast_angle`)

Calculates the angle of the stroke contrast. An angle of 0° means vertical contrast, with positive angles being counter-clockwise.  One representative character is measured for the font's primary script, such as the "o" for Latin. 

_Return Value:_ Integer number (e.g. `5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stroke_contrast_angle"]["value"]
print(value)
>>> 5
```

### Weight (`appearance/weight`)

Measures the weight of all letters in the primary script of the font. This metric measures the amount of ink per glyph as a percentage of an em square and returns the average of all glyphs measured.  Based on fontTools.pens.statisticsPen.StatisticsPen 

_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["weight"]["value"]
print(value)
>>> 0.5
```

### Width (`appearance/width`)

Measures the width of all letters in the primary script of the font. This metric measures the average width of all glyphs as a percentage of the UPM.  Based on fontTools.pens.statisticsPen.StatisticsPen 

_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["width"]["value"]
print(value)
>>> 0.5
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

Reports whether or not a font is a stencil font. 

_Return Value:_ Boolean (`True`or `False`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["appearance"]["stencil"]["value"]
print(value)
>>> True
```

