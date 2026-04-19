"""Espejo Python de lib_info.rs."""
import sys

import gdstk


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: lib_info.py <file.gds>", file=sys.stderr)
        sys.exit(2)

    lib = gdstk.read_gds(sys.argv[1])
    print(f"name: {lib.name}")
    # gdstk Python representa el unit como float
    print(f"unit: {lib.unit}")
    print(f"precision: {lib.precision}")
    print(f"cell_count: {len(lib.cells)}")
    tops = lib.top_level()
    print(f"top_level: {len(tops)}")
    for c in tops:
        print(f"  {c.name}")


if __name__ == "__main__":
    main()
