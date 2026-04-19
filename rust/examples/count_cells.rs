//! Minimal end-to-end test for Fase 1.
//!
//! Usage:
//!   cargo run --example count_cells -- path/to/file.gds
//!
//! Expected: prints the number of top-level cells in the GDS file,
//! matching `len(gdstk.read_gds(path).cells)` in Python.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "count_cells".into());

    let Some(path) = args.next() else {
        eprintln!("usage: {program} <file.gds>");
        return ExitCode::from(2);
    };

    let lib = gdstk_rs::read_gds(&path);
    let count = gdstk_rs::cell_count(&lib);
    println!("{count}");

    ExitCode::SUCCESS
}
