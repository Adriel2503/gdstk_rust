"""Imprime labels por celda. Espejo de list_labels.rs."""
import sys

import gdstk


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: list_labels.py <file.gds>", file=sys.stderr)
        sys.exit(2)

    lib = gdstk.read_gds(sys.argv[1])
    for cell in lib.cells:
        if not cell.labels:
            continue
        print(f"Cell '{cell.name}':")
        for lbl in cell.labels:
            x, y = lbl.origin
            print(
                f"  Layer {lbl.layer}/{lbl.texttype}  "
                f"({x:.2f}, {y:.2f})  '{lbl.text}'"
            )


if __name__ == "__main__":
    main()
