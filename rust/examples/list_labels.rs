//! Imprime labels por celda. Test de paridad para Fase 3.

use std::env;

use gdstk_rs::Library;

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "list_labels".into());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <file.gds>");
        std::process::exit(2);
    };

    let lib = Library::open(&path);
    for cell in lib.cells() {
        if cell.label_count() == 0 {
            continue;
        }
        println!("Cell '{}':", cell.name());
        for label in cell.labels() {
            let p = label.origin();
            println!(
                "  Layer {}/{}  ({:.2}, {:.2})  '{}'",
                label.layer(),
                label.texttype(),
                p.x,
                p.y,
                label.text()
            );
        }
    }
}
