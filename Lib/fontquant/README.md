# Fontquant Documentation

## Casing

### SmallCaps (`casing/smcp`)

Returns the amount of characters that are lowercase letters (`Ll`) and get shaped by the `smcp` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `smcp` feature if the value is above 0.95 (95%), as there are some characters that are lowercase letters but don't get shaped by the `smcp` feature, e.g. `florin`. Alternatively, consider contributing exceptions to the `exceptions_smcp` variable in `casing.py` to see your values rise.

### Caps-To-SmallCaps (`casing/c2sc`)

Returns the amount of characters that are uppercase letters (`Lu`) and get shaped by the `c2sc` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `c2sc` feature if the value is above 0.95 (95%), as there are some characters that are uppercase letters but don't typically get shaped by the `c2sc` feature, e.g. `Ohm`. Alternatively, consider contributing exceptions to the `exceptions_c2sc` variable in `casing.py` to see your values rise.

### Case-Sensitive Punctuation (`casing/case`)

Returns the amount of characters that are punctuation (`P*`) and get shaped by the `case` feature. 
## Numerals

### Proportional Oldstyle Numerals (`numerals/proportional_oldstyle`)

Returns a boolean of whether or not the font has functioning set of _proportional oldstyle_ numerals, either by default or activatable by the `onum`/`pnum` features. 
### Tabular Oldstyle Numerals (`numerals/tabular_oldstyle`)

Returns a boolean of whether or not the font has functioning set of _tabular oldstyle_ numerals, either by default or activatable by the `onum`/`tnum` features. 
### Proportional Lining Numerals (`numerals/proportional_lining`)

Returns a boolean of whether or not the font has functioning set of _proportional lining_ numerals, either by default or activatable by the `lnum`/`pnum` features. 
### Tabular Lining Numerals (`numerals/tabular_lining`)

Returns a boolean of whether or not the font has functioning set of _tabular lining_ numerals, either by default or activatable by the `lnum`/`tnum` features. 
### Default Numerals (`numerals/default_numerals`)

Returns the default numeral set (out of proportional_oldstyle, tabular_oldstyle, proportional_lining, tabular_lining). 
### Superior Numerals (`numerals/superiors`)

Returns the amount of numerals that get shaped by the `sups` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sups` feature if the value is 1.0 (100%). A partial support is useless in practice.

### Inferior Numerals (`numerals/inferiors`)

Returns the amount of numerals that get shaped by the `sinf` feature. 

_Interpretation Hint:_ Consider fonts to have a functioning `sinf` feature if the value is 1.0 (100%). A partial support is useless in practice.

### Encoded Fractions (`numerals/encoded_fractions`)

Returns percentage of encoded default fractions (e.g. ½) that are shaped by the `frac` feature. 

_Interpretation Hint:_ Consider encoded fractions to be inferior to arbitrary fractions as checked by the `numerals/arbitrary_fractions` check.

### Arbitrary Fractions (`numerals/arbitrary_fractions`)

Returns boolean of whether or not arbitrary fractions (e.g. 12/99) can be shaped by the `frac` feature. 

_Interpretation Hint:_ Consider arbitrary fractions to be superior to encoded fractions as checked by the `numerals/encoded_fractions` check.

### Slashed Zero (`numerals/slashed_zero`)

Returns percentage of feature combinations that shape the slashed zero. Here, the `zero` feature is used alone and in combination with other numeral-related features, currently `subs` and `sinf`. 
