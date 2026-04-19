"""Espejo Python de list_references.rs."""
import sys

import gdstk


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: list_references.py <file.gds>", file=sys.stderr)
        sys.exit(2)

    lib = gdstk.read_gds(sys.argv[1])
    for cell in lib.cells:
        if not cell.references:
            continue
        print(f"Cell '{cell.name}':")
        for r in cell.references:
            # r.cell can be a Cell, RawCell, or str (ReferenceType::Name).
            if isinstance(r.cell, str):
                name = r.cell
            else:
                name = r.cell.name
            x, y = r.origin
            print(
                f"  -> '{name}' @ ({x:.2f}, {y:.2f}) "
                f"rot={r.rotation:.3f} mag={r.magnification:.3f} "
                f"xrefl={str(r.x_reflection).lower()}"
            )


if __name__ == "__main__":
    main()
