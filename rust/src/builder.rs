//! Build a [`Library`] from Rust: cells, polygons, references, labels and
//! layer names. Used by the Magic reader; any other reader written in Rust
//! can use it too.
//!
//! Everything is collected on the Rust side and crosses to C++ once, in
//! [`LibraryBuilder::build`], so building a library with millions of boxes
//! costs one FFI call.

use crate::{ffi, Anchor, GdsTag, Library, Point2D};

/// Index of a cell added to a [`LibraryBuilder`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId(pub usize);

/// Where a reference places its cell: gdstk's order is x-reflection, then
/// rotation, then translation to `origin`. With `columns × rows` above 1,
/// copies are repeated along `v1` (columns) and `v2` (rows), in the
/// coordinates of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub origin: Point2D,
    /// Radians.
    pub rotation: f64,
    pub x_reflection: bool,
    pub columns: u64,
    pub rows: u64,
    pub v1: Point2D,
    pub v2: Point2D,
}

impl Default for Placement {
    fn default() -> Self {
        let zero = Point2D { x: 0.0, y: 0.0 };
        Self { origin: zero, rotation: 0.0, x_reflection: false, columns: 1, rows: 1, v1: zero, v2: zero }
    }
}

/// Collects a library and builds it with [`LibraryBuilder::build`].
pub struct LibraryBuilder {
    name: String,
    unit: f64,
    precision: f64,
    cells: Vec<String>,
    polys: Vec<ffi::PolyPart>,
    xy: Vec<f64>,
    refs: Vec<ffi::RefPart>,
    labels: Vec<ffi::LabelPart>,
    texts: Vec<String>,
    name_tags: Vec<GdsTag>,
    names: Vec<String>,
}

impl LibraryBuilder {
    /// `unit` and `precision` in meters, as in [`Library::unit`] (1e-6 means
    /// coordinates in µm).
    pub fn new(name: &str, unit: f64, precision: f64) -> Self {
        Self {
            name: name.to_string(),
            unit,
            precision,
            cells: Vec::new(),
            polys: Vec::new(),
            xy: Vec::new(),
            refs: Vec::new(),
            labels: Vec::new(),
            texts: Vec::new(),
            name_tags: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn add_cell(&mut self, name: &str) -> CellId {
        self.cells.push(name.to_string());
        CellId(self.cells.len() - 1)
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// A polygon, given by its vertices (not closed: the last point is not
    /// the first one repeated).
    pub fn add_polygon(&mut self, cell: CellId, tag: GdsTag, points: &[Point2D]) {
        self.polys.push(ffi::PolyPart {
            cell: cell.0 as u64,
            layer: tag.layer,
            datatype: tag.datatype,
            points: points.len() as u64,
        });
        self.xy.extend(points.iter().flat_map(|p| [p.x, p.y]));
    }

    /// Axis-aligned box from `(x0, y0)` to `(x1, y1)`.
    pub fn add_box(&mut self, cell: CellId, tag: GdsTag, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.polys.push(ffi::PolyPart { cell: cell.0 as u64, layer: tag.layer, datatype: tag.datatype, points: 4 });
        self.xy.extend([x0, y0, x1, y0, x1, y1, x0, y1]);
    }

    /// An instance of `child` inside `cell`.
    pub fn add_reference(&mut self, cell: CellId, child: CellId, at: &Placement) {
        self.refs.push(ffi::RefPart {
            cell: cell.0 as u64,
            child: child.0 as u64,
            origin: at.origin,
            rotation: at.rotation,
            x_reflection: at.x_reflection,
            columns: at.columns.max(1),
            rows: at.rows.max(1),
            v1: at.v1,
            v2: at.v2,
        });
    }

    pub fn add_label(&mut self, cell: CellId, tag: GdsTag, text: &str, origin: Point2D, anchor: Anchor) {
        self.labels.push(ffi::LabelPart {
            cell: cell.0 as u64,
            layer: tag.layer,
            texttype: tag.datatype,
            origin,
            anchor: anchor as u8,
        });
        self.texts.push(text.to_string());
    }

    /// Name of a layer (see [`Library::layer_names`]).
    pub fn set_layer_name(&mut self, tag: GdsTag, name: &str) {
        self.name_tags.push(tag);
        self.names.push(name.to_string());
    }

    pub fn build(self) -> Library {
        let inner = ffi::library_from_parts(
            &self.name,
            self.unit,
            self.precision,
            &self.cells,
            &self.polys,
            &self.xy,
            &self.refs,
            &self.labels,
            &self.texts,
            &self.name_tags,
            &self.names,
        );
        Library { inner, warning: None }
    }
}
