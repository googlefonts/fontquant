# Fontquant Documentation

## Casing

### SmallCaps (`casing/smcp`)

Returns the percentage of characters that are lowercase letters (`Ll`) and get shaped by the `smcp` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `smcp` feature if the value is above `0.95` (95%), as there are some characters that are lowercase letters but don't get shaped by the `smcp` feature, e.g. `florin`. Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.py` to see your values rise.


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["smcp"]["value"]
print(value)
>>> 0.5
```

### Caps-To-SmallCaps (`casing/c2sc`)

Returns the percentage of characters that are uppercase letters (`Lu`) and get shaped by the `c2sc` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `c2sc` feature if the value is above `0.95` (95%), as there are some characters that are uppercase letters but don't typically get shaped by the `c2sc` feature, e.g. `Ohm`. Alternatively, consider contributing exceptions to the `exceptions_c2sc` variable in `casing.py` to see your values rise.


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["c2sc"]["value"]
print(value)
>>> 0.5
```

### Case-Sensitive Punctuation (`casing/case`)

Returns the percentage of characters that are punctuation (`P*`) and get shaped by the `case` feature. 

_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["casing"]["case"]["value"]
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

### Superior Numerals (`numerals/superiors`)

Returns the percentage of numerals that get shaped by the `sups` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%). _A partial support is useless in practice._


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["superiors"]["value"]
print(value)
>>> 0.5
```

### Inferior Numerals (`numerals/inferiors`)

Returns the percentage of numerals that get shaped by the `sinf` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%). _A partial support is useless in practice._


_Return Value:_ Percentage expressed as float 0—1 (e.g. `0.5`)

_Example:_
```python
from fontquant import quantify
results = quantify("path/to/font.ttf")
value = results["numerals"]["inferiors"]["value"]
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

Returns percentage of feature combinations that shape the slashed zero. Here, the `zero` feature is used alone and in combination with other numeral-related features, if supported by the font, currently `subs`, `sinf`, `frac`. 

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

