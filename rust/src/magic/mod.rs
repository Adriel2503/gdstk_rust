//! Magic `.mag` layouts, read into the same [`Library`] as GDSII and OASIS.
//!
//! A `.mag` holds one cell; its sub-cells (`use`) live in other files. The
//! reader does not know where those files are (a directory, a Git commit, a
//! PDK): [`collect`] asks a resolver function for each one, and
//! [`Library::from_mag`] builds the library from what was collected.
//!
//! ```no_run
//! use gdstk_rs::magic::{collect, Found, MagOptions};
//! use gdstk_rs::Library;
//!
//! let top = std::fs::read("inv.mag").unwrap();
//! let sources = collect("inv", "inv.mag", top, |req| {
//!     let path = format!("{}.mag", req.cell);
//!     std::fs::read(&path).ok().map(|bytes| Found { path, bytes })
//! })
//! .unwrap();
//! let (lib, info) = Library::from_mag(&sources, &MagOptions::default());
//! println!("{} cells, missing: {:?}", lib.cell_count(), info.missing);
//! ```
//!
//! Units: a file coordinate is `value · n / d` lambda (`magscale n d`), and
//! lambda comes from the technology, not from the file (0.01 µm in SKY130,
//! 0.05 µm in GF180): it is [`MagOptions::lambda_um`]. Files of one hierarchy
//! may use different `magscale`; they are all placed on one common grid, so
//! coordinates stay exact.
//!
//! Layers: Magic layers have names (`metal1`, `ndiffc`). Each name becomes a
//! `(layer, datatype)` with [`MagOptions::layer_of`] (a stable hash of the
//! name by default, so two libraries read separately agree), and the name is
//! kept in [`Library::layer_names`].

mod parse;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub use parse::{is_mag, parse, ArraySpec, Corner, Label, LayerPaint, MagCell, ParseError, Port, Scale, Use};

use crate::{Anchor, CellId, GdsTag, Library, LibraryBuilder, Placement, Point2D};

/// Layers of Magic that are not mask data: DRC marks, router hints and the
/// empty type. Their paint is left out unless
/// [`MagOptions::keep_hint_layers`] (labels on them are kept).
pub const HINT_LAYERS: &[&str] = &[
    "space", "checkpaint", "CP", "checksubcell", "CS", "error_p", "EP", "error_s", "ES", "error_ps", "EPS",
    "magnet", "fence", "rotate",
];

/// A sub-cell the reader needs.
#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    /// Cell name; its file is `<cell>.mag`.
    pub cell: &'a str,
    /// Directory written in the `use` line, as is (may start with `$VAR`
    /// or `~`, or be relative to the file that uses the cell).
    pub dir: Option<&'a str>,
    /// Path of the file with the `use` (as given by the resolver).
    pub parent: &'a str,
}

/// A file given by the resolver.
#[derive(Clone, Debug)]
pub struct Found {
    /// Where it was found; shown in warnings and passed back as
    /// [`Request::parent`] for its own sub-cells.
    pub path: String,
    pub bytes: Vec<u8>,
}

/// One file of a hierarchy.
#[derive(Clone, Debug)]
pub struct MagFile {
    pub name: String,
    pub path: String,
    pub bytes: Vec<u8>,
    pub cell: MagCell,
}

/// Every file of a hierarchy, read by [`collect`]: the top first, then the
/// sub-cells level by level.
#[derive(Clone, Debug, Default)]
pub struct MagSources {
    pub files: Vec<MagFile>,
    /// Cells no file was found for (they stay as empty cells).
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
}

impl MagSources {
    /// Bytes of every file, in order: what the result depends on (for a
    /// cache key).
    pub fn contents(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.files.iter().map(|f| (f.name.as_str(), f.bytes.as_slice()))
    }

    /// Technology of the top cell (`tech` line).
    pub fn tech(&self) -> Option<&str> {
        self.files.first().and_then(|f| f.cell.tech.as_deref())
    }
}

/// Read the hierarchy of `top`: parse it, ask `resolve` for each sub-cell
/// the first time it is used, and go on with those files. Cells are known by
/// name (as in Magic, where cell names are global): the same name used from
/// two directories reads the first one, with a warning. Files of one level
/// are parsed in parallel (feature `parallel`).
///
/// Fails only if the top file cannot be parsed; a sub-cell that cannot be
/// found or parsed is reported and left empty.
pub fn collect(
    top_name: &str,
    top_path: &str,
    top_bytes: Vec<u8>,
    mut resolve: impl FnMut(&Request<'_>) -> Option<Found>,
) -> Result<MagSources, ParseError> {
    let mut out = MagSources::default();
    let mut dirs: HashMap<String, Option<String>> = HashMap::new();
    dirs.insert(top_name.to_string(), None);
    let mut level = vec![(top_name.to_string(), Found { path: top_path.to_string(), bytes: top_bytes })];
    let mut first = true;
    while !level.is_empty() {
        let parsed = parse_all(&level);
        let mut next = Vec::new();
        for ((name, found), cell) in level.into_iter().zip(parsed) {
            let cell = match cell {
                Ok(c) => c,
                Err(e) if first => return Err(e),
                Err(e) => {
                    out.warnings.push(format!("{}: {e}; cell {name} left empty", found.path));
                    out.missing.push(name);
                    continue;
                }
            };
            out.warnings.extend(cell.warnings.iter().map(|w| format!("{}: {w}", found.path)));
            for u in &cell.uses {
                match dirs.get(&u.cell) {
                    Some(d) => {
                        if u.dir.is_some() && d.is_some() && &u.dir != d {
                            out.warnings.push(format!(
                                "{}: cell {} used from {} and {}; the first one is read",
                                found.path,
                                u.cell,
                                d.as_deref().unwrap_or("."),
                                u.dir.as_deref().unwrap_or(".")
                            ));
                        }
                    }
                    None => {
                        dirs.insert(u.cell.clone(), u.dir.clone());
                        let req = Request { cell: &u.cell, dir: u.dir.as_deref(), parent: &found.path };
                        match resolve(&req) {
                            Some(f) => next.push((u.cell.clone(), f)),
                            None => out.missing.push(u.cell.clone()),
                        }
                    }
                }
            }
            out.files.push(MagFile { name, path: found.path, bytes: found.bytes, cell });
        }
        first = false;
        level = next;
    }
    Ok(out)
}

#[cfg(feature = "parallel")]
fn parse_all(level: &[(String, Found)]) -> Vec<Result<MagCell, ParseError>> {
    use rayon::prelude::*;
    level.par_iter().map(|(_, f)| parse(&f.bytes)).collect()
}

#[cfg(not(feature = "parallel"))]
fn parse_all(level: &[(String, Found)]) -> Vec<Result<MagCell, ParseError>> {
    level.iter().map(|(_, f)| parse(&f.bytes)).collect()
}

/// How to turn Magic units and layer names into a library.
#[derive(Clone)]
pub struct MagOptions {
    /// Lambda in µm: 0.01 in SKY130 and IHP, 0.05 in GF180 (the
    /// `scalefactor` of the `cifoutput` section of the technology file).
    pub lambda_um: f64,
    /// `(layer, datatype)` of each Magic layer name; [`default_layer_of`]
    /// when `None`.
    pub layer_of: Option<Arc<dyn Fn(&str) -> GdsTag + Send + Sync>>,
    /// Keep the paint of [`HINT_LAYERS`] (DRC marks, router hints).
    pub keep_hint_layers: bool,
}

impl Default for MagOptions {
    fn default() -> Self {
        Self { lambda_um: 0.01, layer_of: None, keep_hint_layers: false }
    }
}

impl std::fmt::Debug for MagOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagOptions")
            .field("lambda_um", &self.lambda_um)
            .field("layer_of", &self.layer_of.as_ref().map(|_| "custom"))
            .field("keep_hint_layers", &self.keep_hint_layers)
            .finish()
    }
}

/// Stable `(layer, datatype)` for a layer name, the same in every library:
/// a 30-bit hash of the name with bit 30 set (above any GDSII layer), and
/// datatype 0.
pub fn default_layer_of(name: &str) -> GdsTag {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    let folded = (h ^ (h >> 32)) as u32;
    GdsTag { layer: (1 << 30) | (folded & ((1 << 30) - 1)), datatype: 0 }
}

/// A `port` of a cell, with its label.
#[derive(Clone, Debug, PartialEq)]
pub struct PortInfo {
    /// Text of the label (the port name).
    pub name: String,
    pub layer: String,
    pub port: Port,
    /// Rectangle of the label in µm, in the cell's coordinates.
    pub rect_um: [f64; 4],
}

/// What a library read from Magic has besides geometry.
#[derive(Clone, Debug, Default)]
pub struct CellInfo {
    pub name: String,
    pub path: String,
    pub tech: Option<String>,
    pub ports: Vec<PortInfo>,
    /// `<< properties >>` of the file (key, raw value).
    pub properties: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default)]
pub struct MagInfo {
    /// One per file, in the order of [`MagSources::files`].
    pub cells: Vec<CellInfo>,
    /// Cells left empty because no file was found.
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
    /// Layer names and their tags, sorted by name.
    pub layers: Vec<(String, GdsTag)>,
    /// Size of the common grid in µm (lambda / common denominator of the
    /// `magscale` of the files).
    pub grid_um: f64,
}

impl Library {
    /// Library of a Magic hierarchy read with [`collect`]. Cells are named
    /// like their files; the top cell comes first. Coordinates in µm
    /// (`unit` 1e-6) and `precision` is the common grid.
    pub fn from_mag(sources: &MagSources, opts: &MagOptions) -> (Library, MagInfo) {
        build(sources, opts)
    }
}

fn lcm(a: u64, b: u64) -> u64 {
    (a / parse::gcd(a, b)).saturating_mul(b)
}

/// Rotation (radians) and mirror of a Manhattan `a b d e`, as gdstk applies
/// them: mirror in x first, then rotate. `(a, d)` is the image of the x
/// axis, so the angle is `atan2(d, a)`; the determinant tells the mirror.
fn orientation(t: &[i64; 6]) -> (f64, bool) {
    let [a, b, _, d, e, _] = *t;
    let rotation = match (a, d) {
        (1, 0) => 0.0,
        (0, 1) => std::f64::consts::FRAC_PI_2,
        (-1, 0) => std::f64::consts::PI,
        _ => -std::f64::consts::FRAC_PI_2,
    };
    (rotation, a * e - b * d < 0)
}

/// Point where a label is anchored and gdstk's anchor, from Magic's
/// justification (the text goes to that side of the point).
fn label_anchor(r: [f64; 4], pos: u8) -> (Point2D, Anchor) {
    let [xl, yl, xh, yh] = r;
    let (cx, cy) = ((xl + xh) / 2.0, (yl + yh) / 2.0);
    // pos: 0 center, 1 n, 2 ne, 3 e, 4 se, 5 s, 6 sw, 7 w, 8 nw
    let x = match pos {
        2..=4 => xh,
        6..=8 => xl,
        _ => cx,
    };
    let y = match pos {
        1 | 2 | 8 => yh,
        4..=6 => yl,
        _ => cy,
    };
    let anchor = match pos {
        1 => Anchor::S,
        2 => Anchor::SW,
        3 => Anchor::W,
        4 => Anchor::NW,
        5 => Anchor::N,
        6 => Anchor::NE,
        7 => Anchor::E,
        8 => Anchor::SE,
        _ => Anchor::O,
    };
    (Point2D { x, y }, anchor)
}

/// Uses that close a cycle (a cell inside itself): Magic drops them.
fn cycle_edges(sources: &MagSources) -> HashSet<(usize, usize)> {
    let index: HashMap<&str, usize> = sources.files.iter().enumerate().map(|(i, f)| (f.name.as_str(), i)).collect();
    let mut state = vec![0u8; sources.files.len()]; // 0 new, 1 on stack, 2 done
    let mut back = HashSet::new();
    for root in 0..sources.files.len() {
        if state[root] != 0 {
            continue;
        }
        // Iterative DFS: (file, next use to look at).
        let mut stack = vec![(root, 0usize)];
        state[root] = 1;
        while let Some(&mut (f, ref mut k)) = stack.last_mut() {
            let uses = &sources.files[f].cell.uses;
            if *k >= uses.len() {
                state[f] = 2;
                stack.pop();
                continue;
            }
            let u = *k;
            *k += 1;
            if let Some(&c) = index.get(uses[u].cell.as_str()) {
                match state[c] {
                    0 => {
                        state[c] = 1;
                        stack.push((c, 0));
                    }
                    1 => {
                        back.insert((f, u));
                    }
                    _ => {}
                }
            }
        }
    }
    back
}

fn build(sources: &MagSources, opts: &MagOptions) -> (Library, MagInfo) {
    let mut info = MagInfo { missing: sources.missing.clone(), warnings: sources.warnings.clone(), ..Default::default() };

    // Common grid: every file unit is a whole number of grid units.
    let scales: Vec<Scale> = sources.files.iter().map(|f| f.cell.file_scale().reduced()).collect();
    let common = scales.iter().fold(1u64, |acc, s| lcm(acc, s.d));
    let grid_um = opts.lambda_um / common as f64;
    info.grid_um = grid_um;
    // µm per file unit of each file.
    let um: Vec<f64> = scales.iter().map(|s| (s.n * (common / s.d)) as f64 * grid_um).collect();

    let top = sources.files.first().map_or("magic", |f| f.name.as_str());
    let mut b = LibraryBuilder::new(top, 1e-6, grid_um * 1e-6);
    let mut ids: HashMap<&str, CellId> = HashMap::new();
    for f in &sources.files {
        ids.insert(f.name.as_str(), b.add_cell(&f.name));
    }
    for m in &sources.missing {
        if !ids.contains_key(m.as_str()) {
            ids.insert(m.as_str(), b.add_cell(m));
        }
    }

    let hint: HashSet<&str> = HINT_LAYERS.iter().copied().collect();
    let mut tags: HashMap<String, GdsTag> = HashMap::new();
    let mut tag_of = |name: &str| -> GdsTag {
        if let Some(t) = tags.get(name) {
            return *t;
        }
        let t = match &opts.layer_of {
            Some(f) => f(name),
            None => default_layer_of(name),
        };
        tags.insert(name.to_string(), t);
        t
    };
    let cycles = cycle_edges(sources);

    for (fi, f) in sources.files.iter().enumerate() {
        let id = ids[f.name.as_str()];
        let k = um[fi];
        for layer in &f.cell.layers {
            if !opts.keep_hint_layers && hint.contains(layer.name.as_str()) {
                continue;
            }
            let tag = tag_of(&layer.name);
            for r in &layer.rects {
                b.add_box(id, tag, r[0] as f64 * k, r[1] as f64 * k, r[2] as f64 * k, r[3] as f64 * k);
            }
            for (bx, corner) in &layer.tris {
                let pts = corner.vertices(*bx).map(|(x, y)| Point2D { x: x as f64 * k, y: y as f64 * k });
                b.add_polygon(id, tag, &pts);
            }
        }
        for (ui, u) in f.cell.uses.iter().enumerate() {
            if cycles.contains(&(fi, ui)) {
                info.warnings.push(format!("{}: line {}: use of {} makes a cycle; dropped", f.path, u.line, u.cell));
                continue;
            }
            let Some(&child) = ids.get(u.cell.as_str()) else { continue };
            let t = &u.transform;
            let (rotation, x_reflection) = orientation(t);
            let mut at = Placement {
                origin: Point2D { x: t[2] as f64 * k, y: t[5] as f64 * k },
                rotation,
                x_reflection,
                ..Default::default()
            };
            if let Some(a) = &u.array {
                // Element (i, j) is the child moved by (i·sx, j·sy) in its own
                // coordinates, then transformed: in the parent that is L·(sx, 0)
                // and L·(0, sy), with L the linear part of the transform.
                let sx = a.x_step() as f64 * k;
                let sy = a.y_step() as f64 * k;
                at.columns = a.columns();
                at.rows = a.rows();
                at.v1 = Point2D { x: t[0] as f64 * sx, y: t[3] as f64 * sx };
                at.v2 = Point2D { x: t[1] as f64 * sy, y: t[4] as f64 * sy };
            }
            b.add_reference(id, child, &at);
        }
        let mut cell_info = CellInfo {
            name: f.name.clone(),
            path: f.path.clone(),
            tech: f.cell.tech.clone(),
            properties: f.cell.properties.clone(),
            ports: Vec::new(),
        };
        for l in &f.cell.labels {
            let r = l.rect.map(|v| v as f64 * k);
            let (origin, anchor) = label_anchor(r, l.pos);
            b.add_label(id, tag_of(&l.layer), &l.text, origin, anchor);
            if let Some(p) = &l.port {
                cell_info.ports.push(PortInfo { name: l.text.clone(), layer: l.layer.clone(), port: p.clone(), rect_um: r });
            }
        }
        info.cells.push(cell_info);
    }

    let mut layers: Vec<(String, GdsTag)> = tags.into_iter().collect();
    layers.sort_by(|a, b| a.0.cmp(&b.0));
    let mut owner: HashMap<(u32, u32), &str> = HashMap::new();
    for (name, tag) in &layers {
        match owner.get(&(tag.layer, tag.datatype)) {
            Some(other) => info.warnings.push(format!(
                "layers {other} and {name} share {}/{}; they are compared as one",
                tag.layer, tag.datatype
            )),
            None => {
                owner.insert((tag.layer, tag.datatype), name);
                b.set_layer_name(*tag, name);
            }
        }
    }
    info.layers = layers;
    (b.build(), info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientations_match_the_matrix() {
        // gdstk: p' = R(θ)·(x, ±y) + origin. Check the 8 orientations against
        // x' = a·x + b·y, y' = d·x + e·y on a point off the axes.
        let (x, y) = (3.0f64, 7.0f64);
        for t in [
            [1, 0, 0, 0, 1, 0],
            [0, 1, 0, -1, 0, 0],
            [-1, 0, 0, 0, -1, 0],
            [0, -1, 0, 1, 0, 0],
            [1, 0, 0, 0, -1, 0],
            [-1, 0, 0, 0, 1, 0],
            [0, 1, 0, 1, 0, 0],
            [0, -1, 0, -1, 0, 0],
        ] {
            assert!(parse::is_manhattan(&t));
            let (rot, refl) = orientation(&t);
            let yy = if refl { -y } else { y };
            let (px, py) = (x * rot.cos() - yy * rot.sin(), x * rot.sin() + yy * rot.cos());
            let want = ((t[0] as f64) * x + (t[1] as f64) * y, (t[3] as f64) * x + (t[4] as f64) * y);
            assert!((px - want.0).abs() < 1e-12 && (py - want.1).abs() < 1e-12, "{t:?}");
        }
    }

    #[test]
    fn default_layer_numbers_are_stable_and_distinct() {
        assert_eq!(default_layer_of("metal1"), default_layer_of("metal1"));
        assert_ne!(default_layer_of("metal1"), default_layer_of("metal2"));
        assert!(default_layer_of("poly").layer >= 1 << 30);
    }

    #[test]
    fn cycles_are_found() {
        let file = |name: &str, uses: &[&str]| MagFile {
            name: name.into(),
            path: format!("{name}.mag"),
            bytes: vec![],
            cell: MagCell {
                uses: uses
                    .iter()
                    .map(|c| Use { cell: c.to_string(), id: "i".into(), dir: None, array: None, transform: [1, 0, 0, 0, 1, 0], line: 1 })
                    .collect(),
                ..Default::default()
            },
        };
        let s = MagSources { files: vec![file("a", &["b"]), file("b", &["c", "a"]), file("c", &[])], ..Default::default() };
        assert_eq!(cycle_edges(&s), HashSet::from([(1, 1)]));
    }

    #[test]
    fn labels_anchor_like_klayout() {
        let r = [0.0, 0.0, 2.0, 4.0];
        assert_eq!(label_anchor(r, 0), (Point2D { x: 1.0, y: 2.0 }, Anchor::O));
        assert_eq!(label_anchor(r, 2), (Point2D { x: 2.0, y: 4.0 }, Anchor::SW));
        assert_eq!(label_anchor(r, 6), (Point2D { x: 0.0, y: 0.0 }, Anchor::NE));
    }
}
