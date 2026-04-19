//! Variante de count_many que SOLO ejecuta la pasada secuencial
//! (una lectura por archivo), para comparación justa con Python.
//!
//! Para ver el speedup paralelo, usar count_many.rs.

use std::env;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: count_many_fair <file1.gds> ...");
        std::process::exit(2);
    }
    let total: u64 = files
        .iter()
        .map(|p| gdstk_rs::cell_count(&gdstk_rs::read_gds(p)))
        .sum();
    println!("{total}");
}
