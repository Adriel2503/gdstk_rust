//! Rust bindings for gdstk — Fase 2.
//!
//! Exposes reading of GDSII files plus iteration over cells and polygons
//! with per-polygon accessors (area, layer, datatype, bbox). This is the
//! foundation for Miku's glayout structured diff.

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

    unsafe extern "C++" {
        include!("gdstk-rs/src/shims.h");

        type LibraryHandle;
        type CellHandle;
        type PolygonHandle;

        // Library
        fn read_gds_shim(filename: &str) -> UniquePtr<LibraryHandle>;
        fn library_cell_count(handle: &LibraryHandle) -> u64;
        fn library_cell_at(handle: &LibraryHandle, idx: u64) -> &CellHandle;

        // Cell
        fn cell_name(cell: &CellHandle) -> &str;
        fn cell_polygon_count(cell: &CellHandle) -> u64;
        fn cell_polygon_at(cell: &CellHandle, idx: u64) -> &PolygonHandle;

        // Polygon
        fn polygon_area(poly: &PolygonHandle) -> f64;
        fn polygon_layer(poly: &PolygonHandle) -> u32;
        fn polygon_datatype(poly: &PolygonHandle) -> u32;
        fn polygon_bbox(poly: &PolygonHandle) -> BoundingBox;
        fn polygon_point_count(poly: &PolygonHandle) -> u64;
    }
}

pub use ffi::BoundingBox;

// ---- Ergonomic wrappers ----

/// Owned handle to a parsed GDSII library.
pub struct Library {
    inner: cxx::UniquePtr<ffi::LibraryHandle>,
}

impl Library {
    /// Open and parse a GDSII file.
    ///
    /// In Fase 2 errors are swallowed — a corrupt/missing file returns a
    /// library with zero cells. Error propagation is future work.
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

// ---- Back-compat with Fase 1 examples ----
// Previous examples used `gdstk_rs::read_gds(path)` + `gdstk_rs::cell_count(&lib)`.
// We keep these thin wrappers so the old examples still compile.

pub fn read_gds(path: &str) -> cxx::UniquePtr<ffi::LibraryHandle> {
    ffi::read_gds_shim(path)
}

pub fn cell_count(handle: &ffi::LibraryHandle) -> u64 {
    ffi::library_cell_count(handle)
}
