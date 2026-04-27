//! Compara tiempo de gds_info (peek) vs Library::open (full parse).

use std::env;
use std::time::Instant;

fn main() {
    let path = env::args()
        .nth(1)
        .expect("usage: gds_info_bench <file.gds>");

    // gds_info (peek rápido)
    let t0 = Instant::now();
    let info = gdstk_rs::gds_info(&path).expect("gds_info failed");
    let gds_info_ms = t0.elapsed().as_secs_f64() * 1000.0;

    // Library::open (parse completo)
    let t1 = Instant::now();
    let lib = gdstk_rs::Library::open(&path);
    let read_gds_ms = t1.elapsed().as_secs_f64() * 1000.0;

    println!("gds_info    : {:>7.2} ms", gds_info_ms);
    println!("Library::open: {:>7.2} ms", read_gds_ms);
    println!("speedup     : {:>7.2}x", read_gds_ms / gds_info_ms);
    println!();
    println!("Metadata del gds_info:");
    println!("  unit: {}", info.unit());
    println!("  precision: {}", info.precision());
    println!("  cells: {}", info.cell_count());
    println!("  polygons: {}", info.num_polygons());
    println!("  paths: {}", info.num_paths());
    println!("  references: {}", info.num_references());
    println!("  labels: {}", info.num_labels());
    println!("  shape tags: {}", info.shape_tag_count());
    println!("  label tags: {}", info.label_tag_count());
    println!();
    println!("Library completa:");
    println!("  cells: {}", lib.cell_count());
    println!("  top_level: {}", lib.top_level().count());
}
