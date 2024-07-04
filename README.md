# polars-readlines

[![PyPi Latest Release](https://img.shields.io/pypi/v/polars-readlines.svg)](https://pypi.org/project/polars-readlines)

A util for fast reading lines of files into Polars.
It uses memory mapping and SIMD.

Usage:

```python
import polars_readlines as plrl

df = plrl.read_lines("input.txt")
```
