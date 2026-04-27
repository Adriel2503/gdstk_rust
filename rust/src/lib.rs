//! Rust bindings for gdstk — Fase 5.
//!
//! Full GDS inspection surface for Miku: cells, polygons, labels, references,
//! and paths (FlexPath / RobustPath). Boolean XOR (`cell.xor_with`) now
//! converts paths to polygons before the diff, so GDS files with wire paths
//! (common in real designs) are compared correctly.

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

    /// (layer, datatype) pair — GDSII tag decomposed.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct GdsTag {
        layer: u32,
        datatype: u32,
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
        type FlexPathHandle;
        type RobustPathHandle;
        type TopLevelView;
        type GdsInfoHandle;
        type RawCellHandle;
        type RepetitionHandle;
        type FlattenedPolygonsHandle;
        type XorSplitHandle;

        // Library
        fn read_gds_shim(filename: &str) -> UniquePtr<LibraryHandle>;
        fn library_cell_count(handle: &LibraryHandle) -> u64;
        fn library_cell_at(handle: &LibraryHandle, idx: u64) -> &CellHandle;
        fn library_name(handle: &LibraryHandle) -> &str;
        fn library_unit(handle: &LibraryHandle) -> f64;
        fn library_precision(handle: &LibraryHandle) -> f64;
        fn library_write_gds(handle: &LibraryHandle, path: &str) -> u8;
        fn library_top_level(handle: &LibraryHandle) -> UniquePtr<TopLevelView>;
        fn top_level_count(view: &TopLevelView) -> u64;
        fn top_level_at(view: &TopLevelView, idx: u64) -> &CellHandle;

        // GdsInfo — fast metadata peek
        fn gds_info_read(path: &str, out_error: &mut u8) -> UniquePtr<GdsInfoHandle>;
        fn gds_info_unit(h: &GdsInfoHandle) -> f64;
        fn gds_info_precision(h: &GdsInfoHandle) -> f64;
        fn gds_info_num_polygons(h: &GdsInfoHandle) -> u64;
        fn gds_info_num_paths(h: &GdsInfoHandle) -> u64;
        fn gds_info_num_references(h: &GdsInfoHandle) -> u64;
        fn gds_info_num_labels(h: &GdsInfoHandle) -> u64;
        fn gds_info_cell_count(h: &GdsInfoHandle) -> u64;
        fn gds_info_cell_name(h: &GdsInfoHandle, idx: u64) -> &str;
        fn gds_info_shape_tag_count(h: &GdsInfoHandle) -> u64;
        fn gds_info_shape_tag(h: &GdsInfoHandle, idx: u64) -> GdsTag;
        fn gds_info_label_tag_count(h: &GdsInfoHandle) -> u64;
        fn gds_info_label_tag(h: &GdsInfoHandle, idx: u64) -> GdsTag;

        // Cell
        fn cell_name(cell: &CellHandle) -> &str;
        fn cell_polygon_count(cell: &CellHandle) -> u64;
        fn cell_polygon_at(cell: &CellHandle, idx: u64) -> &PolygonHandle;
        fn cell_label_count(cell: &CellHandle) -> u64;
        fn cell_label_at(cell: &CellHandle, idx: u64) -> &LabelHandle;
        fn cell_reference_count(cell: &CellHandle) -> u64;
        fn cell_reference_at(cell: &CellHandle, idx: u64) -> &ReferenceHandle;
        fn cell_flexpath_count(cell: &CellHandle) -> u64;
        fn cell_flexpath_at(cell: &CellHandle, idx: u64) -> &FlexPathHandle;
        fn cell_robustpath_count(cell: &CellHandle) -> u64;
        fn cell_robustpath_at(cell: &CellHandle, idx: u64) -> &RobustPathHandle;

        // Polygon
        fn polygon_area(poly: &PolygonHandle) -> f64;
        fn polygon_layer(poly: &PolygonHandle) -> u32;
        fn polygon_datatype(poly: &PolygonHandle) -> u32;
        fn polygon_bbox(poly: &PolygonHandle) -> BoundingBox;
        fn polygon_point_count(poly: &PolygonHandle) -> u64;
        fn polygon_point_at(poly: &PolygonHandle, idx: u64) -> Point2D;

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

        // FlexPath
        fn flexpath_num_elements(path: &FlexPathHandle) -> u64;
        fn flexpath_element_layer(path: &FlexPathHandle, element_idx: u64) -> u32;
        fn flexpath_element_datatype(path: &FlexPathHandle, element_idx: u64) -> u32;
        fn flexpath_spine_point_count(path: &FlexPathHandle) -> u64;
        fn flexpath_spine_point(path: &FlexPathHandle, point_idx: u64) -> Point2D;
        fn flexpath_element_half_width(
            path: &FlexPathHandle,
            element_idx: u64,
            spine_idx: u64,
        ) -> f64;
        fn flexpath_element_offset(path: &FlexPathHandle, element_idx: u64, spine_idx: u64) -> f64;
        fn flexpath_element_end_type(path: &FlexPathHandle, element_idx: u64) -> u8;
        fn flexpath_element_join_type(path: &FlexPathHandle, element_idx: u64) -> u8;
        fn flexpath_element_bend_type(path: &FlexPathHandle, element_idx: u64) -> u8;
        fn flexpath_element_bend_radius(path: &FlexPathHandle, element_idx: u64) -> f64;
        fn flexpath_element_end_extensions(path: &FlexPathHandle, element_idx: u64) -> Point2D;
        fn flexpath_simple_path(path: &FlexPathHandle) -> bool;
        fn flexpath_scale_width(path: &FlexPathHandle) -> bool;

        // RobustPath
        fn robustpath_num_elements(path: &RobustPathHandle) -> u64;
        fn robustpath_element_layer(path: &RobustPathHandle, element_idx: u64) -> u32;
        fn robustpath_element_datatype(path: &RobustPathHandle, element_idx: u64) -> u32;
        fn robustpath_subpath_count(path: &RobustPathHandle) -> u64;
        fn robustpath_end_point(path: &RobustPathHandle) -> Point2D;
        fn robustpath_tolerance(path: &RobustPathHandle) -> f64;
        fn robustpath_max_evals(path: &RobustPathHandle) -> u64;
        fn robustpath_element_end_width(path: &RobustPathHandle, element_idx: u64) -> f64;
        fn robustpath_element_end_offset(path: &RobustPathHandle, element_idx: u64) -> f64;
        fn robustpath_element_end_type(path: &RobustPathHandle, element_idx: u64) -> u8;
        fn robustpath_simple_path(path: &RobustPathHandle) -> bool;
        fn robustpath_scale_width(path: &RobustPathHandle) -> bool;

        // Fase 8: gap-closing accessors
        fn polygon_perimeter(poly: &PolygonHandle) -> f64;
        fn polygon_signed_area(poly: &PolygonHandle) -> f64;
        fn polygon_repetition_count(poly: &PolygonHandle) -> u64;
        fn polygon_repetition_offset(poly: &PolygonHandle, idx: u64) -> Point2D;
        fn label_repetition_count(label: &LabelHandle) -> u64;
        fn label_repetition_offset(label: &LabelHandle, idx: u64) -> Point2D;
        fn reference_repetition_count(r: &ReferenceHandle) -> u64;
        fn reference_repetition_offset(r: &ReferenceHandle, idx: u64) -> Point2D;
        fn cell_bbox(cell: &CellHandle) -> BoundingBox;
        fn reference_bbox(r: &ReferenceHandle) -> BoundingBox;

        // RawCell
        fn library_rawcell_count(handle: &LibraryHandle) -> u64;
        fn library_rawcell_at(handle: &LibraryHandle, idx: u64) -> &RawCellHandle;
        fn rawcell_name(rc: &RawCellHandle) -> &str;
        fn rawcell_size(rc: &RawCellHandle) -> u64;
        fn rawcell_dependency_count(rc: &RawCellHandle) -> u64;
        fn rawcell_dependency_at(rc: &RawCellHandle, idx: u64) -> &RawCellHandle;

        // Fase 8.5: Repetition struct detail
        fn polygon_repetition(poly: &PolygonHandle) -> &RepetitionHandle;
        fn label_repetition(label: &LabelHandle) -> &RepetitionHandle;
        fn reference_repetition(r: &ReferenceHandle) -> &RepetitionHandle;
        fn repetition_type(rep: &RepetitionHandle) -> u8;
        #[rust_name = "repetition_total_count"]
        fn repetition_count(rep: &RepetitionHandle) -> u64;
        fn repetition_columns(rep: &RepetitionHandle) -> u64;
        fn repetition_rows(rep: &RepetitionHandle) -> u64;
        fn repetition_spacing(rep: &RepetitionHandle) -> Point2D;
        fn repetition_v1(rep: &RepetitionHandle) -> Point2D;
        fn repetition_v2(rep: &RepetitionHandle) -> Point2D;
        fn repetition_coord_count(rep: &RepetitionHandle) -> u64;
        fn repetition_coord(rep: &RepetitionHandle, idx: u64) -> f64;
        fn repetition_generated_offset(rep: &RepetitionHandle, idx: u64) -> Point2D;
        fn repetition_extrema_count(rep: &RepetitionHandle) -> u64;
        fn repetition_extremum(rep: &RepetitionHandle, idx: u64) -> Point2D;

        // Flatten: get_polygons with transformations applied.
        fn cell_get_polygons_flat(
            cell: &CellHandle,
            apply_repetitions: bool,
            include_paths: bool,
            depth: i64,
            use_filter: bool,
            layer: u32,
            datatype: u32,
        ) -> UniquePtr<FlattenedPolygonsHandle>;
        fn reference_get_polygons_flat(
            r: &ReferenceHandle,
            apply_repetitions: bool,
            include_paths: bool,
            depth: i64,
            use_filter: bool,
            layer: u32,
            datatype: u32,
        ) -> UniquePtr<FlattenedPolygonsHandle>;
        fn flattened_polygons_count(view: &FlattenedPolygonsHandle) -> u64;
        fn flattened_polygons_at(view: &FlattenedPolygonsHandle, idx: u64) -> &PolygonHandle;

        // Boolean XOR (includes path-derived polygons).
        fn cell_xor_with(a: &CellHandle, b: &CellHandle, layer: u32) -> XorMetrics;
        // Legacy: polygons only, ignores paths.
        fn cell_xor_with_polygons_only(a: &CellHandle, b: &CellHandle, layer: u32) -> XorMetrics;

        // Directional XOR (added/removed split).
        fn cell_xor_polygons_split(
            a: &CellHandle,
            b: &CellHandle,
            layer: u32,
        ) -> UniquePtr<XorSplitHandle>;
        fn xor_split_added_count(h: &XorSplitHandle) -> u64;
        fn xor_split_removed_count(h: &XorSplitHandle) -> u64;
        fn xor_split_added_layer(h: &XorSplitHandle, poly_idx: u64) -> u32;
        fn xor_split_added_datatype(h: &XorSplitHandle, poly_idx: u64) -> u32;
        fn xor_split_added_point_count(h: &XorSplitHandle, poly_idx: u64) -> u64;
        fn xor_split_added_point(h: &XorSplitHandle, poly_idx: u64, point_idx: u64) -> Point2D;
        fn xor_split_removed_layer(h: &XorSplitHandle, poly_idx: u64) -> u32;
        fn xor_split_removed_datatype(h: &XorSplitHandle, poly_idx: u64) -> u32;
        fn xor_split_removed_point_count(h: &XorSplitHandle, poly_idx: u64) -> u64;
        fn xor_split_removed_point(h: &XorSplitHandle, poly_idx: u64, point_idx: u64) -> Point2D;

        // Library tag discovery (lazy-cached set of (layer, datatype)).
        fn library_tag_count(handle: &LibraryHandle) -> u64;
        fn library_tag_at(handle: &LibraryHandle, idx: u64) -> GdsTag;
    }
}

pub use ffi::{BoundingBox, GdsTag, Point2D, XorMetrics};

/// Self-contained polygon with no lifetime to a Library. Used for diff
/// results that must outlive the cells they came from (e.g. passing XOR
/// geometry to a renderer or to another crate).
#[derive(Clone, Debug, PartialEq)]
pub struct OwnedPolygon {
    pub layer: u32,
    pub datatype: u32,
    pub points: Vec<Point2D>,
}

/// Result of `Cell::xor_polygons_split`. Polygons are partitioned by
/// direction so a UI can paint them differently (e.g. green = added,
/// red = removed).
#[derive(Clone, Debug, Default)]
pub struct XorSplit {
    /// Polygons present in `other` but not in `self` (B \ A).
    pub added: Vec<OwnedPolygon>,
    /// Polygons present in `self` but not in `other` (A \ B).
    pub removed: Vec<OwnedPolygon>,
}

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

    /// GDSII library name. Empty string if the library has no name set.
    pub fn name(&self) -> &str {
        ffi::library_name(&self.inner)
    }

    /// Unit in meters (e.g. 1e-6 = µm, 1e-9 = nm).
    pub fn unit(&self) -> f64 {
        ffi::library_unit(&self.inner)
    }

    /// Precision in meters (database resolution within the unit).
    pub fn precision(&self) -> f64 {
        ffi::library_precision(&self.inner)
    }

    /// Write the library to a GDSII file using defaults (no fracture,
    /// current timestamp).
    pub fn write_gds(&self, path: &str) -> Result<(), Error> {
        let code = ffi::library_write_gds(&self.inner, path);
        let ec = ErrorCode::from_u8(code);
        if ec == ErrorCode::NoError {
            Ok(())
        } else {
            Err(Error(ec))
        }
    }

    /// Cells that are not referenced by any other cell (top-level).
    pub fn top_level(&self) -> TopLevel<'_> {
        TopLevel {
            view: ffi::library_top_level(&self.inner),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Top-level cells of a Library. Borrows from the Library it was derived from.
pub struct TopLevel<'a> {
    view: cxx::UniquePtr<ffi::TopLevelView>,
    _marker: std::marker::PhantomData<&'a Library>,
}

impl<'a> TopLevel<'a> {
    pub fn count(&self) -> u64 {
        ffi::top_level_count(&self.view)
    }

    pub fn cell(&self, idx: u64) -> Cell<'a> {
        // SAFETY: the CellHandle returned by top_level_at is a pointer
        // to a gdstk::Cell that lives in the parent Library's cell_array.
        // The Library outlives this TopLevel (enforced by 'a lifetime on
        // PhantomData<&'a Library>), so extending the borrow from the
        // TopLevelView's lifetime to 'a is sound. cxx sees it as &view.
        let handle: &ffi::CellHandle = ffi::top_level_at(&self.view, idx);
        let handle_a: &'a ffi::CellHandle = unsafe { std::mem::transmute(handle) };
        Cell { handle: handle_a }
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell<'a>> + '_ {
        (0..self.count()).map(move |i| self.cell(i))
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

    pub fn flexpath_count(&self) -> u64 {
        ffi::cell_flexpath_count(self.handle)
    }

    pub fn flexpath(&self, idx: u64) -> FlexPath<'a> {
        FlexPath {
            handle: ffi::cell_flexpath_at(self.handle, idx),
        }
    }

    pub fn flexpaths(&self) -> impl Iterator<Item = FlexPath<'a>> + use<'a> {
        let this = *self;
        (0..this.flexpath_count()).map(move |i| this.flexpath(i))
    }

    pub fn robustpath_count(&self) -> u64 {
        ffi::cell_robustpath_count(self.handle)
    }

    pub fn robustpath(&self, idx: u64) -> RobustPath<'a> {
        RobustPath {
            handle: ffi::cell_robustpath_at(self.handle, idx),
        }
    }

    pub fn robustpaths(&self) -> impl Iterator<Item = RobustPath<'a>> + use<'a> {
        let this = *self;
        (0..this.robustpath_count()).map(move |i| this.robustpath(i))
    }

    /// Geometric XOR between two cells, filtered by layer. Converts FlexPath
    /// and RobustPath to polygons internally before the diff, so wire-like
    /// geometry is correctly compared. Does NOT flatten cell references.
    ///
    /// Returns area, region count, and bbox of the symmetric difference.
    pub fn xor_with(&self, other: &Cell<'_>, layer: u32) -> XorMetrics {
        ffi::cell_xor_with(self.handle, other.handle, layer)
    }

    /// Legacy XOR: polygons only, ignores FlexPath and RobustPath. Use
    /// `xor_with` for correct diff; this exists for cases where you only
    /// care about shapes already saved as polygons.
    pub fn xor_with_polygons_only(&self, other: &Cell<'_>, layer: u32) -> XorMetrics {
        ffi::cell_xor_with_polygons_only(self.handle, other.handle, layer)
    }

    /// Directional XOR. Returns the polygons of the difference partitioned
    /// into `added` (in `other` but not in `self`) and `removed` (in `self`
    /// but not in `other`), filtered by `layer`. Includes path-derived
    /// polygons (FlexPath / RobustPath polygonized internally).
    ///
    /// Costs roughly twice a single `xor_with` (two `boolean` calls) and
    /// allocates owned geometry. Use `xor_with` for a fast scalar summary.
    pub fn xor_polygons_split(&self, other: &Cell<'_>, layer: u32) -> XorSplit {
        let h = ffi::cell_xor_polygons_split(self.handle, other.handle, layer);
        XorSplit {
            added: collect_split_polys(&h, SplitSide::Added),
            removed: collect_split_polys(&h, SplitSide::Removed),
        }
    }

    /// Axis-aligned bounding box covering all polygons, labels, paths, and
    /// references in this cell. Returns a zero box if the cell is empty.
    pub fn bbox(&self) -> BoundingBox {
        ffi::cell_bbox(self.handle)
    }

    /// Start a builder to flatten this cell's polygons with transformations
    /// and references expanded recursively. Defaults:
    /// `apply_repetitions=true`, `include_paths=true`, `depth=-1` (unlimited
    /// recursion into references), no filter.
    pub fn get_polygons(&self) -> GetPolygonsBuilder<'a> {
        GetPolygonsBuilder {
            source: GetPolygonsSource::Cell(self.handle),
            apply_repetitions: true,
            include_paths: true,
            depth: -1,
            filter: None,
        }
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

    /// Vertex at `idx`. Returns (0,0) for out-of-range.
    pub fn point(&self, idx: u64) -> Point2D {
        ffi::polygon_point_at(self.handle, idx)
    }

    /// Iterator over all vertices of the polygon.
    pub fn points(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.point_count()).map(move |i| this.point(i))
    }

    /// Perímetro total (suma de longitudes de los segmentos).
    pub fn perimeter(&self) -> f64 {
        ffi::polygon_perimeter(self.handle)
    }

    /// Área con signo. Negativo si el polígono está orientado en sentido horario.
    pub fn signed_area(&self) -> f64 {
        ffi::polygon_signed_area(self.handle)
    }

    /// Número de instancias incluyendo el origen (1 si no hay repetición).
    pub fn repetition_count(&self) -> u64 {
        ffi::polygon_repetition_count(self.handle)
    }

    /// Offset de la instancia en `idx` (0 = origen). (0,0) si out-of-range.
    pub fn repetition_offset(&self, idx: u64) -> Point2D {
        ffi::polygon_repetition_offset(self.handle, idx)
    }

    pub fn repetition_offsets(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.repetition_count()).map(move |i| this.repetition_offset(i))
    }

    /// Full repetition detail (kind, columns, rows, spacing, etc).
    pub fn repetition(&self) -> Repetition<'a> {
        Repetition {
            handle: ffi::polygon_repetition(self.handle),
        }
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

    pub fn repetition_count(&self) -> u64 {
        ffi::label_repetition_count(self.handle)
    }

    pub fn repetition_offset(&self, idx: u64) -> Point2D {
        ffi::label_repetition_offset(self.handle, idx)
    }

    pub fn repetition_offsets(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.repetition_count()).map(move |i| this.repetition_offset(i))
    }

    /// Full repetition detail (kind, columns, rows, spacing, etc).
    pub fn repetition(&self) -> Repetition<'a> {
        Repetition {
            handle: ffi::label_repetition(self.handle),
        }
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

    /// Axis-aligned bounding box of this reference (instance) considering
    /// its transformation.
    pub fn bbox(&self) -> BoundingBox {
        ffi::reference_bbox(self.handle)
    }

    pub fn repetition_count(&self) -> u64 {
        ffi::reference_repetition_count(self.handle)
    }

    pub fn repetition_offset(&self, idx: u64) -> Point2D {
        ffi::reference_repetition_offset(self.handle, idx)
    }

    pub fn repetition_offsets(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.repetition_count()).map(move |i| this.repetition_offset(i))
    }

    /// Full repetition detail (kind, columns, rows, spacing, etc).
    pub fn repetition(&self) -> Repetition<'a> {
        Repetition {
            handle: ffi::reference_repetition(self.handle),
        }
    }

    /// Start a builder to flatten the referenced cell's polygons with this
    /// reference's transformation (rotation, magnification, x_reflection,
    /// origin) applied, and recursively expand nested references.
    /// Defaults: `apply_repetitions=true`, `include_paths=true`, `depth=-1`,
    /// no filter.
    pub fn get_polygons(&self) -> GetPolygonsBuilder<'a> {
        GetPolygonsBuilder {
            source: GetPolygonsSource::Reference(self.handle),
            apply_repetitions: true,
            include_paths: true,
            depth: -1,
            filter: None,
        }
    }
}

/// Shape of the line endings at path termini. Matches `gdstk::EndType`.
///
/// `Function` means a custom C callback is set — Miku's Rust binding
/// cannot introspect it; the other fields still work.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndType {
    Flush = 0,
    Round = 1,
    HalfWidth = 2,
    Extended = 3,
    Smooth = 4,
    Function = 5,
}

impl EndType {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Flush,
            1 => Self::Round,
            2 => Self::HalfWidth,
            3 => Self::Extended,
            4 => Self::Smooth,
            5 => Self::Function,
            _ => Self::Flush,
        }
    }
}

/// Shape of a corner between path segments. Matches `gdstk::JoinType`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinType {
    Natural = 0,
    Miter = 1,
    Bevel = 2,
    Round = 3,
    Smooth = 4,
    Function = 5,
}

impl JoinType {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Natural,
            1 => Self::Miter,
            2 => Self::Bevel,
            3 => Self::Round,
            4 => Self::Smooth,
            5 => Self::Function,
            _ => Self::Natural,
        }
    }
}

/// Kind of bend applied at path corners. Matches `gdstk::BendType`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BendType {
    None = 0,
    Circular = 1,
    Function = 2,
}

impl BendType {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::None,
            1 => Self::Circular,
            2 => Self::Function,
            _ => Self::None,
        }
    }
}

/// Borrowed view into a FlexPath inside a Cell.
///
/// A FlexPath can contain multiple "elements", each with its own
/// layer/datatype/width/offset. Think of it as one spine (a series of
/// points) used to draw one or more parallel wires.
#[derive(Clone, Copy)]
pub struct FlexPath<'a> {
    handle: &'a ffi::FlexPathHandle,
}

impl<'a> FlexPath<'a> {
    /// Number of path elements (distinct layer/width/offset configurations).
    pub fn num_elements(&self) -> u64 {
        ffi::flexpath_num_elements(self.handle)
    }

    pub fn element_layer(&self, element_idx: u64) -> u32 {
        ffi::flexpath_element_layer(self.handle, element_idx)
    }

    pub fn element_datatype(&self, element_idx: u64) -> u32 {
        ffi::flexpath_element_datatype(self.handle, element_idx)
    }

    /// Number of points in the spine curve.
    pub fn spine_point_count(&self) -> u64 {
        ffi::flexpath_spine_point_count(self.handle)
    }

    /// Spine point at `idx`. Returns origin if `idx` is out of range.
    pub fn spine_point(&self, idx: u64) -> Point2D {
        ffi::flexpath_spine_point(self.handle, idx)
    }

    /// Half-width of element at a given spine index. Returns 0.0 if either
    /// index is out of range (including simple_path paths where the width
    /// array has fewer entries than the spine).
    pub fn element_half_width(&self, element_idx: u64, spine_idx: u64) -> f64 {
        ffi::flexpath_element_half_width(self.handle, element_idx, spine_idx)
    }

    /// Offset of element at a given spine index relative to the spine.
    pub fn element_offset(&self, element_idx: u64, spine_idx: u64) -> f64 {
        ffi::flexpath_element_offset(self.handle, element_idx, spine_idx)
    }

    pub fn element_end_type(&self, element_idx: u64) -> EndType {
        EndType::from_u8(ffi::flexpath_element_end_type(self.handle, element_idx))
    }

    pub fn element_join_type(&self, element_idx: u64) -> JoinType {
        JoinType::from_u8(ffi::flexpath_element_join_type(self.handle, element_idx))
    }

    pub fn element_bend_type(&self, element_idx: u64) -> BendType {
        BendType::from_u8(ffi::flexpath_element_bend_type(self.handle, element_idx))
    }

    /// Bend radius for `BendType::Circular`; otherwise the value is
    /// informational but not used by gdstk.
    pub fn element_bend_radius(&self, element_idx: u64) -> f64 {
        ffi::flexpath_element_bend_radius(self.handle, element_idx)
    }

    /// End extensions applied at path termini (only meaningful for
    /// `EndType::Extended`).
    pub fn element_end_extensions(&self, element_idx: u64) -> Point2D {
        ffi::flexpath_element_end_extensions(self.handle, element_idx)
    }

    /// If true, path is saved as a GDSII PATH record with constant width.
    pub fn simple_path(&self) -> bool {
        ffi::flexpath_simple_path(self.handle)
    }

    /// If true, widths scale with the path's transform.
    pub fn scale_width(&self) -> bool {
        ffi::flexpath_scale_width(self.handle)
    }
}

/// Borrowed view into a RobustPath inside a Cell.
///
/// Like FlexPath but with parametric spine (subpaths with smooth transitions).
#[derive(Clone, Copy)]
pub struct RobustPath<'a> {
    handle: &'a ffi::RobustPathHandle,
}

impl<'a> RobustPath<'a> {
    pub fn num_elements(&self) -> u64 {
        ffi::robustpath_num_elements(self.handle)
    }

    pub fn element_layer(&self, element_idx: u64) -> u32 {
        ffi::robustpath_element_layer(self.handle, element_idx)
    }

    pub fn element_datatype(&self, element_idx: u64) -> u32 {
        ffi::robustpath_element_datatype(self.handle, element_idx)
    }

    /// Number of subpaths in the parametric spine.
    pub fn subpath_count(&self) -> u64 {
        ffi::robustpath_subpath_count(self.handle)
    }

    /// Last point on the path.
    pub fn end_point(&self) -> Point2D {
        ffi::robustpath_end_point(self.handle)
    }

    /// Numeric tolerance for spline approximation.
    pub fn tolerance(&self) -> f64 {
        ffi::robustpath_tolerance(self.handle)
    }

    pub fn max_evals(&self) -> u64 {
        ffi::robustpath_max_evals(self.handle)
    }

    pub fn element_end_width(&self, element_idx: u64) -> f64 {
        ffi::robustpath_element_end_width(self.handle, element_idx)
    }

    pub fn element_end_offset(&self, element_idx: u64) -> f64 {
        ffi::robustpath_element_end_offset(self.handle, element_idx)
    }

    pub fn element_end_type(&self, element_idx: u64) -> EndType {
        EndType::from_u8(ffi::robustpath_element_end_type(self.handle, element_idx))
    }

    pub fn simple_path(&self) -> bool {
        ffi::robustpath_simple_path(self.handle)
    }

    pub fn scale_width(&self) -> bool {
        ffi::robustpath_scale_width(self.handle)
    }
}

// ---- Error handling ----

/// Mirror of `gdstk::ErrorCode` (utils.hpp) with stable numeric values.
/// Values 1..=8 are warnings; 9..=16 are errors.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    NoError = 0,
    BooleanError = 1,
    EmptyPath = 2,
    IntersectionNotFound = 3,
    MissingReference = 4,
    UnsupportedRecord = 5,
    UnofficialSpecification = 6,
    InvalidRepetition = 7,
    Overflow = 8,
    ChecksumError = 9,
    OutputFileOpenError = 10,
    InputFileOpenError = 11,
    InputFileError = 12,
    FileError = 13,
    InvalidFile = 14,
    InsufficientMemory = 15,
    ZlibError = 16,
}

impl ErrorCode {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::NoError,
            1 => Self::BooleanError,
            2 => Self::EmptyPath,
            3 => Self::IntersectionNotFound,
            4 => Self::MissingReference,
            5 => Self::UnsupportedRecord,
            6 => Self::UnofficialSpecification,
            7 => Self::InvalidRepetition,
            8 => Self::Overflow,
            9 => Self::ChecksumError,
            10 => Self::OutputFileOpenError,
            11 => Self::InputFileOpenError,
            12 => Self::InputFileError,
            13 => Self::FileError,
            14 => Self::InvalidFile,
            15 => Self::InsufficientMemory,
            16 => Self::ZlibError,
            _ => Self::NoError,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::NoError => "no error",
            Self::BooleanError => "boolean operation error",
            Self::EmptyPath => "empty path",
            Self::IntersectionNotFound => "intersection not found",
            Self::MissingReference => "missing reference",
            Self::UnsupportedRecord => "unsupported GDSII record",
            Self::UnofficialSpecification => "unofficial specification",
            Self::InvalidRepetition => "invalid repetition",
            Self::Overflow => "overflow",
            Self::ChecksumError => "checksum error",
            Self::OutputFileOpenError => "could not open output file",
            Self::InputFileOpenError => "could not open input file",
            Self::InputFileError => "input file read error",
            Self::FileError => "file error",
            Self::InvalidFile => "invalid file",
            Self::InsufficientMemory => "insufficient memory",
            Self::ZlibError => "zlib error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(pub ErrorCode);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gdstk error: {}", self.0.as_str())
    }
}

impl std::error::Error for Error {}

// ---- GdsInfo ----

/// Fast peek of GDSII metadata (cell names, layer tags, counts) without
/// parsing the full file. Orders of magnitude faster than `Library::open`.
/// Useful for VCS-style workflows (e.g. `miku log --stat`).
pub struct GdsInfo {
    inner: cxx::UniquePtr<ffi::GdsInfoHandle>,
}

impl std::fmt::Debug for GdsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GdsInfo")
            .field("unit", &self.unit())
            .field("precision", &self.precision())
            .field("cell_count", &self.cell_count())
            .field("num_polygons", &self.num_polygons())
            .field("num_paths", &self.num_paths())
            .field("num_references", &self.num_references())
            .field("num_labels", &self.num_labels())
            .finish()
    }
}

/// Read metadata from a GDSII file without a full parse.
pub fn gds_info(path: &str) -> Result<GdsInfo, Error> {
    let mut code: u8 = 0;
    let handle = ffi::gds_info_read(path, &mut code);
    let ec = ErrorCode::from_u8(code);
    if ec != ErrorCode::NoError || handle.is_null() {
        return Err(Error(ec));
    }
    Ok(GdsInfo { inner: handle })
}

impl GdsInfo {
    pub fn unit(&self) -> f64 {
        ffi::gds_info_unit(&self.inner)
    }

    pub fn precision(&self) -> f64 {
        ffi::gds_info_precision(&self.inner)
    }

    pub fn num_polygons(&self) -> u64 {
        ffi::gds_info_num_polygons(&self.inner)
    }

    pub fn num_paths(&self) -> u64 {
        ffi::gds_info_num_paths(&self.inner)
    }

    pub fn num_references(&self) -> u64 {
        ffi::gds_info_num_references(&self.inner)
    }

    pub fn num_labels(&self) -> u64 {
        ffi::gds_info_num_labels(&self.inner)
    }

    pub fn cell_count(&self) -> u64 {
        ffi::gds_info_cell_count(&self.inner)
    }

    pub fn cell_name(&self, idx: u64) -> &str {
        ffi::gds_info_cell_name(&self.inner, idx)
    }

    pub fn cell_names(&self) -> impl Iterator<Item = &str> + '_ {
        (0..self.cell_count()).map(move |i| self.cell_name(i))
    }

    pub fn shape_tag_count(&self) -> u64 {
        ffi::gds_info_shape_tag_count(&self.inner)
    }

    pub fn shape_tag(&self, idx: u64) -> GdsTag {
        ffi::gds_info_shape_tag(&self.inner, idx)
    }

    pub fn shape_tags(&self) -> impl Iterator<Item = GdsTag> + '_ {
        (0..self.shape_tag_count()).map(move |i| self.shape_tag(i))
    }

    pub fn label_tag_count(&self) -> u64 {
        ffi::gds_info_label_tag_count(&self.inner)
    }

    pub fn label_tag(&self, idx: u64) -> GdsTag {
        ffi::gds_info_label_tag(&self.inner, idx)
    }

    pub fn label_tags(&self) -> impl Iterator<Item = GdsTag> + '_ {
        (0..self.label_tag_count()).map(move |i| self.label_tag(i))
    }
}

/// Kind of repetition attached to a Polygon, Label, or Reference.
/// Mirrors `gdstk::RepetitionType` (utils.hpp).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepetitionType {
    /// Not repeated — a single instance at origin.
    None = 0,
    /// columns × rows grid with axis-aligned `spacing`.
    Rectangular = 1,
    /// columns × rows grid along arbitrary axes `v1` and `v2`.
    Regular = 2,
    /// Explicit list of offsets (origin NOT included in the raw array).
    Explicit = 3,
    /// Repetitions along the x axis at explicit coords.
    ExplicitX = 4,
    /// Repetitions along the y axis at explicit coords.
    ExplicitY = 5,
}

impl RepetitionType {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::None,
            1 => Self::Rectangular,
            2 => Self::Regular,
            3 => Self::Explicit,
            4 => Self::ExplicitX,
            5 => Self::ExplicitY,
            _ => Self::None,
        }
    }
}

/// Borrowed view into a `gdstk::Repetition`. Obtained via
/// `polygon.repetition()`, `label.repetition()`, `reference.repetition()`.
///
/// The interesting fields depend on `kind()`:
/// - `Rectangular`: `columns`, `rows`, `spacing`
/// - `Regular`: `columns`, `rows`, `v1`, `v2`
/// - `Explicit`: iterate `explicit_offsets()` (each excluding origin)
/// - `ExplicitX` / `ExplicitY`: iterate `coords()`
/// - `None`: single instance at origin; `count()` returns 1
#[derive(Clone, Copy)]
pub struct Repetition<'a> {
    handle: &'a ffi::RepetitionHandle,
}

impl<'a> Repetition<'a> {
    /// Variant of this repetition.
    pub fn kind(&self) -> RepetitionType {
        RepetitionType::from_u8(ffi::repetition_type(self.handle))
    }

    /// Effective total instance count. Returns 1 for `None`.
    pub fn count(&self) -> u64 {
        ffi::repetition_total_count(self.handle)
    }

    /// For `Rectangular` / `Regular`. 0 otherwise.
    pub fn columns(&self) -> u64 {
        ffi::repetition_columns(self.handle)
    }

    /// For `Rectangular` / `Regular`. 0 otherwise.
    pub fn rows(&self) -> u64 {
        ffi::repetition_rows(self.handle)
    }

    /// For `Rectangular`. Zero point otherwise.
    pub fn spacing(&self) -> Point2D {
        ffi::repetition_spacing(self.handle)
    }

    /// For `Regular` (first axis). Zero point otherwise.
    pub fn v1(&self) -> Point2D {
        ffi::repetition_v1(self.handle)
    }

    /// For `Regular` (second axis). Zero point otherwise.
    pub fn v2(&self) -> Point2D {
        ffi::repetition_v2(self.handle)
    }

    /// For `ExplicitX` / `ExplicitY`: number of coords.
    pub fn coord_count(&self) -> u64 {
        ffi::repetition_coord_count(self.handle)
    }

    pub fn coord(&self, idx: u64) -> f64 {
        ffi::repetition_coord(self.handle, idx)
    }

    pub fn coords(&self) -> impl Iterator<Item = f64> + use<'a> {
        let this = *self;
        (0..this.coord_count()).map(move |i| this.coord(i))
    }

    /// Generated offset at `idx`, including the origin (0,0) as idx=0.
    /// Works for all repetition kinds.
    pub fn offset(&self, idx: u64) -> Point2D {
        ffi::repetition_generated_offset(self.handle, idx)
    }

    pub fn offsets(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.count()).map(move |i| this.offset(i))
    }

    /// Number of extrema (boundary corners) of the repetition pattern.
    /// Returns 1-4 for rectangular/regular grids, varies for explicit.
    /// For `None`: always 1 (the origin).
    pub fn extrema_count(&self) -> u64 {
        ffi::repetition_extrema_count(self.handle)
    }

    /// Extremum at `idx`. (0,0) for out-of-range.
    /// For `None` kind, `extremum(0)` is the origin.
    pub fn extremum(&self, idx: u64) -> Point2D {
        ffi::repetition_extremum(self.handle, idx)
    }

    /// Iterator over all extrema (boundary corners).
    pub fn extrema(&self) -> impl Iterator<Item = Point2D> + use<'a> {
        let this = *self;
        (0..this.extrema_count()).map(move |i| this.extremum(i))
    }
}

/// Borrowed view into a RawCell of a Library. RawCells are cells stored
/// as raw GDSII bytes (unparsed) — typically imported external IP libraries.
/// Miku can iterate them and see name/size but cannot inspect geometry inside.
#[derive(Clone, Copy)]
pub struct RawCell<'a> {
    handle: &'a ffi::RawCellHandle,
}

impl<'a> RawCell<'a> {
    pub fn name(&self) -> &'a str {
        ffi::rawcell_name(self.handle)
    }

    /// Size of the raw GDSII bytes backing this cell.
    pub fn size(&self) -> u64 {
        ffi::rawcell_size(self.handle)
    }

    pub fn dependency_count(&self) -> u64 {
        ffi::rawcell_dependency_count(self.handle)
    }

    pub fn dependency(&self, idx: u64) -> RawCell<'a> {
        RawCell {
            handle: ffi::rawcell_dependency_at(self.handle, idx),
        }
    }

    pub fn dependencies(&self) -> impl Iterator<Item = RawCell<'a>> + use<'a> {
        let this = *self;
        (0..this.dependency_count()).map(move |i| this.dependency(i))
    }
}

impl Library {
    pub fn rawcell_count(&self) -> u64 {
        ffi::library_rawcell_count(&self.inner)
    }

    pub fn rawcell(&self, idx: u64) -> RawCell<'_> {
        RawCell {
            handle: ffi::library_rawcell_at(&self.inner, idx),
        }
    }

    pub fn rawcells(&self) -> impl Iterator<Item = RawCell<'_>> {
        (0..self.rawcell_count()).map(move |i| self.rawcell(i))
    }
}

// ---- Flatten: Cell::get_polygons / Reference::get_polygons ----

/// Internal: either a Cell or a Reference as the source for flattening.
enum GetPolygonsSource<'a> {
    Cell(&'a ffi::CellHandle),
    Reference(&'a ffi::ReferenceHandle),
}

/// Builder for `Cell::get_polygons` and `Reference::get_polygons`. Configure
/// the recursion depth and filter, then call `.build()` to materialize the
/// flattened polygons with transformations applied.
///
/// Defaults: `apply_repetitions=true`, `include_paths=true`, `depth=-1`
/// (unlimited), no filter.
///
/// **Warning:** with `apply_repetitions=true` and deep hierarchies, the
/// number of polygons can grow multiplicatively. Use `depth(0)` to limit
/// to the immediate level or `with_filter()` to restrict to one layer.
#[must_use]
pub struct GetPolygonsBuilder<'a> {
    source: GetPolygonsSource<'a>,
    apply_repetitions: bool,
    include_paths: bool,
    depth: i64,
    filter: Option<GdsTag>,
}

impl<'a> GetPolygonsBuilder<'a> {
    /// Apply the repetition of polygons/references. Default: true.
    pub fn with_repetitions(mut self, v: bool) -> Self {
        self.apply_repetitions = v;
        self
    }

    /// Convert FlexPath / RobustPath to polygons. Default: true.
    pub fn with_paths(mut self, v: bool) -> Self {
        self.include_paths = v;
        self
    }

    /// Recursion depth for expanding references. Default: -1 (unlimited).
    /// - `-1`: recurse into all references
    /// - `0`: only direct polygons (no reference expansion)
    /// - `n > 0`: recurse up to n levels
    pub fn depth(mut self, d: i64) -> Self {
        self.depth = d;
        self
    }

    /// Only return polygons matching this (layer, datatype). Default: no filter.
    pub fn with_filter(mut self, layer: u32, datatype: u32) -> Self {
        self.filter = Some(GdsTag { layer, datatype });
        self
    }

    /// Execute the flatten and return an owned view of the polygons.
    pub fn build(self) -> FlattenedPolygons<'a> {
        let (use_filter, layer, datatype) = match self.filter {
            Some(t) => (true, t.layer, t.datatype),
            None => (false, 0, 0),
        };
        let inner = match self.source {
            GetPolygonsSource::Cell(c) => ffi::cell_get_polygons_flat(
                c,
                self.apply_repetitions,
                self.include_paths,
                self.depth,
                use_filter,
                layer,
                datatype,
            ),
            GetPolygonsSource::Reference(r) => ffi::reference_get_polygons_flat(
                r,
                self.apply_repetitions,
                self.include_paths,
                self.depth,
                use_filter,
                layer,
                datatype,
            ),
        };
        FlattenedPolygons {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Owned view of polygons produced by `Cell::get_polygons` or
/// `Reference::get_polygons`. All polygons are heap-allocated, transformed
/// copies and get freed when this value drops.
///
/// Polygon coordinates are **already transformed** — don't apply the source
/// cell's or reference's transformation a second time.
pub struct FlattenedPolygons<'a> {
    inner: cxx::UniquePtr<ffi::FlattenedPolygonsHandle>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> FlattenedPolygons<'a> {
    pub fn count(&self) -> u64 {
        ffi::flattened_polygons_count(&self.inner)
    }

    /// Polygon at index. Borrows from this view — the polygon is freed
    /// when the view drops.
    pub fn polygon(&self, idx: u64) -> Polygon<'_> {
        Polygon {
            handle: ffi::flattened_polygons_at(&self.inner, idx),
        }
    }

    pub fn polygons(&self) -> impl Iterator<Item = Polygon<'_>> + '_ {
        (0..self.count()).map(move |i| self.polygon(i))
    }
}

// ---- Directional XOR helpers ----

#[derive(Clone, Copy)]
enum SplitSide {
    Added,
    Removed,
}

fn collect_split_polys(
    h: &cxx::UniquePtr<ffi::XorSplitHandle>,
    side: SplitSide,
) -> Vec<OwnedPolygon> {
    let n = match side {
        SplitSide::Added => ffi::xor_split_added_count(h),
        SplitSide::Removed => ffi::xor_split_removed_count(h),
    };
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let (layer, datatype, point_count) = match side {
            SplitSide::Added => (
                ffi::xor_split_added_layer(h, i),
                ffi::xor_split_added_datatype(h, i),
                ffi::xor_split_added_point_count(h, i),
            ),
            SplitSide::Removed => (
                ffi::xor_split_removed_layer(h, i),
                ffi::xor_split_removed_datatype(h, i),
                ffi::xor_split_removed_point_count(h, i),
            ),
        };
        let mut points = Vec::with_capacity(point_count as usize);
        for j in 0..point_count {
            let p = match side {
                SplitSide::Added => ffi::xor_split_added_point(h, i, j),
                SplitSide::Removed => ffi::xor_split_removed_point(h, i, j),
            };
            points.push(p);
        }
        out.push(OwnedPolygon {
            layer,
            datatype,
            points,
        });
    }
    out
}

impl Library {
    /// Distinct (layer, datatype) tags found in the library's direct
    /// polygons. Sorted ascending by `(layer, datatype)`. Cached after
    /// the first call.
    ///
    /// Path elements (FlexPath / RobustPath) are not polygonized — only
    /// pre-existing polygons contribute. This is intentionally fast for
    /// layer discovery before iterating with `xor_polygons_split`.
    pub fn layers(&self) -> Vec<GdsTag> {
        let n = ffi::library_tag_count(&self.inner);
        (0..n).map(|i| ffi::library_tag_at(&self.inner, i)).collect()
    }
}

// ---- Back-compat with Fase 1 examples ----

pub fn read_gds(path: &str) -> cxx::UniquePtr<ffi::LibraryHandle> {
    ffi::read_gds_shim(path)
}

pub fn cell_count(handle: &ffi::LibraryHandle) -> u64 {
    ffi::library_cell_count(handle)
}
