"""Espejo Python de list_paths.rs (Fase 5.5, formato extendido)."""
import sys

import gdstk


def first_or_scalar(value, idx, default=None):
    """gdstk a veces expone un tuple (per-element), a veces un scalar."""
    if isinstance(value, (list, tuple)):
        if idx < len(value):
            return value[idx]
        return default
    return value


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: list_paths.py <file.gds>", file=sys.stderr)
        sys.exit(2)

    lib = gdstk.read_gds(sys.argv[1])
    for cell in lib.cells:
        flexpaths = [p for p in cell.paths if isinstance(p, gdstk.FlexPath)]
        robustpaths = [p for p in cell.paths if isinstance(p, gdstk.RobustPath)]
        if not flexpaths and not robustpaths:
            continue
        print(f"Cell '{cell.name}': {len(flexpaths)} flexpath, "
              f"{len(robustpaths)} robustpath")

        for i, fp in enumerate(flexpaths):
            simple = str(fp.simple_path).lower()
            scale = str(fp.scale_width).lower()
            spine_pts = len(fp.spine())
            print(f"  flexpath[{i}]: simple={simple} scale={scale} "
                  f"spine_pts={spine_pts}")
            for e in range(fp.num_paths):
                layer = fp.layers[e]
                dt = fp.datatypes[e]
                end = first_or_scalar(fp.ends, e, "flush")
                join = first_or_scalar(fp.joins, e, "natural")
                # gdstk Python doesn't expose bend_type name directly.
                # Derive: function > circular > none.
                bend_func = getattr(fp, "bend_function", None)
                bend_rad = first_or_scalar(fp.bend_radius, e, 0.0) or 0.0
                if callable(bend_func):
                    bend = "function"
                elif bend_rad > 0:
                    bend = "circular"
                else:
                    bend = "none"
                # widths(): numpy array of shape (num_paths, spine_pts)
                widths = fp.widths()
                hw0 = 0.0
                try:
                    hw0 = float(widths[e, 0]) / 2.0
                except (IndexError, TypeError):
                    pass
                print(f"    element[{e}]: layer {layer}/{dt} "
                      f"end={end} join={join} bend={bend} hw0={hw0:.4f}")

        for i, rp in enumerate(robustpaths):
            simple = str(rp.simple_path).lower()
            scale = str(rp.scale_width).lower()
            subpaths = rp.size
            ex, ey = rp.end_point
            tol = rp.tolerance
            print(f"  robustpath[{i}]: simple={simple} scale={scale} "
                  f"subpaths={subpaths} end=({ex:.2f},{ey:.2f}) tol={tol:.4f}")
            for e in range(rp.num_paths):
                layer = rp.layers[e]
                dt = rp.datatypes[e]
                end = first_or_scalar(rp.ends, e, "flush")
                # For RobustPath the "end width/offset" are the last values
                # used; gdstk Python exposes width(u) / offset(u) functions
                # evaluated at the final u.
                final_u = float(rp.size)
                ew_arr = rp.widths(final_u) if rp.size > 0 else None
                eo_arr = rp.offsets(final_u) if rp.size > 0 else None
                ew = ew_arr[e] if ew_arr and e < len(ew_arr) else 0.0
                eo = eo_arr[e] if eo_arr and e < len(eo_arr) else 0.0
                print(f"    element[{e}]: layer {layer}/{dt} end={end} "
                      f"ew={ew:.4f} eo={eo:.4f}")


if __name__ == "__main__":
    main()
