//! Imprime metadata de una Library (Fase 6).

use std::env;

use gdstk_rs::Library;

fn main() {
    let path = env::args().nth(1).expect("usage: lib_info <file.gds>");
    let lib = Library::open(&path);
    println!("name: {}", lib.name());
    println!("unit: {}", lib.unit());
    println!("precision: {}", lib.precision());
    println!("cell_count: {}", lib.cell_count());
    let tl = lib.top_level();
    println!("top_level: {}", tl.count());
    for c in tl.cells() {
        println!("  {}", c.name());
    }
}
