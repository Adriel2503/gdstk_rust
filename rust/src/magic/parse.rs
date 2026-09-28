//! Parser of one Magic `.mag` file (one cell), without FFI.
//!
//! The grammar follows Magic 8.3 (`database/DBio.c`, the owner of the
//! format). Where Magic rejects a file for something harmless (a blank line
//! between sections, a missing `tech` before `magscale`), this reader accepts
//! it and records a warning, so one odd file does not stop a whole diff.

use std::collections::HashMap;

/// File units per lambda: every coordinate of the file is `value * n / d`
/// lambda (`magscale n d`; 1/1 when the line is missing).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    pub n: u64,
    pub d: u64,
}

impl Scale {
    pub const ONE: Scale = Scale { n: 1, d: 1 };

    /// Same fraction with no common factors.
    pub fn reduced(self) -> Scale {
        let g = gcd(self.n, self.d);
        Scale { n: self.n / g, d: self.d / g }
    }
}

pub(crate) fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// Corner of the right angle of a `tri` (the suffix in the file).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Corner {
    Nw,
    Sw,
    Se,
    Ne,
}

impl Corner {
    /// Vertices of the triangle inside the box `[xlo, ylo, xhi, yhi]`.
    pub fn vertices(self, b: [i64; 4]) -> [(i64, i64); 3] {
        let [xl, yl, xh, yh] = b;
        match self {
            Corner::Nw => [(xl, yl), (xl, yh), (xh, yh)],
            Corner::Sw => [(xl, yl), (xl, yh), (xh, yl)],
            Corner::Se => [(xl, yl), (xh, yl), (xh, yh)],
            Corner::Ne => [(xl, yh), (xh, yh), (xh, yl)],
        }
    }

    /// Magic reads the direction from the letters `s` and `e` anywhere in
    /// the rest of the line (`DBio.c`, GetRect): `se`, `sw` (s), `ne` (e),
    /// `nw` (neither).
    fn from_rest(rest: &str) -> Corner {
        match (rest.contains('s'), rest.contains('e')) {
            (true, true) => Corner::Se,
            (true, false) => Corner::Sw,
            (false, true) => Corner::Ne,
            (false, false) => Corner::Nw,
        }
    }
}

/// Paint of one layer: axis-aligned boxes and right triangles, in file units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayerPaint {
    pub name: String,
    /// `[xlo, ylo, xhi, yhi]`, normalized, never empty.
    pub rects: Vec<[i64; 4]>,
    pub tris: Vec<([i64; 4], Corner)>,
}

/// Most elements one `array` may have. Flattening makes a copy of the child
/// per element, so a typo (`array 0 999999999 …`) would exhaust memory; real
/// arrays (an SRAM bit array) stay far below this.
pub const MAX_ARRAY_ELEMENTS: u64 = 100_000_000;

/// `array xlo xhi xsep ylo yhi ysep`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArraySpec {
    pub xlo: i64,
    pub xhi: i64,
    pub xsep: i64,
    pub ylo: i64,
    pub yhi: i64,
    pub ysep: i64,
}

impl ArraySpec {
    pub fn columns(&self) -> u64 {
        self.xhi.abs_diff(self.xlo).saturating_add(1)
    }
    pub fn rows(&self) -> u64 {
        self.yhi.abs_diff(self.ylo).saturating_add(1)
    }
    /// Step between columns, in file units of the parent. Magic walks from
    /// `xlo` to `xhi`, so a reversed range steps backwards (DBcellbox.c).
    pub fn x_step(&self) -> i64 {
        if self.xlo > self.xhi { -self.xsep } else { self.xsep }
    }
    pub fn y_step(&self) -> i64 {
        if self.ylo > self.yhi { -self.ysep } else { self.ysep }
    }
}

/// One instance: `use cell id [dir]` + `array` + `transform` + `box`.
#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    pub cell: String,
    /// Instance id (`cell_0` when the file has none). A leading `*` (locked
    /// instance) is removed.
    pub id: String,
    /// Directory of the child's file, as written (may start with `$VAR` or
    /// `~`). A `use` without it inherits the one of an earlier `use` of the
    /// same cell in this file.
    pub dir: Option<String>,
    pub array: Option<ArraySpec>,
    /// `a b c d e f`: `x' = a·x + b·y + c`, `y' = d·x + e·y + f`. `c` and `f`
    /// are in file units of the parent; `a b d e` are one of the eight
    /// Manhattan orientations.
    pub transform: [i64; 6],
    pub line: usize,
}

/// `port idx nsew [use class [shape]]`, attached to the label before it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Port {
    pub index: i64,
    /// Sides the port can connect from (`nsew`).
    pub sides: String,
    /// signal, analog, power, ground, clock, default.
    pub usage: Option<String>,
    /// input, output, tristate, bidirectional, inout, feedthrough, default.
    pub class: Option<String>,
    /// abutment, ring, feedthrough, default.
    pub shape: Option<String>,
}

/// `rlabel` / `flabel` (and the old `label`).
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub layer: String,
    pub text: String,
    /// `[xlo, ylo, xhi, yhi]` in file units; may be a point.
    pub rect: [i64; 4],
    /// Justification 0–8: center, n, ne, e, se, s, sw, w, nw.
    pub pos: u8,
    pub sticky: bool,
    pub port: Option<Port>,
}

/// One `.mag` file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MagCell {
    pub tech: Option<String>,
    pub scale: Option<Scale>,
    pub timestamp: Option<i64>,
    /// Paint by layer, in the order of the file; a layer written in two
    /// sections is merged.
    pub layers: Vec<LayerPaint>,
    pub uses: Vec<Use>,
    pub labels: Vec<Label>,
    /// `<< properties >>`: key and raw value (`string GDS_FILE path`).
    pub properties: Vec<(String, String)>,
    /// Harmless oddities, with their line number.
    pub warnings: Vec<String>,
}

impl MagCell {
    /// `magscale` of the file, or 1/1.
    pub fn file_scale(&self) -> Scale {
        self.scale.unwrap_or(Scale::ONE)
    }

    /// Value of a property.
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
}

/// A `.mag` that cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// `true` if `data` starts like a `.mag` file (first line `magic`).
pub fn is_mag(data: &[u8]) -> bool {
    let data = data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data);
    data.starts_with(b"magic")
        && matches!(data.get(5), None | Some(b'\n' | b'\r' | b' ' | b'\t'))
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    None,
    Paint(usize),
    Labels,
    Properties,
    Elements,
    End,
}

/// Parse one `.mag` file.
pub fn parse(data: &[u8]) -> Result<MagCell, ParseError> {
    if !is_mag(data) {
        return Err(ParseError { line: 1, message: "not a Magic file (first line is not `magic`)".into() });
    }
    let text = String::from_utf8_lossy(data);
    let mut p = Parser {
        cell: MagCell::default(),
        section: Section::None,
        layer_index: HashMap::new(),
        use_dirs: HashMap::new(),
        use_count: HashMap::new(),
        blank_before_section: false,
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 1; // line 0 is `magic`
    while i < lines.len() {
        let no = i + 1;
        let line = lines[i].trim_end_matches('\r');
        i += 1;
        if p.section == Section::End {
            break;
        }
        let t = line.trim();
        if t.is_empty() {
            p.blank_before_section = true;
            continue;
        }
        if t.starts_with('#') {
            continue;
        }
        if let Some(name) = section_name(t) {
            p.open_section(name, no);
            continue;
        }
        let first = t.split_whitespace().next().unwrap_or("");
        if first == "use" {
            // The group runs up to `box`, or up to the first line that is not
            // part of it.
            let mut group = vec![(no, t)];
            while i < lines.len() {
                let g = lines[i].trim();
                let k = g.split_whitespace().next().unwrap_or("");
                if !matches!(k, "array" | "timestamp" | "transform" | "box") {
                    break;
                }
                group.push((i + 1, g));
                i += 1;
                if k == "box" {
                    break;
                }
            }
            p.section = Section::None;
            p.read_use(&group)?;
            continue;
        }
        match p.section {
            Section::Paint(idx) => p.read_paint(idx, first, t, no)?,
            Section::Labels => p.read_label(first, t, no)?,
            Section::Properties => p.read_property(t, no),
            Section::Elements | Section::End => {}
            Section::None => p.read_header(first, t, no)?,
        }
    }
    Ok(p.cell)
}

fn section_name(t: &str) -> Option<&str> {
    let inner = t.strip_prefix("<<")?.strip_suffix(">>")?.trim();
    (!inner.is_empty() && !inner.contains(char::is_whitespace)).then_some(inner)
}

struct Parser {
    cell: MagCell,
    section: Section,
    layer_index: HashMap<String, usize>,
    /// Directory of the first `use` of each cell that gave one.
    use_dirs: HashMap<String, String>,
    /// Instances without id, per cell, to name them like Magic (`cell_N`).
    use_count: HashMap<String, u64>,
    blank_before_section: bool,
}

impl Parser {
    fn warn(&mut self, line: usize, msg: impl std::fmt::Display) {
        self.cell.warnings.push(format!("line {line}: {msg}"));
    }

    fn open_section(&mut self, name: &str, line: usize) {
        if std::mem::take(&mut self.blank_before_section) {
            self.warn(line, "blank line before a section (Magic would reject the file)");
        }
        self.section = match name {
            "labels" => Section::Labels,
            "properties" => Section::Properties,
            "elements" => Section::Elements,
            "end" => Section::End,
            layer => {
                let next = self.cell.layers.len();
                let idx = *self.layer_index.entry(layer.to_string()).or_insert(next);
                if idx == next {
                    self.cell.layers.push(LayerPaint { name: layer.to_string(), ..Default::default() });
                }
                Section::Paint(idx)
            }
        };
    }

    fn read_header(&mut self, first: &str, t: &str, line: usize) -> Result<(), ParseError> {
        let args: Vec<&str> = t.split_whitespace().skip(1).collect();
        match first {
            "tech" => self.cell.tech = args.first().map(|s| s.to_string()),
            "magscale" => {
                let n = args.first().and_then(|s| s.parse::<u64>().ok());
                let d = args.get(1).and_then(|s| s.parse::<u64>().ok());
                match (n, d) {
                    (Some(n), Some(d)) if n > 0 && d > 0 => {
                        if self.cell.tech.is_none() {
                            self.warn(line, "magscale without a tech line");
                        }
                        self.cell.scale = Some(Scale { n, d });
                    }
                    _ => return Err(err(line, format!("bad magscale: `{t}`"))),
                }
            }
            "timestamp" => self.cell.timestamp = args.first().and_then(|s| s.parse().ok()),
            // Old, ignored by Magic too.
            "maxlabscale" => {}
            _ => self.warn(line, format!("unknown line ignored: `{}`", short(t))),
        }
        Ok(())
    }

    fn read_paint(&mut self, idx: usize, first: &str, t: &str, line: usize) -> Result<(), ParseError> {
        let (is_tri, rest) = match first {
            "rect" => (false, &t[4..]),
            "tri" => (true, &t[3..]),
            _ => {
                self.warn(line, format!("unknown line in a paint section: `{}`", short(t)));
                return Ok(());
            }
        };
        let mut it = rest.split_whitespace();
        let mut v = [0i64; 4];
        for slot in &mut v {
            *slot = coord(it.next(), line, t)?;
        }
        let b = [v[0].min(v[2]), v[1].min(v[3]), v[0].max(v[2]), v[1].max(v[3])];
        if b[0] == b[2] || b[1] == b[3] {
            return Ok(()); // no area: Magic drops it too
        }
        let layer = &mut self.cell.layers[idx];
        if is_tri {
            let tail: String = it.collect::<Vec<_>>().join(" ");
            layer.tris.push((b, Corner::from_rest(&tail)));
        } else {
            layer.rects.push(b);
        }
        Ok(())
    }

    fn read_label(&mut self, first: &str, t: &str, line: usize) -> Result<(), ParseError> {
        match first {
            "rlabel" | "flabel" => {
                let fl = first == "flabel";
                let (head, _) = split_tokens(t, 3);
                let sticky = head.get(2) == Some(&"s");
                // rlabel layer [s] x1 y1 x2 y2 pos text
                // flabel layer [s] x1 y1 x2 y2 pos font size rot offx offy text
                let fixed = 2 + usize::from(sticky) + 5 + if fl { 5 } else { 0 };
                let (tok, text) = split_tokens(t, fixed);
                if tok.len() < fixed {
                    return Err(err(line, format!("short {first}: `{}`", short(t))));
                }
                let base = 2 + usize::from(sticky);
                let mut r = [0i64; 4];
                for (k, slot) in r.iter_mut().enumerate() {
                    *slot = coord(Some(tok[base + k]), line, t)?;
                }
                let pos = tok[base + 4].parse::<u8>().unwrap_or(0).min(8);
                self.cell.labels.push(Label {
                    layer: tok[1].to_string(),
                    text: text.to_string(),
                    rect: [r[0].min(r[2]), r[1].min(r[3]), r[0].max(r[2]), r[1].max(r[3])],
                    pos,
                    sticky,
                    port: None,
                });
            }
            "label" => {
                // Old form: label layer x y pos text
                let (tok, text) = split_tokens(t, 5);
                if tok.len() < 5 {
                    return Err(err(line, format!("short label: `{}`", short(t))));
                }
                let x = coord(Some(tok[2]), line, t)?;
                let y = coord(Some(tok[3]), line, t)?;
                self.cell.labels.push(Label {
                    layer: tok[1].to_string(),
                    text: text.to_string(),
                    rect: [x, y, x, y],
                    pos: tok[4].parse::<u8>().unwrap_or(0).min(8),
                    sticky: false,
                    port: None,
                });
            }
            "port" => {
                let tok: Vec<&str> = t.split_whitespace().collect();
                let port = Port {
                    index: tok.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                    sides: tok.get(2).map(|s| s.to_string()).unwrap_or_default(),
                    usage: tok.get(3).map(|s| s.to_string()),
                    class: tok.get(4).map(|s| s.to_string()),
                    shape: tok.get(5).map(|s| s.to_string()),
                };
                match self.cell.labels.last_mut() {
                    Some(l) => l.port = Some(port),
                    None => self.warn(line, "port without a label before it"),
                }
            }
            _ => self.warn(line, format!("unknown line in labels: `{}`", short(t))),
        }
        Ok(())
    }

    fn read_property(&mut self, t: &str, line: usize) {
        let (tok, value) = split_tokens(t, 2);
        let is_type = |s: &str| {
            ["string", "integer", "dimension", "double"].iter().any(|k| !s.is_empty() && k.starts_with(s))
        };
        match tok.as_slice() {
            [ty, key] if is_type(ty) => self.cell.properties.push((key.to_string(), value.to_string())),
            _ => self.warn(line, format!("unknown property line: `{}`", short(t))),
        }
    }

    fn read_use(&mut self, group: &[(usize, &str)]) -> Result<(), ParseError> {
        let (line, head) = group[0];
        let tok: Vec<&str> = head.split_whitespace().collect();
        let Some(cell) = tok.get(1).map(|s| s.to_string()) else {
            return Err(err(line, "use without a cell name"));
        };
        let id = match tok.get(2) {
            Some(id) => id.trim_start_matches('*').to_string(),
            None => {
                let n = self.use_count.entry(cell.clone()).or_insert(0);
                let id = format!("{cell}_{n}");
                *n += 1;
                id
            }
        };
        let dir = match tok.get(3) {
            Some(d) => {
                self.use_dirs.entry(cell.clone()).or_insert_with(|| d.to_string());
                Some(d.to_string())
            }
            None => self.use_dirs.get(&cell).cloned(),
        };
        let mut array = None;
        let mut transform = None;
        for &(no, g) in &group[1..] {
            let mut it = g.split_whitespace();
            let key = it.next().unwrap_or("");
            let nums: Vec<&str> = it.collect();
            let ints = |count: usize| -> Result<Vec<i64>, ParseError> {
                if nums.len() < count {
                    return Err(err(no, format!("short {key}: `{}`", short(g))));
                }
                nums[..count].iter().map(|s| coord(Some(s), no, g)).collect()
            };
            match key {
                "array" => {
                    let v = ints(6)?;
                    let a = ArraySpec { xlo: v[0], xhi: v[1], xsep: v[2], ylo: v[3], yhi: v[4], ysep: v[5] };
                    if a.columns().checked_mul(a.rows()).is_none_or(|n| n > MAX_ARRAY_ELEMENTS) {
                        return Err(err(
                            no,
                            format!("array of {} × {} elements is over the limit ({MAX_ARRAY_ELEMENTS})", a.columns(), a.rows()),
                        ));
                    }
                    array = Some(a);
                }
                "transform" => {
                    let v = ints(6)?;
                    let m = [v[0], v[1], v[2], v[3], v[4], v[5]];
                    if !is_manhattan(&m) {
                        return Err(err(no, format!("transform is not one of the 8 Manhattan orientations: `{g}`")));
                    }
                    transform = Some(m);
                }
                _ => {} // timestamp, box: not geometry
            }
        }
        let transform = transform.unwrap_or_else(|| {
            self.warn(line, format!("use {cell} without transform: identity"));
            [1, 0, 0, 0, 1, 0]
        });
        self.cell.uses.push(Use { cell, id, dir, array, transform, line });
        Ok(())
    }
}

/// `a b d e` of a transform is a rotation by a multiple of 90°, maybe
/// mirrored (the only ones Magic accepts, DBio.c).
pub(crate) fn is_manhattan(t: &[i64; 6]) -> bool {
    let [a, b, _, d, e, _] = *t;
    (a == 0 && e == 0 && b.abs() == 1 && d.abs() == 1) || (b == 0 && d == 0 && a.abs() == 1 && e.abs() == 1)
}

/// Whitespace tokens: the first `n` and the rest of the line after them
/// (trimmed), for texts with spaces.
fn split_tokens(t: &str, n: usize) -> (Vec<&str>, &str) {
    let mut out = Vec::with_capacity(n);
    let mut rest = t.trim_start();
    while out.len() < n && !rest.is_empty() {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = rest[end..].trim_start();
    }
    (out, rest.trim_end())
}

/// Integer coordinate. Magic reads integers; a fractional value (seen in old
/// label lines) is rounded.
fn coord(s: Option<&str>, line: usize, t: &str) -> Result<i64, ParseError> {
    let s = s.ok_or_else(|| err(line, format!("missing coordinate: `{}`", short(t))))?;
    if let Ok(v) = s.parse::<i64>() {
        return Ok(v);
    }
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|v| v.round() as i64)
        .ok_or_else(|| err(line, format!("bad number `{s}`: `{}`", short(t))))
}

fn err(line: usize, message: impl Into<String>) -> ParseError {
    ParseError { line, message: message.into() }
}

fn short(t: &str) -> &str {
    match t.char_indices().nth(80) {
        Some((i, _)) => &t[..i],
        None => t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INV: &str = "magic
tech sky130A
magscale 1 2
timestamp 1700000000
<< nwell >>
rect -38 261 314 582
<< metal1 >>
rect 0 496 276 592
rect 276 0 0 96
tri 10 10 20 20 se
tri 10 10 20 20 nw
<< labels >>
rlabel locali s 64 215 130 263 6 A
port 1 nsew signal input
flabel metal1 s 0 51 960 125 0 FreeSans 340 0 0 0 VGND bus
<< properties >>
string FIXED_BBOX 0 0 276 544
string GDS_FILE $PDKPATH/libs.ref/x.gds
<< end >>
rect 9 9 99 99
";

    #[test]
    fn reads_a_cell() {
        let c = parse(INV.as_bytes()).unwrap();
        assert_eq!(c.tech.as_deref(), Some("sky130A"));
        assert_eq!(c.scale, Some(Scale { n: 1, d: 2 }));
        assert_eq!(c.timestamp, Some(1700000000));
        assert_eq!(c.layers.len(), 2);
        assert_eq!(c.layers[1].name, "metal1");
        // Swapped corners are normalized.
        assert_eq!(c.layers[1].rects, vec![[0, 496, 276, 592], [0, 0, 276, 96]]);
        assert_eq!(c.layers[1].tris, vec![([10, 10, 20, 20], Corner::Se), ([10, 10, 20, 20], Corner::Nw)]);
        assert_eq!(c.labels.len(), 2);
        let a = &c.labels[0];
        assert_eq!((a.layer.as_str(), a.text.as_str(), a.pos, a.sticky), ("locali", "A", 6, true));
        let port = a.port.as_ref().unwrap();
        assert_eq!((port.index, port.usage.as_deref(), port.class.as_deref()), (1, Some("signal"), Some("input")));
        // The text of a flabel is the rest of the line, spaces included.
        assert_eq!(c.labels[1].text, "VGND bus");
        assert_eq!(c.property("GDS_FILE"), Some("$PDKPATH/libs.ref/x.gds"));
        // `<< end >>` stops reading.
        assert_eq!(c.layers[1].rects.len(), 2);
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
    }

    #[test]
    fn triangle_directions_follow_magic() {
        let b = [0, 0, 10, 20];
        assert_eq!(Corner::Nw.vertices(b), [(0, 0), (0, 20), (10, 20)]);
        assert_eq!(Corner::Sw.vertices(b), [(0, 0), (0, 20), (10, 0)]);
        assert_eq!(Corner::Se.vertices(b), [(0, 0), (10, 0), (10, 20)]);
        assert_eq!(Corner::Ne.vertices(b), [(0, 20), (10, 20), (10, 0)]);
        // Flags scanned like Magic: `s`/`e` anywhere after the numbers.
        assert_eq!(Corner::from_rest("e"), Corner::Ne);
        assert_eq!(Corner::from_rest("s"), Corner::Sw);
        assert_eq!(Corner::from_rest(""), Corner::Nw);
    }

    #[test]
    fn reads_uses_with_arrays_dirs_and_generated_ids() {
        let f = "magic
tech sky130A
use inv  inv_0 $PDKPATH/libs.ref/sc/mag
timestamp 1
transform 1 0 10 0 1 20
box 0 0 10 10
use inv  *inv_1
array 0 3 50 2 0 40
transform 0 1 0 -1 0 5
box 0 0 10 10
use nand
transform -1 0 0 0 -1 0
box 0 0 1 1
use nand
transform 1 0 0 0 -1 0
box 0 0 1 1
";
        let c = parse(f.as_bytes()).unwrap();
        assert_eq!(c.uses.len(), 4);
        let u = &c.uses[0];
        assert_eq!((u.cell.as_str(), u.id.as_str(), u.dir.as_deref()), ("inv", "inv_0", Some("$PDKPATH/libs.ref/sc/mag")));
        assert_eq!(u.transform, [1, 0, 10, 0, 1, 20]);
        // Locked id loses its `*`; the directory is inherited.
        let u = &c.uses[1];
        assert_eq!((u.id.as_str(), u.dir.as_deref()), ("inv_1", Some("$PDKPATH/libs.ref/sc/mag")));
        let a = u.array.unwrap();
        assert_eq!((a.columns(), a.rows(), a.x_step(), a.y_step()), (4, 3, 50, -40));
        assert_eq!((c.uses[2].id.as_str(), c.uses[3].id.as_str()), ("nand_0", "nand_1"));
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
    }

    #[test]
    fn huge_arrays_are_an_error_not_an_allocation() {
        let use_with = |array: &str| format!("magic\nuse a\n{array}\ntransform 1 0 0 0 1 0\nbox 0 0 1 1\n");
        let e = parse(use_with("array 0 999999999 10 0 999999999 10").as_bytes()).unwrap_err();
        assert_eq!(e.line, 3);
        assert!(parse(use_with(&format!("array {} {} 1 0 0 1", i64::MIN, i64::MAX)).as_bytes()).is_err(), "overflow");
        let c = parse(use_with("array 0 9999 1 0 9999 1").as_bytes()).unwrap();
        let a = c.uses[0].array.unwrap();
        assert_eq!(a.columns() * a.rows(), MAX_ARRAY_ELEMENTS, "the limit itself is allowed");
    }

    #[test]
    fn rejects_what_magic_rejects_and_tolerates_the_rest() {
        assert!(parse(b"gds\n").is_err());
        let e = parse(b"magic\nuse a\ntransform 1 1 0 0 1 0\nbox 0 0 1 1\n").unwrap_err();
        assert_eq!(e.line, 3);
        assert!(parse(b"magic\n<< m1 >>\nrect 0 0 x 1\n").is_err());
        let c = parse(b"magic\nmagscale 1 2\n<< m1 >>\nrect 0 0 1 1\n\n<< m2 >>\nrect 0 0 0 5\nbogus\n").unwrap();
        assert_eq!(c.layers.len(), 2);
        assert!(c.layers[1].rects.is_empty(), "zero-area rect dropped");
        assert_eq!(c.warnings.len(), 3, "{:?}", c.warnings);
        // CRLF and comments.
        let c = parse(b"magic\r\n# note\r\ntech t\r\n<< m1 >>\r\nrect 0 0 2 2\r\n<< m1 >>\r\nrect 5 5 6 6\r\n").unwrap();
        assert_eq!(c.layers.len(), 1, "same layer in two sections is merged");
        assert_eq!(c.layers[0].rects.len(), 2);
    }

    #[test]
    fn old_forms() {
        let c = parse(b"magic\n<< labels >>\nlabel metal1 5 6 0 OUT\nrlabel poly 1 2 3 4 1 IN\n").unwrap();
        assert_eq!(c.tech, None);
        assert_eq!(c.labels[0].rect, [5, 6, 5, 6]);
        assert_eq!(c.labels[1].text, "IN");
        assert!(!c.labels[1].sticky);
    }
}
