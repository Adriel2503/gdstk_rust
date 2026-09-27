//! OASIS: lectura desde bytes, deteccion de formato y equivalencia con GDSII.

mod common;

use std::collections::BTreeMap;

use common::proof_lib_path;
use gdstk_rs::{sniff_format, Library, LayoutFormat};

/// Poligonos aplanados y area total (m²) por (cell, layer, datatype).
fn summary(lib: &Library) -> BTreeMap<(String, u32, u32), (usize, f64)> {
    let u2 = lib.unit() * lib.unit();
    let mut out = BTreeMap::new();
    for cell in lib.cells() {
        for p in cell.get_polygons().build().polygons() {
            let e = out.entry((cell.name().to_string(), p.layer(), p.datatype())).or_insert((0, 0.0));
            e.0 += 1;
            e.1 += p.area() * u2;
        }
    }
    out
}

fn proof_lib_as_oas() -> Vec<u8> {
    // Un nombre por llamada: los tests corren en paralelo en el mismo proceso
    // y uno borraría el archivo que otro está por leer.
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let lib = Library::open(&proof_lib_path());
    let path = std::env::temp_dir().join(format!("gdstk_rs_oas_test_{}_{seq}.oas", std::process::id()));
    lib.write_oas(path.to_str().unwrap()).expect("write_oas");
    let bytes = std::fs::read(&path).expect("leer .oas");
    let _ = std::fs::remove_file(&path);
    bytes
}

#[test]
fn sniff_detects_both_formats() {
    let gds = std::fs::read(proof_lib_path()).unwrap();
    assert_eq!(sniff_format(&gds), Some(LayoutFormat::Gds));
    assert_eq!(sniff_format(&proof_lib_as_oas()), Some(LayoutFormat::Oasis));
    assert_eq!(sniff_format(b"<svg/>"), None);
    assert_eq!(sniff_format(&[]), None);
}

#[test]
fn oasis_roundtrip_matches_gds_geometry() {
    let gds = Library::open(&proof_lib_path());
    let oas = Library::from_bytes_any(&proof_lib_as_oas()).expect("from_bytes_any .oas");
    assert!((oas.unit() - 1e-6).abs() < 1e-15, "OASIS usa 1 µm: {}", oas.unit());
    let (a, b) = (summary(&gds), summary(&oas));
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (k, (n, area)) in &a {
        let (m, area_b) = b[k];
        assert_eq!(*n, m, "{k:?}");
        assert!((area - area_b).abs() <= 1e-9 * area.abs().max(1e-18), "{k:?}: {area} vs {area_b}");
    }
}

#[test]
fn invalid_oasis_bytes_are_an_error() {
    let mut bad = gdstk_rs::OASIS_MAGIC.to_vec();
    bad.extend_from_slice(b"truncated");
    assert!(Library::from_oas_bytes(&bad).is_err());
}
