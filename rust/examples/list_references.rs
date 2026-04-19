//! Imprime referencias por celda. Test de paridad Fase 4.

use std::env;

use gdstk_rs::Library;

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "list_references".into());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <file.gds>");
        std::process::exit(2);
    };

    let lib = Library::open(&path);
    for cell in lib.cells() {
        if cell.reference_count() == 0 {
            continue;
        }
        println!("Cell '{}':", cell.name());
        for r in cell.references() {
            let p = r.origin();
            println!(
                "  -> '{}' @ ({:.2}, {:.2}) rot={:.3} mag={:.3} xrefl={}",
                r.cell_name(),
                p.x,
                p.y,
                r.rotation(),
                r.magnification(),
                r.x_reflection()
            );
        }
    }
}
