//! Benchmark: leer N archivos GDS secuencial vs paralelo.
//!
//! Usage:
//!   count_many <file1.gds> <file2.gds> ...
//!
//! Imprime:
//!   - celdas por archivo
//!   - total secuencial vs paralelo
//!   - speedup

use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn count_one(path: &str) -> u64 {
    let lib = gdstk_rs::read_gds(path);
    gdstk_rs::cell_count(&lib)
}

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: count_many <file1.gds> <file2.gds> ...");
        std::process::exit(2);
    }

    println!("{} archivos GDS\n", files.len());

    // --- Secuencial ---
    let t0 = Instant::now();
    let seq_totals: Vec<(String, u64)> = files
        .iter()
        .map(|p| (p.clone(), count_one(p)))
        .collect();
    let seq_elapsed = t0.elapsed();

    for (path, count) in &seq_totals {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(path);
        println!("  {name:30} {count} celdas");
    }
    println!("\nSecuencial: {:?}", seq_elapsed);

    // --- Paralelo (std::thread, un hilo por archivo) ---
    let files_arc = Arc::new(files.clone());
    let t1 = Instant::now();
    let handles: Vec<_> = (0..files_arc.len())
        .map(|i| {
            let files = Arc::clone(&files_arc);
            thread::spawn(move || count_one(&files[i]))
        })
        .collect();
    let _par_results: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let par_elapsed = t1.elapsed();

    println!("Paralelo:   {:?}", par_elapsed);

    let speedup = seq_elapsed.as_secs_f64() / par_elapsed.as_secs_f64();
    println!("Speedup:    {:.2}x", speedup);
}
