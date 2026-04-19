"""Imprime area total por capa, por celda. Espejo de list_polygons.rs."""
import sys
from collections import defaultdict

import gdstk


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: list_polygons.py <file.gds>", file=sys.stderr)
        sys.exit(2)

    lib = gdstk.read_gds(sys.argv[1])
    for cell in lib.cells:
        by_layer: dict[int, float] = defaultdict(float)
        for poly in cell.polygons:
            by_layer[poly.layer] += poly.area()
        print(f"Cell '{cell.name}':")
        for layer in sorted(by_layer):
            print(f"  Layer {layer}: {by_layer[layer]:.2f}")


if __name__ == "__main__":
    main()
