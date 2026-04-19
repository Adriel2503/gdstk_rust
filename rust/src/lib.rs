//! Rust bindings for gdstk — Fase 4.
//!
//! Exposes reading of GDSII files plus iteration over cells, polygons, labels,
//! and references, with per-item accessors and boolean XOR for computing the
//! geometric diff between two cells. This is the core of Miku's structured
//! diff: for each commit pair, XOR per layer reveals what changed physically.

use std::borrow::Cow;

#[cxx::bridge(namespace = "gdstk_shim")]
mod ffi {
    /// Axis-aligned bounding box in layout units (µm × unit).
    /// Shared POD struct — same layout in Rust and C++.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct BoundingBox {
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    }

    /// 2D point in layout units. Shared POD struct.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Point2D {
        x: f64,
        y: f64,
    }

    /// Aggregate metrics from a boolean XOR operation between two cells.
    /// `bbox` is all-zeros when `region_count == 0`.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct XorMetrics {
        area: f64,
        region_count: u64,
        bbox: BoundingBox,
    }

    unsafe extern "C++" {
        include!("gdstk-rs/src/shims.h");

        type LibraryHandle;
        type CellHandle;
        type PolygonHandle;
        type LabelHandle;
        type ReferenceHandle;

        // Library
        fn read_gds_shim(filename: &str) -> UniquePtr<LibraryHandle>;
        fn library_cell_count(handle: &LibraryHandle) -> u64;
        fn library_cell_at(handle: &LibraryHandle, idx: u64) -> &CellHandle;

        // Cell
        fn cell_name(cell: &CellHandle) -> &str;
        fn cell_polygon_count(cell: &CellHandle) -> u64;
        fn cell_polygon_at(cell: &CellHandle, idx: u64) -> &PolygonHandle;
        fn cell_label_count(cell: &CellHandle) -> u64;
        fn cell_label_at(cell: &CellHandle, idx: u64) -> &LabelHandle;
        fn cell_reference_count(cell: &CellHandle) -> u64;
        fn cell_reference_at(cell: &CellHandle, idx: u64) -> &ReferenceHandle;

        // Polygon
        fn polygon_area(poly: &PolygonHandle) -> f64;
        fn polygon_layer(poly: &PolygonHandle) -> u32;
        fn polygon_datatype(poly: &PolygonHandle) -> u32;
        fn polygon_bbox(poly: &PolygonHandle) -> BoundingBox;
        fn polygon_point_count(poly: &PolygonHandle) -> u64;

        // Label
        fn label_text_bytes(label: &LabelHandle) -> &[u8];
        fn label_layer(label: &LabelHandle) -> u32;
        fn label_texttype(label: &LabelHandle) -> u32;
        fn label_origin(label: &LabelHandle) -> Point2D;
        fn label_anchor(label: &LabelHandle) -> u8;
        fn label_rotation(label: &LabelHandle) -> f64;
        fn label_magnification(label: &LabelHandle) -> f64;
        fn label_x_reflection(label: &LabelHandle) -> bool;

        // Reference
        fn reference_cell_name(r: &ReferenceHandle) -> &str;
        fn reference_origin(r: &ReferenceHandle) -> Point2D;
        fn reference_rotation(r: &ReferenceHandle) -> f64;
        fn reference_magnification(r: &ReferenceHandle) -> f64;
        fn reference_x_reflection(r: &ReferenceHandle) -> bool;

        // Boolean XOR
        fn cell_xor_with(a: &CellHandle, b: &CellHandle, layer: u32) -> XorMetrics;
    }
}

pub use ffi::{BoundingBox, Point2D, XorMetrics};

// ---- Ergonomic wrappers ----

/// Owned handle to a parsed GDSII library.
pub struct Library {
    inner: cxx::UniquePtr<ffi::LibraryHandle>,
}

impl Library {
    /// Open and parse a GDSII file.
    ///
    /// Errors are swallowed in current phase — a corrupt/missing file returns
    /// a library with zero cells. Error propagation is future work.
    pub fn open(path: &str) -> Self {
        Self {
            inner: ffi::read_gds_shim(path),
        }
    }

    pub fn cell_count(&self) -> u64 {
        ffi::library_cell_count(&self.inner)
    }

    pub fn cell(&self, idx: u64) -> Cell<'_> {
        Cell {
            handle: ffi::library_cell_at(&self.inner, idx),
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell<'_>> {
        (0..self.cell_count()).map(move |i| self.cell(i))
    }

    /// Linear search for a cell by name. Returns the first match.
    pub fn find_cell(&self, name: &str) -> Option<Cell<'_>> {
        self.cells().find(|c| c.name() == name)
    }
}

/// Borrowed view into a cell of a Library.
///
/// The `'a` lifetime ties the Cell to its Library — it cannot outlive it.
#[derive(Clone, Copy)]
pub struct Cell<'a> {
    handle: &'a ffi::CellHandle,
}

impl<'a> Cell<'a> {
    pub fn name(&self) -> &'a str {
        ffi::cell_name(self.handle)
    }

    pub fn polygon_count(&self) -> u64 {
        ffi::cell_polygon_count(self.handle)
    }

    pub fn polygon(&self, idx: u64) -> Polygon<'a> {
        Polygon {
            handle: ffi::cell_polygon_at(self.handle, idx),
        }
    }

    pub fn polygons(&self) -> impl Iterator<Item = Polygon<'a>> + use<'a> {
        let this = *self;
        (0..this.polygon_count()).map(move |i| this.polygon(i))
    }

    pub fn label_count(&self) -> u64 {
        ffi::cell_label_count(self.handle)
    }

    pub fn label(&self, idx: u64) -> Label<'a> {
        Label {
            handle: ffi::cell_label_at(self.handle, idx),
        }
    }

    pub fn labels(&self) -> impl Iterator<Item = Label<'a>> + use<'a> {
        let this = *self;
        (0..this.label_count()).map(move |i| this.label(i))
    }

    pub fn reference_count(&self) -> u64 {
        ffi::cell_reference_count(self.handle)
    }

    pub fn reference(&self, idx: u64) -> Reference<'a> {
        Reference {
            handle: ffi::cell_reference_at(self.handle, idx),
        }
    }

    pub fn references(&self) -> impl Iterator<Item = Reference<'a>> + use<'a> {
        let this = *self;
        (0..this.reference_count()).map(move |i| this.reference(i))
    }

    /// Geometric XOR between two cells, filtered by layer. Returns area,
    /// region count, and bbox of the symmetric difference. Only considers
    /// polygons in the cells themselves — does NOT flatten references.
    pub fn xor_with(&self, other: &Cell<'_>, layer: u32) -> XorMetrics {
        ffi::cell_xor_with(self.handle, other.handle, layer)
    }
}

/// Borrowed view into a polygon of a Cell.
#[derive(Clone, Copy)]
pub struct Polygon<'a> {
    handle: &'a ffi::PolygonHandle,
}

impl<'a> Polygon<'a> {
    /// Polygon area in layout units² (applies repetition internally).
    pub fn area(&self) -> f64 {
        ffi::polygon_area(self.handle)
    }

    pub fn layer(&self) -> u32 {
        ffi::polygon_layer(self.handle)
    }

    pub fn datatype(&self) -> u32 {
        ffi::polygon_datatype(self.handle)
    }

    pub fn bbox(&self) -> BoundingBox {
        ffi::polygon_bbox(self.handle)
    }

    pub fn point_count(&self) -> u64 {
        ffi::polygon_point_count(self.handle)
    }
}

/// Anchor position of a label's text relative to its origin.
///
/// Values match gdstk's `enum struct Anchor` (sparse numbering).
/// NW=0, N=1, NE=2, W=4, O=5, E=6, SW=8, S=9, SE=10.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    NW = 0,
    N = 1,
    NE = 2,
    W = 4,
    /// Origin / center.
    O = 5,
    E = 6,
    SW = 8,
    S = 9,
    SE = 10,
}

impl Anchor {
    /// Map a u8 (as produced by the shim) to an Anchor. Invalid values
    /// (3, 7, 11+) fall back to `O` (center) — matches what gdstk treats
    /// as "default" anchor for malformed labels.
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::NW,
            1 => Self::N,
            2 => Self::NE,
            4 => Self::W,
            5 => Self::O,
            6 => Self::E,
            8 => Self::SW,
            9 => Self::S,
            10 => Self::SE,
            _ => Self::O,
        }
    }
}

/// Borrowed view into a label of a Cell.
#[derive(Clone, Copy)]
pub struct Label<'a> {
    handle: &'a ffi::LabelHandle,
}

impl<'a> Label<'a> {
    /// Label text.
    ///
    /// Returns `Cow::Borrowed` if the underlying bytes are valid UTF-8 (the
    /// common case for modern GDS files). Returns `Cow::Owned` with
    /// U+FFFD replacement for invalid sequences (legacy GDS with non-UTF-8
    /// encodings).
    pub fn text(&self) -> Cow<'a, str> {
        let bytes: &'a [u8] = ffi::label_text_bytes(self.handle);
        // String::from_utf8_lossy returns Cow<'_, str> with the lifetime of
        // the input slice — matches our 'a lifetime.
        String::from_utf8_lossy(bytes)
    }

    pub fn layer(&self) -> u32 {
        ffi::label_layer(self.handle)
    }

    pub fn texttype(&self) -> u32 {
        ffi::label_texttype(self.handle)
    }

    pub fn origin(&self) -> Point2D {
        ffi::label_origin(self.handle)
    }

    pub fn anchor(&self) -> Anchor {
        Anchor::from_u8(ffi::label_anchor(self.handle))
    }

    pub fn rotation(&self) -> f64 {
        ffi::label_rotation(self.handle)
    }

    pub fn magnification(&self) -> f64 {
        ffi::label_magnification(self.handle)
    }

    pub fn x_reflection(&self) -> bool {
        ffi::label_x_reflection(self.handle)
    }
}

/// Borrowed view into a cell reference (an instance of another cell,
/// placed at `origin` with an optional rotation/magnification/reflection).
#[derive(Clone, Copy)]
pub struct Reference<'a> {
    handle: &'a ffi::ReferenceHandle,
}

impl<'a> Reference<'a> {
    /// Name of the referenced cell. Empty if the reference is unresolved.
    pub fn cell_name(&self) -> &'a str {
        ffi::reference_cell_name(self.handle)
    }

    pub fn origin(&self) -> Point2D {
        ffi::reference_origin(self.handle)
    }

    /// Rotation in radians.
    pub fn rotation(&self) -> f64 {
        ffi::reference_rotation(self.handle)
    }

    pub fn magnification(&self) -> f64 {
        ffi::reference_magnification(self.handle)
    }

    pub fn x_reflection(&self) -> bool {
        ffi::reference_x_reflection(self.handle)
    }
}

// ---- Back-compat with Fase 1 examples ----

pub fn read_gds(path: &str) -> cxx::UniquePtr<ffi::LibraryHandle> {
    ffi::read_gds_shim(path)
}

pub fn cell_count(handle: &ffi::LibraryHandle) -> u64 {
    ffi::library_cell_count(handle)
}
