//! Compara dos archivos GDS y reporta cambios geométricos por celda y layer.
//!
//! Usage: diff_gds <a.gds> <b.gds>
//!
//! - Para cada celda presente en ambos: corre XOR por layer y reporta área cambiada.
//! - Para celdas solo en A: reporta como ELIMINADA.
//! - Para celdas solo en B: reporta como AGREGADA.

use std::collections::BTreeSet;
use std::env;

use gdstk_rs::Library;

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "diff_gds".into());
    let (Some(a_path), Some(b_path)) = (args.next(), args.next()) else {
        eprintln!("usage: {program} <a.gds> <b.gds>");
        std::process::exit(2);
    };

    let lib_a = Library::open(&a_path);
    let lib_b = Library::open(&b_path);

    // Unión de layers presentes en A o B.
    let mut layers: BTreeSet<u32> = BTreeSet::new();
    for cell in lib_a.cells() {
        for p in cell.polygons() {
            layers.insert(p.layer());
        }
    }
    for cell in lib_b.cells() {
        for p in cell.polygons() {
            layers.insert(p.layer());
        }
    }

    // Celdas compartidas: diff por layer.
    for cell_a in lib_a.cells() {
        let name = cell_a.name();
        if let Some(cell_b) = lib_b.find_cell(name) {
            let mut header_printed = false;
            for &layer in &layers {
                let m = cell_a.xor_with(&cell_b, layer);
                if m.region_count > 0 {
                    if !header_printed {
                        println!("Cell '{name}':");
                        header_printed = true;
                    }
                    println!(
                        "  Layer {}: {:.2} µm² en {} región(es)",
                        layer, m.area, m.region_count
                    );
                }
            }
        } else {
            println!("Cell '{name}': ELIMINADA (solo en A)");
        }
    }

    // Celdas solo en B.
    for cell_b in lib_b.cells() {
        if lib_a.find_cell(cell_b.name()).is_none() {
            println!("Cell '{}': AGREGADA (solo en B)", cell_b.name());
        }
    }
}
