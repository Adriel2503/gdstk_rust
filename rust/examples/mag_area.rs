//! Polygons and area per layer of a Magic hierarchy, flattened, to compare
//! against another reader such as KLayout.
//!
//! ```text
//! cargo run --release --example mag_area -- top.mag [--lambda 0.01] [--lib DIR]... [--merge]
//! ```
//!
//! Sub-cells are looked up in the directory of the `use` line (with `$VAR`
//! and `~` expanded), then next to the file that uses them, then in each
//! `--lib` directory. Output: `layer<TAB>polygons<TAB>area_um2`, sorted by
//! layer name; the area is the sum over the flattened polygons (overlaps
//! count twice), or of their union with `--merge` (slow on big layouts).
//! Missing cells and warnings go to stderr.

use std::path::{Path, PathBuf};

use gdstk_rs::magic::{collect, Found, MagOptions, Request};
use gdstk_rs::{xor_split_owned, GdsTag, Library, OwnedPolygon};

fn expand(dir: &str) -> String {
    let mut s = dir.to_string();
    if let Some(rest) = s.strip_prefix('~') {
        if let Ok(home) = std::env::var("HOME") {
            s = format!("{home}{rest}");
        }
    }
    if let Some(rest) = s.strip_prefix('$') {
        let rest = rest.trim_start_matches('{');
        let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        let (var, tail) = rest.split_at(end);
        if let Ok(v) = std::env::var(var) {
            s = format!("{v}{}", tail.trim_start_matches('}'));
        }
    }
    s
}

fn resolver(libs: Vec<PathBuf>) -> impl FnMut(&Request<'_>) -> Option<Found> {
    move |req| {
        let parent = Path::new(req.parent).parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut dirs = Vec::new();
        if let Some(d) = req.dir {
            dirs.push(parent.join(expand(d)));
        }
        dirs.push(parent);
        dirs.extend(libs.iter().cloned());
        dirs.iter().find_map(|d| {
            let p = d.join(format!("{}.mag", req.cell));
            std::fs::read(&p).ok().map(|bytes| Found { path: p.display().to_string(), bytes })
        })
    }
}

fn shoelace(points: &[gdstk_rs::Point2D]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut top = None;
    let mut lambda = 0.01;
    let mut libs = Vec::new();
    let mut merge = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--merge" => merge = true,
            "--lambda" => lambda = args.next().and_then(|v| v.parse().ok()).expect("--lambda <µm>"),
            "--lib" => libs.push(PathBuf::from(args.next().expect("--lib <dir>"))),
            _ => top = Some(PathBuf::from(a)),
        }
    }
    let top = top.expect("usage: mag_area top.mag [--lambda 0.01] [--lib DIR]...");
    let name = top.file_stem().unwrap().to_string_lossy().to_string();
    let bytes = std::fs::read(&top).expect("read top");
    let sources = collect(&name, &top.display().to_string(), bytes, resolver(libs)).expect("parse top");
    let (lib, info) = Library::from_mag(&sources, &MagOptions { lambda_um: lambda, ..Default::default() });
    for m in &info.missing {
        eprintln!("missing cell: {m}");
    }
    for w in &info.warnings {
        eprintln!("warning: {w}");
    }
    eprintln!("{} files, {} cells, grid {} µm", sources.files.len(), lib.cell_count(), info.grid_um);

    let cell = lib.cell(0);
    for (layer, tag) in &info.layers {
        let flat = cell.get_polygons().with_filter(tag.layer, tag.datatype).build();
        let polys: Vec<OwnedPolygon> = flat
            .polygons()
            .map(|p| OwnedPolygon { layer: tag.layer, datatype: tag.datatype, points: p.points().collect() })
            .collect();
        if polys.is_empty() {
            continue;
        }
        let area: f64 = if merge {
            let merged = xor_split_owned(&polys, &[], GdsTag { layer: tag.layer, datatype: tag.datatype });
            merged.removed.iter().map(|p| shoelace(&p.points)).sum()
        } else {
            polys.iter().map(|p| shoelace(&p.points)).sum()
        };
        println!("{layer}\t{}\t{area:.6}", polys.len());
    }
}
