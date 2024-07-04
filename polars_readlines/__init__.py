import os
import polars as pl

from ._polars_readlines import read_lines as _read_lines

__all__ = ("read_lines",)
__version__ = "0.1.0"


def read_lines(path: os.PathLike) -> pl.DataFrame:
    return _read_lines(os.fsdecode(path))
