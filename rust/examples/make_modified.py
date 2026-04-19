"""Crea proof_lib_v2.gds agregando un rectángulo a una celda.

Útil para probar diff_gds: el output debería mostrar ese rectángulo como
área agregada en la celda y layer donde se insertó.
"""
import sys

import gdstk


def main() -> None:
    src = sys.argv[1]  # proof_lib.gds
    dst = sys.argv[2]  # proof_lib_v2.gds
    target_cell_name = sys.argv[3] if len(sys.argv) > 3 else None
    target_layer = int(sys.argv[4]) if len(sys.argv) > 4 else 0

    lib = gdstk.read_gds(src)

    # Elegir celda a modificar: la primera si no se especifica.
    target_cell = None
    if target_cell_name:
        for c in lib.cells:
            if c.name == target_cell_name:
                target_cell = c
                break
    else:
        target_cell = lib.cells[0]

    # Insertar un rectángulo nuevo de área conocida: 5 x 3 = 15 µm².
    rect = gdstk.rectangle((100.0, 100.0), (105.0, 103.0), layer=target_layer)
    target_cell.add(rect)

    lib.write_gds(dst)
    print(f"Modificado: cell '{target_cell.name}' layer {target_layer}")
    print(f"Rectángulo agregado 5x3 (área=15.00 µm²) en (100,100)-(105,103)")


if __name__ == "__main__":
    main()
