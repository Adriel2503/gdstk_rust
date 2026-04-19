//! Roundtrip test: open A, write B, open B, diff A vs B. Debe ser 0 cambios.

use std::collections::BTreeSet;
use std::env;

use gdstk_rs::Library;

fn main() {
    let src = env::args().nth(1).expect("usage: roundtrip <src.gds> <dst.gds>");
    let dst = env::args().nth(2).expect("usage: roundtrip <src.gds> <dst.gds>");

    let lib_a = Library::open(&src);
    lib_a.write_gds(&dst).expect("write_gds failed");
    let lib_b = Library::open(&dst);

    // Comparar celdas por nombre y layer.
    let mut layers: BTreeSet<u32> = BTreeSet::new();
    for cell in lib_a.cells().chain(lib_b.cells()) {
        for p in cell.polygons() {
            layers.insert(p.layer());
        }
    }

    let mut diffs = 0u64;
    let mut total_area = 0.0f64;
    for cell_a in lib_a.cells() {
        if let Some(cell_b) = lib_b.find_cell(cell_a.name()) {
            for &layer in &layers {
                let m = cell_a.xor_with(&cell_b, layer);
                if m.region_count > 0 {
                    diffs += m.region_count;
                    total_area += m.area;
                    println!(
                        "  '{}' layer {}: {} regiones, {:.4} µm²",
                        cell_a.name(),
                        layer,
                        m.region_count,
                        m.area
                    );
                }
            }
        } else {
            println!("cell '{}' AGREGADA", cell_a.name());
            diffs += 1;
        }
    }

    if diffs == 0 {
        println!("ROUNDTRIP ✓ — 0 cambios detectados");
    } else {
        println!("ROUNDTRIP ✗ — {diffs} regiones, {:.4} µm² total", total_area);
        std::process::exit(1);
    }
}
