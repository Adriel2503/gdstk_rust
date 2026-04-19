//! Solo pasada paralela (std::thread, un hilo por archivo).
//! Para comparación justa con Python paralelo.

use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: count_many_parallel <file1.gds> ...");
        std::process::exit(2);
    }
    let files = Arc::new(files);
    let handles: Vec<_> = (0..files.len())
        .map(|i| {
            let files = Arc::clone(&files);
            thread::spawn(move || {
                let lib = gdstk_rs::read_gds(&files[i]);
                gdstk_rs::cell_count(&lib)
            })
        })
        .collect();
    let total: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    println!("{total}");
}
