//! Imprime paths detallados por celda. Fase 5.5.

use std::env;

use gdstk_rs::{BendType, EndType, JoinType, Library};

fn end_name(e: EndType) -> &'static str {
    match e {
        EndType::Flush => "flush",
        EndType::Round => "round",
        EndType::HalfWidth => "half-width",
        EndType::Extended => "extended",
        EndType::Smooth => "smooth",
        EndType::Function => "function",
    }
}

fn join_name(j: JoinType) -> &'static str {
    match j {
        JoinType::Natural => "natural",
        JoinType::Miter => "miter",
        JoinType::Bevel => "bevel",
        JoinType::Round => "round",
        JoinType::Smooth => "smooth",
        JoinType::Function => "function",
    }
}

fn bend_name(b: BendType) -> &'static str {
    match b {
        BendType::None => "none",
        BendType::Circular => "circular",
        BendType::Function => "function",
    }
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "list_paths".into());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <file.gds>");
        std::process::exit(2);
    };

    let lib = Library::open(&path);
    for cell in lib.cells() {
        let fp_n = cell.flexpath_count();
        let rp_n = cell.robustpath_count();
        if fp_n == 0 && rp_n == 0 {
            continue;
        }
        println!(
            "Cell '{}': {} flexpath, {} robustpath",
            cell.name(),
            fp_n,
            rp_n
        );
        for (i, fp) in cell.flexpaths().enumerate() {
            println!(
                "  flexpath[{i}]: simple={} scale={} spine_pts={}",
                fp.simple_path(),
                fp.scale_width(),
                fp.spine_point_count()
            );
            for e in 0..fp.num_elements() {
                println!(
                    "    element[{e}]: layer {}/{} end={} join={} bend={} hw0={:.4}",
                    fp.element_layer(e),
                    fp.element_datatype(e),
                    end_name(fp.element_end_type(e)),
                    join_name(fp.element_join_type(e)),
                    bend_name(fp.element_bend_type(e)),
                    fp.element_half_width(e, 0)
                );
            }
        }
        for (i, rp) in cell.robustpaths().enumerate() {
            let ep = rp.end_point();
            println!(
                "  robustpath[{i}]: simple={} scale={} subpaths={} end=({:.2},{:.2}) tol={:.4}",
                rp.simple_path(),
                rp.scale_width(),
                rp.subpath_count(),
                ep.x,
                ep.y,
                rp.tolerance()
            );
            for e in 0..rp.num_elements() {
                println!(
                    "    element[{e}]: layer {}/{} end={} ew={:.4} eo={:.4}",
                    rp.element_layer(e),
                    rp.element_datatype(e),
                    end_name(rp.element_end_type(e)),
                    rp.element_end_width(e),
                    rp.element_end_offset(e)
                );
            }
        }
    }
}
