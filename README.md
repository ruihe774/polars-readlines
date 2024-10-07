# polars-readlines

[![PyPi Latest Release](https://img.shields.io/pypi/v/polars-readlines)](https://pypi.org/project/polars-readlines)
[![Crates.io Version](https://img.shields.io/crates/v/polars-readlines)](https://crates.io/crates/polars-readlines)

A util for fast reading lines of files into Polars.
It uses memory mapping and SIMD.

Usage:

```python
import polars_readlines as plrl

df = plrl.read_lines("input.txt")
```
