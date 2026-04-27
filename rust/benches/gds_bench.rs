//! Criterion benchmarks para las operaciones principales de gdstk-rs.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use gdstk_rs::{Library, gds_info};

fn proof_lib_path() -> String {
    format!("{}/../tests/proof_lib.gds", env!("CARGO_MANIFEST_DIR"))
}

fn bench_read_gds(c: &mut Criterion) {
    let path = proof_lib_path();
    c.bench_function("read_gds_proof_lib", |b| {
        b.iter(|| {
            let lib = Library::open(black_box(&path));
            black_box(lib.cell_count());
        })
    });
}

fn bench_gds_info(c: &mut Criterion) {
    let path = proof_lib_path();
    c.bench_function("gds_info_proof_lib", |b| {
        b.iter(|| {
            let info = gds_info(black_box(&path)).unwrap();
            black_box(info.cell_count());
        })
    });
}

fn bench_xor_with_self(c: &mut Criterion) {
    let lib = Library::open(&proof_lib_path());
    c.bench_function("cell_xor_with_self_layer_0", |b| {
        b.iter(|| {
            let cell = lib.cell(0);
            let m = cell.xor_with(&cell, black_box(0));
            black_box(m);
        })
    });
}

fn bench_iterate_polygons(c: &mut Criterion) {
    let lib = Library::open(&proof_lib_path());
    c.bench_function("iterate_polygons_all_cells", |b| {
        b.iter(|| {
            let mut total_area = 0.0f64;
            for cell in lib.cells() {
                for poly in cell.polygons() {
                    total_area += poly.area();
                }
            }
            black_box(total_area);
        })
    });
}

criterion_group!(
    benches,
    bench_read_gds,
    bench_gds_info,
    bench_xor_with_self,
    bench_iterate_polygons
);
criterion_main!(benches);
