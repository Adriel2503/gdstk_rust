//! Magic `.mag` hierarchies read into a `Library`: units, transforms,
//! arrays, mixed `magscale`, missing cells, layer names.
//!
//! Two tests are `#[ignore]` because they need files outside the repo:
//! - `pdk_corpus`: every `.mag` listed in `$MAG_CORPUS` (one path per line)
//!   parses, and its hierarchy (resolved in its own directory and in the
//!   directories of `$MAG_LIBS`, `:`-separated) builds a library.
//! - `klayout_testdata`: the uncompressed cases of KLayout's
//!   `testdata/magic` (`$KLAYOUT_TESTDATA`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gdstk_rs::magic::{collect, default_layer_of, Found, MagOptions};
use gdstk_rs::{GdsTag, Library};

/// Resolver over an in-memory set of files named `<cell>.mag`.
fn from_map(files: &HashMap<&str, &str>) -> impl FnMut(&gdstk_rs::magic::Request<'_>) -> Option<Found> {
    let files: HashMap<String, String> = files.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    move |req| {
        files.get(req.cell).map(|t| Found { path: format!("{}.mag", req.cell), bytes: t.as_bytes().to_vec() })
    }
}

fn read(top: &str, files: &HashMap<&str, &str>, opts: &MagOptions) -> (Library, gdstk_rs::magic::MagInfo) {
    let sources = collect(top, &format!("{top}.mag"), files[top].as_bytes().to_vec(), from_map(files)).unwrap();
    Library::from_mag(&sources, opts)
}

/// Flattened area of one layer of the top cell, in µm².
fn area(lib: &Library, tag: GdsTag) -> f64 {
    let top = lib.cell(0);
    let flat = top.get_polygons().with_filter(tag.layer, tag.datatype).build();
    flat.polygons().map(|p| p.area()).sum()
}

fn bbox(lib: &Library, tag: GdsTag) -> [f64; 4] {
    let top = lib.cell(0);
    let flat = top.get_polygons().with_filter(tag.layer, tag.datatype).build();
    let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for p in flat.polygons() {
        for q in p.points() {
            b = [b[0].min(q.x), b[1].min(q.y), b[2].max(q.x), b[3].max(q.y)];
        }
    }
    b.map(|v| (v * 1e6).round() / 1e6)
}

const LEAF: &str = "magic
tech sky130A
magscale 1 10
<< metal1 >>
rect 0 0 100 50
<< end >>
";

// Child in magscale 1 2 with a 3×2 array of the leaf (magscale 1 10):
// the leaf is 10×5 lambda... in child units that is 20×10.
const CHILD: &str = "magic
tech sky130A
magscale 1 2
<< poly >>
rect 0 0 10 10
tri 10 0 20 10 se
<< labels >>
rlabel poly s 0 0 10 10 0 G
port 1 nsew signal input
use leaf  leaf_0
array 0 2 40 0 1 30
timestamp 1
transform 1 0 100 0 1 0
box 0 0 20 10
<< end >>
";

// Top without magscale (1 lambda per unit): the child rotated 90° and
// mirrored, plus a missing cell and a DRC mark.
const TOP: &str = "magic
tech sky130A
<< metal1 >>
rect 0 0 5 5
<< error_p >>
rect 0 0 1 1
use child  c0
transform 0 1 50 1 0 0
box 0 0 1 1
use ghost  g0
transform 1 0 0 0 1 0
box 0 0 1 1
<< end >>
";

fn files() -> HashMap<&'static str, &'static str> {
    HashMap::from([("top", TOP), ("child", CHILD), ("leaf", LEAF)])
}

#[test]
fn hierarchy_units_arrays_and_names() {
    let lambda = 0.01;
    let (lib, info) = read("top", &files(), &MagOptions { lambda_um: lambda, ..Default::default() });
    assert_eq!(lib.cell_count(), 4, "top, child, leaf and the missing ghost");
    assert_eq!(lib.cell(0).name(), "top");
    assert_eq!(info.missing, vec!["ghost".to_string()]);
    // Common grid: denominators 1, 2 and 10 → lambda / 10.
    assert!((info.grid_um - lambda / 10.0).abs() < 1e-15);
    assert!((lib.precision() - lambda / 10.0 * 1e-6).abs() < 1e-18);

    let m1 = default_layer_of("metal1");
    let poly = default_layer_of("poly");
    // metal1: 5×5 lambda of top + 6 leaves of 10×5 lambda.
    let want_m1 = (25.0 + 6.0 * 50.0) * lambda * lambda;
    assert!((area(&lib, m1) - want_m1).abs() < 1e-12, "{}", area(&lib, m1));
    // poly: 5×5 lambda box + triangle of half 5×5 lambda.
    let want_poly = (25.0 + 12.5) * lambda * lambda;
    assert!((area(&lib, poly) - want_poly).abs() < 1e-12);
    // DRC marks are not geometry.
    assert_eq!(area(&lib, default_layer_of("error_p")), 0.0);

    // transform 0 1 50 / 1 0 0 maps child (x, y) to (y + 50, x): the poly
    // box (0..5, 0..5) lambda lands on (50..55, 0..5), the triangle
    // (5..10, 0..5) on (50..55, 5..10).
    assert_eq!(bbox(&lib, poly), [0.5, 0.0, 0.55, 0.1]);
    // Leaves: child places them at x = 50 + i·20, y = j·15 lambda (40 and 30
    // child units), each 10×5; the top swaps x and y.
    assert_eq!(bbox(&lib, m1), [0.0, 0.0, 0.7, 1.0]);

    let names: HashMap<String, GdsTag> = lib.layer_names().into_iter().map(|(t, n)| (n, t)).collect();
    assert_eq!(names["metal1"], m1);
    assert_eq!(names["poly"], poly);
    assert!(!names.contains_key("error_p"), "hint layer without labels has no name");

    let child = info.cells.iter().find(|c| c.name == "child").unwrap();
    assert_eq!(child.ports.len(), 1);
    assert_eq!(child.ports[0].name, "G");
    assert_eq!(child.ports[0].port.class.as_deref(), Some("input"));
    assert_eq!(child.ports[0].rect_um, [0.0, 0.0, 0.05, 0.05]);
}

#[test]
fn reversed_array_steps_backwards_like_magic() {
    let f = HashMap::from([
        ("top", "magic\ntech t\nuse leaf  l\narray 2 0 10 0 0 0\ntransform 1 0 0 0 1 0\nbox 0 0 1 1\n"),
        ("leaf", "magic\ntech t\n<< m >>\nrect 0 0 1 1\n"),
    ]);
    let (lib, _) = read("top", &f, &MagOptions { lambda_um: 1.0, ..Default::default() });
    // Three copies at x = 0, -10, -20.
    assert_eq!(bbox(&lib, default_layer_of("m")), [-20.0, 0.0, 1.0, 1.0]);
    assert!((area(&lib, default_layer_of("m")) - 3.0).abs() < 1e-12);
}

#[test]
fn custom_layer_numbers_and_hint_layers() {
    let map = std::sync::Arc::new(|name: &str| match name {
        "metal1" => GdsTag { layer: 68, datatype: 20 },
        _ => GdsTag { layer: 999, datatype: 0 },
    });
    let opts = MagOptions { lambda_um: 0.01, layer_of: Some(map), keep_hint_layers: true };
    let (lib, info) = read("top", &files(), &opts);
    assert!(area(&lib, GdsTag { layer: 68, datatype: 20 }) > 0.0);
    // poly and error_p share 999/0: warned, compared as one.
    assert!(info.warnings.iter().any(|w| w.contains("share 999/0")), "{:?}", info.warnings);
}

#[test]
fn cycles_are_dropped() {
    let f = HashMap::from([
        ("a", "magic\nuse b  b0\ntransform 1 0 0 0 1 0\nbox 0 0 1 1\n<< m >>\nrect 0 0 1 1\n"),
        ("b", "magic\nuse a  a0\ntransform 1 0 5 0 1 0\nbox 0 0 1 1\n<< m >>\nrect 0 0 1 1\n"),
    ]);
    let (lib, info) = read("a", &f, &MagOptions::default());
    assert!(info.warnings.iter().any(|w| w.contains("cycle")), "{:?}", info.warnings);
    assert!(area(&lib, default_layer_of("m")) > 0.0, "flattening ends");
}

#[test]
fn top_that_is_not_magic_fails() {
    assert!(collect("x", "x.mag", b"HEADER".to_vec(), |_| None).is_err());
    assert_eq!(gdstk_rs::sniff_format(TOP.as_bytes()), Some(gdstk_rs::LayoutFormat::Magic));
    assert!(Library::from_bytes_any(TOP.as_bytes()).is_err());
}

#[test]
fn layer_names_survive_oasis() {
    let (lib, _) = read("top", &files(), &MagOptions::default());
    let dir = std::env::temp_dir().join(format!("gdstk_rs_mag_{}.oas", std::process::id()));
    let path = dir.to_str().unwrap();
    lib.write_oas(path).unwrap();
    let back = Library::from_oas_bytes(&std::fs::read(path).unwrap()).unwrap();
    let _ = std::fs::remove_file(path);
    let mut a = lib.layer_names();
    let mut b = back.layer_names();
    a.sort_by(|x, y| x.1.cmp(&y.1));
    b.sort_by(|x, y| x.1.cmp(&y.1));
    assert_eq!(a, b);
}

#[test]
fn library_is_shared_between_threads() {
    let (lib, _) = read("top", &files(), &MagOptions::default());
    let m1 = default_layer_of("metal1");
    let want = area(&lib, m1);
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| assert_eq!(area(&lib, m1), want));
        }
    });
}

/// Resolver over directories: the file's own, then `$MAG_LIBS`.
fn disk_resolver(libs: Vec<PathBuf>) -> impl FnMut(&gdstk_rs::magic::Request<'_>) -> Option<Found> {
    move |req| {
        let parent = Path::new(req.parent).parent().unwrap_or(Path::new("."));
        let mut dirs = Vec::new();
        if let Some(d) = req.dir.filter(|d| !d.starts_with('$')) {
            dirs.push(parent.join(d));
        }
        dirs.push(parent.to_path_buf());
        dirs.extend(libs.iter().cloned());
        dirs.iter().find_map(|d| {
            let p = d.join(format!("{}.mag", req.cell));
            std::fs::read(&p).ok().map(|bytes| Found { path: p.display().to_string(), bytes })
        })
    }
}

fn read_path(path: &Path, libs: &[PathBuf]) -> Result<(Library, gdstk_rs::magic::MagInfo), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let name = path.file_stem().unwrap().to_string_lossy().to_string();
    let sources = collect(&name, &path.display().to_string(), bytes, disk_resolver(libs.to_vec()))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Library::from_mag(&sources, &MagOptions::default()))
}

#[test]
#[ignore = "needs $MAG_CORPUS (list of .mag files)"]
fn pdk_corpus() {
    let list = std::env::var("MAG_CORPUS").expect("MAG_CORPUS");
    let libs: Vec<PathBuf> = std::env::var("MAG_LIBS").unwrap_or_default().split(':').filter(|s| !s.is_empty()).map(PathBuf::from).collect();
    let files: Vec<String> = std::fs::read_to_string(list).unwrap().lines().map(str::to_string).collect();
    let mut failed = Vec::new();
    let mut cells = 0u64;
    for f in &files {
        let bytes = std::fs::read(f).unwrap();
        if let Err(e) = gdstk_rs::magic::parse(&bytes) {
            failed.push(format!("{f}: {e}"));
            continue;
        }
        match read_path(Path::new(f), &libs) {
            Ok((lib, _)) => cells += lib.cell_count(),
            Err(e) => failed.push(e),
        }
    }
    println!("{} files, {} cells in their hierarchies", files.len(), cells);
    assert!(failed.is_empty(), "{} failed:\n{}", failed.len(), failed.join("\n"));
}

#[test]
#[ignore = "needs $KLAYOUT_TESTDATA (klayout/testdata/magic)"]
fn klayout_testdata() {
    let base = PathBuf::from(std::env::var("KLAYOUT_TESTDATA").expect("KLAYOUT_TESTDATA"));
    for (top, libs) in [
        ("PearlRiver/Layout/magic/PearlRiver_die.mag", vec![base.join("PearlRiver")]),
        ("ringo/RINGO.mag", vec![]),
        ("issue_1925/redux.mag", vec![]),
    ] {
        let (lib, info) = read_path(&base.join(top), &libs).unwrap();
        println!("{top}: {} cells, missing {:?}", lib.cell_count(), info.missing);
        assert!(info.missing.is_empty(), "{top}: {:?}", info.missing);
        assert!(!lib.layers().is_empty());
    }
}
