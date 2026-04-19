"""Crea tinytapeout_v2.gds moviendo un FlexPath para probar XOR con paths.

Usage: make_path_modified.py <src.gds> <dst.gds>

Mueve el primer FlexPath de la primera celda que tenga paths, en +1 um en x e y.
"""
import sys

import gdstk


def main() -> None:
    src = sys.argv[1]
    dst = sys.argv[2]

    lib = gdstk.read_gds(src)
    modified_cell_name = None
    for cell in lib.cells:
        flexpaths = [p for p in cell.paths if isinstance(p, gdstk.FlexPath)]
        if flexpaths:
            flexpaths[0].translate(1.0, 1.0)  # move 1 um in x and y
            modified_cell_name = cell.name
            break
    else:
        print("no cells with flexpaths found", file=sys.stderr)
        sys.exit(1)

    lib.write_gds(dst)
    print(f"Modified cell '{modified_cell_name}': translated flexpath[0] by (+1, +1) um")


if __name__ == "__main__":
    main()
