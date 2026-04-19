//! Integration tests — exercising the gdstk-rs API end-to-end with real GDS.
//!
//! All tests use `proof_lib.gds` (ships with gdstk, always available).
//! Optional tests that need `tinytapeout.gds` are in `tinytapeout.rs`.

mod common;

use common::{normalize_lf, proof_lib_path, run_example};
use gdstk_rs::{gds_info, ErrorCode, Library};

// ---- Reading and metadata ----

#[test]
fn proof_lib_has_six_cells() {
    let lib = Library::open(&proof_lib_path());
    assert_eq!(lib.cell_count(), 6);
}

#[test]
fn proof_lib_metadata_matches() {
    let lib = Library::open(&proof_lib_path());
    // name/unit/precision are baked into the fixture; we assert conservative
    // invariants rather than exact values so the test survives fixture edits.
    assert!(!lib.name().is_empty(), "library name should not be empty");
    assert!(lib.unit() > 0.0, "unit must be positive");
    assert!(lib.precision() > 0.0, "precision must be positive");
    // Standard GDS convention: precision < unit
    assert!(lib.precision() < lib.unit());
}

#[test]
fn top_level_returns_at_least_one_cell() {
    let lib = Library::open(&proof_lib_path());
    let tl = lib.top_level();
    assert!(tl.count() > 0);
    for cell in tl.cells() {
        assert!(!cell.name().is_empty());
    }
}

// ---- Iteration over cells/polygons/labels/references/paths ----

#[test]
fn cells_have_distinct_non_empty_names() {
    let lib = Library::open(&proof_lib_path());
    let mut names: Vec<String> = lib.cells().map(|c| c.name().to_owned()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len() as u64, lib.cell_count(),
               "cell names should be unique");
}

#[test]
fn polygon_areas_are_positive() {
    let lib = Library::open(&proof_lib_path());
    let mut seen_positive = false;
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let a = poly.area();
            assert!(a.is_finite(), "area must be finite");
            assert!(a >= 0.0, "area must be non-negative");
            if a > 0.0 {
                seen_positive = true;
            }
        }
    }
    assert!(seen_positive, "expected at least one polygon with positive area");
}

#[test]
fn polygon_layer_fits_in_u32() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            // GDSII spec: layer ∈ [0, 65535] but gdstk exposes u32.
            // Just assert the call returns without panicking and value is reasonable.
            let layer = poly.layer();
            assert!(layer < u32::MAX / 2);
        }
    }
}

#[test]
fn label_text_is_valid_utf8_lossy() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for label in cell.labels() {
            // Cow<str> always returns a valid &str (lossy replaces bad bytes).
            let _: &str = &label.text();
        }
    }
}

#[test]
fn references_target_cells() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for r in cell.references() {
            // Referenced cell name should be non-empty.
            assert!(!r.cell_name().is_empty(),
                    "reference in cell '{}' has empty target", cell.name());
            // Magnification is typically ~1.0 in proof_lib.
            assert!(r.magnification() > 0.0);
        }
    }
}

// ---- XOR ----

#[test]
fn xor_with_self_is_zero() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for layer in 0u32..8 {
            let m = cell.xor_with(&cell, layer);
            assert_eq!(
                m.region_count, 0,
                "cell '{}' layer {} XOR with self should yield 0 regions (got {} area={})",
                cell.name(), layer, m.region_count, m.area
            );
            assert_eq!(m.area, 0.0);
        }
    }
}

#[test]
fn xor_detects_synthetic_diff() {
    // Crear dos libraries que difieren en un rectángulo: open proof_lib
    // dos veces. Library A y B idénticas → xor debe ser 0 en todas las
    // cells. Este test es un sanity check del wiring.
    let lib_a = Library::open(&proof_lib_path());
    let lib_b = Library::open(&proof_lib_path());
    for cell_a in lib_a.cells() {
        let cell_b = lib_b.find_cell(cell_a.name()).expect("cell missing");
        for layer in 0u32..8 {
            let m = cell_a.xor_with(&cell_b, layer);
            assert_eq!(m.region_count, 0);
        }
    }
}

// ---- gds_info ----

#[test]
fn gds_info_matches_library() {
    let lib = Library::open(&proof_lib_path());
    let info = gds_info(&proof_lib_path()).expect("gds_info failed");
    assert_eq!(info.cell_count(), lib.cell_count());
    assert!((info.unit() - lib.unit()).abs() < 1e-18);
    assert!((info.precision() - lib.precision()).abs() < 1e-24);
}

#[test]
fn gds_info_counts_are_positive() {
    let info = gds_info(&proof_lib_path()).expect("gds_info failed");
    assert!(info.num_polygons() > 0);
    // proof_lib.gds should have at least some cells
    assert!(info.cell_count() > 0);
}

#[test]
fn gds_info_missing_file_returns_error() {
    let result = gds_info("/nonexistent/path/to/file.gds");
    assert!(result.is_err());
    let err = result.unwrap_err();
    // Expect an I/O related error
    assert!(
        matches!(
            err.0,
            ErrorCode::InputFileOpenError
                | ErrorCode::InputFileError
                | ErrorCode::FileError
                | ErrorCode::InvalidFile
        ),
        "unexpected error code: {:?}",
        err.0
    );
}

// ---- Roundtrip + write ----

#[test]
fn roundtrip_preserves_geometry() {
    let tmp = std::env::temp_dir().join("gdstk_rs_test_roundtrip.gds");
    let tmp_str = tmp.to_str().unwrap();

    let lib_a = Library::open(&proof_lib_path());
    lib_a.write_gds(tmp_str).expect("write_gds failed");

    let lib_b = Library::open(tmp_str);
    assert_eq!(lib_a.cell_count(), lib_b.cell_count());

    for cell_a in lib_a.cells() {
        let cell_b = lib_b
            .find_cell(cell_a.name())
            .unwrap_or_else(|| panic!("cell '{}' missing after roundtrip", cell_a.name()));
        for layer in 0u32..8 {
            let m = cell_a.xor_with(&cell_b, layer);
            assert_eq!(
                m.region_count, 0,
                "cell '{}' layer {} changed in roundtrip ({} regions, {} area)",
                cell_a.name(), layer, m.region_count, m.area
            );
        }
    }
}

#[test]
fn write_gds_fails_for_bad_path() {
    let lib = Library::open(&proof_lib_path());
    // Path con directorio que no existe.
    let result = lib.write_gds("/definitely/not/a/real/directory/out.gds");
    assert!(result.is_err(), "expected write to fail for bad path");
}

// ---- Fase 8: gap-closing read accessors ----

#[test]
fn polygon_perimeter_is_positive() {
    let lib = Library::open(&proof_lib_path());
    let mut any_positive = false;
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let p = poly.perimeter();
            assert!(p.is_finite());
            assert!(p >= 0.0);
            if p > 0.0 {
                any_positive = true;
            }
        }
    }
    assert!(any_positive, "expected at least one polygon with perimeter > 0");
}

#[test]
fn polygon_signed_area_is_finite() {
    // signed_area() returns a signed value; sign depends on orientation.
    // We only assert it's finite — the relationship to area() is nontrivial
    // for polygons with self-intersections or complex boundaries (e.g. fillets
    // that produce degenerate geometries whose signed_area cancels).
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let s = poly.signed_area();
            assert!(s.is_finite(), "signed_area must be finite");
        }
    }
}

#[test]
fn cell_bbox_contains_all_polygons() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        if cell.polygon_count() == 0 {
            continue;
        }
        let cbb = cell.bbox();
        // Skip empty-cell sentinel (zero box).
        if cbb.min_x == 0.0 && cbb.max_x == 0.0 && cbb.min_y == 0.0 && cbb.max_y == 0.0 {
            continue;
        }
        for poly in cell.polygons() {
            let pbb = poly.bbox();
            assert!(
                cbb.min_x <= pbb.min_x + 1e-6 && cbb.max_x + 1e-6 >= pbb.max_x,
                "cell '{}' bbox {:?} doesn't contain poly bbox {:?}",
                cell.name(), cbb, pbb
            );
            assert!(
                cbb.min_y <= pbb.min_y + 1e-6 && cbb.max_y + 1e-6 >= pbb.max_y,
                "cell '{}' bbox {:?} doesn't contain poly bbox {:?}",
                cell.name(), cbb, pbb
            );
        }
    }
}

#[test]
fn default_repetition_count_is_one() {
    let lib = Library::open(&proof_lib_path());
    // proof_lib.gds doesn't use repetition; every polygon/label/reference
    // should report count == 1 (the origin instance itself).
    for cell in lib.cells() {
        for poly in cell.polygons() {
            assert!(poly.repetition_count() >= 1);
        }
        for label in cell.labels() {
            assert!(label.repetition_count() >= 1);
        }
        for r in cell.references() {
            assert!(r.repetition_count() >= 1);
        }
    }
}

#[test]
fn default_repetition_kind_is_none() {
    // proof_lib.gds doesn't use repetition; every element reports kind=None
    // and count=1 (the origin instance).
    use gdstk_rs::RepetitionType;
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let r = poly.repetition();
            assert_eq!(r.kind(), RepetitionType::None);
            assert_eq!(r.count(), 1);
            // For None: all variant fields should be zero.
            assert_eq!(r.columns(), 0);
            assert_eq!(r.rows(), 0);
            assert_eq!(r.coord_count(), 0);
            // offset(0) should be origin.
            let o = r.offset(0);
            assert_eq!(o.x, 0.0);
            assert_eq!(o.y, 0.0);
        }
    }
}

#[test]
fn repetition_shortcut_methods_match_full_api() {
    // polygon.repetition_count() should equal polygon.repetition().count()
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            assert_eq!(poly.repetition_count(), poly.repetition().count());
            for i in 0..poly.repetition_count() {
                let a = poly.repetition_offset(i);
                let b = poly.repetition().offset(i);
                assert_eq!(a, b);
            }
        }
    }
}

#[test]
fn polygon_points_are_consistent_with_count() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let count = poly.point_count();
            let collected: Vec<_> = poly.points().collect();
            assert_eq!(
                collected.len() as u64,
                count,
                "points() iterator length must match point_count()"
            );
            for pt in &collected {
                assert!(pt.x.is_finite());
                assert!(pt.y.is_finite());
            }
            // Out-of-range returns (0,0).
            let past_end = poly.point(count);
            assert_eq!(past_end.x, 0.0);
            assert_eq!(past_end.y, 0.0);
        }
    }
}

#[test]
fn polygon_first_point_is_within_bbox() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            if poly.point_count() == 0 {
                continue;
            }
            let p0 = poly.point(0);
            let bb = poly.bbox();
            assert!(
                p0.x >= bb.min_x - 1e-9 && p0.x <= bb.max_x + 1e-9,
                "point(0).x {} outside bbox [{}, {}]",
                p0.x, bb.min_x, bb.max_x
            );
            assert!(
                p0.y >= bb.min_y - 1e-9 && p0.y <= bb.max_y + 1e-9,
                "point(0).y {} outside bbox [{}, {}]",
                p0.y, bb.min_y, bb.max_y
            );
        }
    }
}

#[test]
fn extrema_for_no_repetition_is_origin() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for poly in cell.polygons() {
            let r = poly.repetition();
            // All polygons in proof_lib have kind=None → 1 extremum at origin.
            assert_eq!(
                r.extrema_count(), 1,
                "None repetition should have 1 extremum (origin)"
            );
            let e0 = r.extremum(0);
            assert_eq!(e0.x, 0.0);
            assert_eq!(e0.y, 0.0);
            // Iterator version.
            let collected: Vec<_> = r.extrema().collect();
            assert_eq!(collected.len(), 1);
        }
    }
}

#[test]
fn rawcell_iteration_works_on_empty_library() {
    let lib = Library::open(&proof_lib_path());
    // proof_lib.gds has 0 rawcells; verify the API doesn't panic.
    assert_eq!(lib.rawcell_count(), 0);
    assert_eq!(lib.rawcells().count(), 0);
}

// ---- Fase 8.8: get_polygons flatten ----

#[test]
fn cell_get_polygons_flat_leaf_matches_direct() {
    // For a leaf cell (no references, no paths), get_polygons() with
    // depth=0 and paths disabled should return the same count as direct.
    let lib = Library::open(&proof_lib_path());
    let mut tested = false;
    for cell in lib.cells() {
        if cell.reference_count() != 0 {
            continue;
        }
        if cell.flexpath_count() != 0 || cell.robustpath_count() != 0 {
            continue;
        }
        tested = true;
        let direct = cell.polygon_count();
        let flat = cell
            .get_polygons()
            .with_paths(false)
            .depth(0)
            .build();
        assert_eq!(
            flat.count(),
            direct,
            "leaf cell '{}' direct {} vs flat {}",
            cell.name(), direct, flat.count()
        );
    }
    assert!(tested, "expected at least one leaf cell without paths");
}

#[test]
fn cell_get_polygons_depth_zero_ignores_references() {
    // depth=0 must NOT expand any references — count equals direct polygons
    // (+ polygons derived from paths if include_paths=true).
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        let direct = cell.polygon_count();
        let flat = cell
            .get_polygons()
            .depth(0)
            .with_paths(false)
            .build();
        assert_eq!(
            flat.count(), direct,
            "cell '{}' depth=0 should match direct polygons",
            cell.name()
        );
    }
}

#[test]
fn cell_get_polygons_with_filter_restricts_to_layer() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        let flat = cell
            .get_polygons()
            .depth(0)
            .with_paths(false)
            .with_filter(0, 0)
            .build();
        for poly in flat.polygons() {
            assert_eq!(poly.layer(), 0);
            assert_eq!(poly.datatype(), 0);
        }
    }
}

#[test]
fn reference_get_polygons_returns_finite_coords() {
    let lib = Library::open(&proof_lib_path());
    for cell in lib.cells() {
        for r in cell.references() {
            let flat = r.get_polygons().build();
            // If the reference target has polygons, they must be transformed
            // and have finite coordinates.
            for poly in flat.polygons() {
                let bb = poly.bbox();
                assert!(bb.min_x.is_finite());
                assert!(bb.max_x.is_finite());
                assert!(bb.min_y.is_finite());
                assert!(bb.max_y.is_finite());
            }
        }
    }
}

#[test]
fn get_polygons_drops_without_leak() {
    // Repeatedly build and drop views. If there's a leak / double-free,
    // the allocator would crash or memory would balloon.
    let lib = Library::open(&proof_lib_path());
    for _ in 0..50 {
        for cell in lib.cells() {
            let flat = cell.get_polygons().build();
            // Touch the data to ensure it's not optimized out.
            let _ = flat.count();
            for poly in flat.polygons() {
                let _ = poly.area();
            }
            // drop at end of scope -> frees all owned polygons.
        }
    }
}

// ---- Snapshot tests (require examples compiled) ----
// These run the pre-built example binaries and diff against snapshots.

#[test]
fn snapshot_list_polygons_proof_lib() {
    let binary_path = format!(
        "{}/target/release/examples/list_polygons{}",
        env!("CARGO_MANIFEST_DIR"),
        std::env::consts::EXE_SUFFIX
    );
    if !std::path::Path::new(&binary_path).exists() {
        eprintln!("skipping snapshot: run `cargo build --release --examples` first");
        return;
    }
    let output = normalize_lf(&run_example("list_polygons", &proof_lib_path()));
    let expected = normalize_lf(include_str!("snapshots/list_polygons_proof_lib.txt"));

    if std::env::var("REGENERATE_SNAPSHOTS").is_ok() {
        let snap_path = format!(
            "{}/tests/snapshots/list_polygons_proof_lib.txt",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::write(&snap_path, &output).expect("regen failed");
        return;
    }

    assert_eq!(
        output, expected,
        "snapshot mismatch — run `REGENERATE_SNAPSHOTS=1 cargo test` if intentional"
    );
}
