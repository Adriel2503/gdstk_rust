//! Imprime área total por capa, por celda. Fase 2 test de paridad con Python.
//!
//! Usage: list_polygons <file.gds>

use std::collections::BTreeMap;
use std::env;

use gdstk_rs::Library;

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "list_polygons".into());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <file.gds>");
        std::process::exit(2);
    };

    let lib = Library::open(&path);

    for cell in lib.cells() {
        let mut by_layer: BTreeMap<u32, f64> = BTreeMap::new();
        for poly in cell.polygons() {
            *by_layer.entry(poly.layer()).or_insert(0.0) += poly.area();
        }
        println!("Cell '{}':", cell.name());
        for (layer, area) in &by_layer {
            println!("  Layer {layer}: {area:.2}");
        }
    }
}
