#ifndef GDSTK_RS_SHIMS_H
#define GDSTK_RS_SHIMS_H
// Text include guard (not #pragma once) because cxx-build copies this file
// into target/.../cxxbridge/crate/gdstk-rs/src/shims.h, producing two
// distinct paths. #pragma once deduplicates by path identity; an #ifndef
// guard deduplicates by macro definition and works across path copies.

#include <cstdint>
#include <memory>

#include "rust/cxx.h"

// Forward-declare shared structs from the cxx bridge.
namespace gdstk_shim {
struct BoundingBox;
struct Point2D;
struct XorMetrics;
struct GdsTag;
}  // namespace gdstk_shim

namespace gdstk_shim {

// ---- Opaque handles ----
struct LibraryHandle {
    struct Impl;
    std::unique_ptr<Impl> impl;

    LibraryHandle();
    ~LibraryHandle();

    LibraryHandle(const LibraryHandle&) = delete;
    LibraryHandle& operator=(const LibraryHandle&) = delete;
};

struct CellHandle;
struct PolygonHandle;
struct LabelHandle;
struct ReferenceHandle;
struct FlexPathHandle;
struct RobustPathHandle;
struct RawCellHandle;
struct RepetitionHandle;

// PIMPL-style handle owning an Array<Polygon*> with heap-allocated polygons
// produced by Cell::get_polygons / Reference::get_polygons. The destructor
// frees every polygon individually before releasing the array.
struct FlattenedPolygonsHandle {
    struct Impl;
    std::unique_ptr<Impl> impl;
    FlattenedPolygonsHandle();
    ~FlattenedPolygonsHandle();
    FlattenedPolygonsHandle(const FlattenedPolygonsHandle&) = delete;
    FlattenedPolygonsHandle& operator=(const FlattenedPolygonsHandle&) = delete;
};

// PIMPL-style handles: cxx requires full definition visible in the header
// for UniquePtr destructor generation. Impl lives in shims.cpp.
struct TopLevelView {
    struct Impl;
    std::unique_ptr<Impl> impl;
    TopLevelView();
    ~TopLevelView();
    TopLevelView(const TopLevelView&) = delete;
    TopLevelView& operator=(const TopLevelView&) = delete;
};

struct GdsInfoHandle {
    struct Impl;
    std::unique_ptr<Impl> impl;
    GdsInfoHandle();
    ~GdsInfoHandle();
    GdsInfoHandle(const GdsInfoHandle&) = delete;
    GdsInfoHandle& operator=(const GdsInfoHandle&) = delete;
};

// PIMPL handle owning the polygons of a directional XOR diff
// (added = in B \ A, removed = in A \ B). Polygons are stored as flat
// arrays of points (in layout units) so the Rust side can copy them
// into OwnedPolygon without crossing the cxx boundary one polygon at a
// time per gdstk::Polygon allocation.
struct XorSplitHandle {
    struct Impl;
    std::unique_ptr<Impl> impl;
    XorSplitHandle();
    ~XorSplitHandle();
    XorSplitHandle(const XorSplitHandle&) = delete;
    XorSplitHandle& operator=(const XorSplitHandle&) = delete;
};

// ---- Library ----
std::unique_ptr<LibraryHandle> read_gds_shim(rust::Str filename);

// Same as read_gds_shim but exposes the parser's error code via out_error.
// Returns null on error; caller decides how to surface the failure.
// Mirrors the pattern of gds_info_read.
std::unique_ptr<LibraryHandle> read_gds_with_error(rust::Str filename, uint8_t& out_error);

uint64_t library_cell_count(const LibraryHandle& handle);
const CellHandle& library_cell_at(const LibraryHandle& handle, uint64_t idx);
// Fase 6 — metadata + write + top_level
rust::Str library_name(const LibraryHandle& handle);
double library_unit(const LibraryHandle& handle);
double library_precision(const LibraryHandle& handle);
// Returns gdstk::ErrorCode cast to uint8_t (0 = NoError).
uint8_t library_write_gds(const LibraryHandle& handle, rust::Str path);

std::unique_ptr<TopLevelView> library_top_level(const LibraryHandle& handle);
uint64_t top_level_count(const TopLevelView& view);
const CellHandle& top_level_at(const TopLevelView& view, uint64_t idx);

// ---- GdsInfo (fast metadata peek) ----
// Loads info via gdstk::gds_info. On error, sets *out_error and returns
// a null unique_ptr.
std::unique_ptr<GdsInfoHandle> gds_info_read(rust::Str path, uint8_t& out_error);

double gds_info_unit(const GdsInfoHandle& h);
double gds_info_precision(const GdsInfoHandle& h);
uint64_t gds_info_num_polygons(const GdsInfoHandle& h);
uint64_t gds_info_num_paths(const GdsInfoHandle& h);
uint64_t gds_info_num_references(const GdsInfoHandle& h);
uint64_t gds_info_num_labels(const GdsInfoHandle& h);

uint64_t gds_info_cell_count(const GdsInfoHandle& h);
rust::Str gds_info_cell_name(const GdsInfoHandle& h, uint64_t idx);

uint64_t gds_info_shape_tag_count(const GdsInfoHandle& h);
GdsTag gds_info_shape_tag(const GdsInfoHandle& h, uint64_t idx);
uint64_t gds_info_label_tag_count(const GdsInfoHandle& h);
GdsTag gds_info_label_tag(const GdsInfoHandle& h, uint64_t idx);

// ---- Cell ----
rust::Str cell_name(const CellHandle& cell);
uint64_t cell_polygon_count(const CellHandle& cell);
const PolygonHandle& cell_polygon_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_label_count(const CellHandle& cell);
const LabelHandle& cell_label_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_reference_count(const CellHandle& cell);
const ReferenceHandle& cell_reference_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_flexpath_count(const CellHandle& cell);
const FlexPathHandle& cell_flexpath_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_robustpath_count(const CellHandle& cell);
const RobustPathHandle& cell_robustpath_at(const CellHandle& cell, uint64_t idx);

// ---- Polygon ----
double polygon_area(const PolygonHandle& poly);
uint32_t polygon_layer(const PolygonHandle& poly);
uint32_t polygon_datatype(const PolygonHandle& poly);
BoundingBox polygon_bbox(const PolygonHandle& poly);
uint64_t polygon_point_count(const PolygonHandle& poly);
// Individual vertex access. Returns (0,0) for idx >= point_count.
Point2D polygon_point_at(const PolygonHandle& poly, uint64_t idx);

// ---- Label ----
rust::Slice<const uint8_t> label_text_bytes(const LabelHandle& label);
uint32_t label_layer(const LabelHandle& label);
uint32_t label_texttype(const LabelHandle& label);
Point2D label_origin(const LabelHandle& label);
uint8_t label_anchor(const LabelHandle& label);
double label_rotation(const LabelHandle& label);
double label_magnification(const LabelHandle& label);
bool label_x_reflection(const LabelHandle& label);

// ---- Reference ----
rust::Str reference_cell_name(const ReferenceHandle& ref);
Point2D reference_origin(const ReferenceHandle& ref);
double reference_rotation(const ReferenceHandle& ref);
double reference_magnification(const ReferenceHandle& ref);
bool reference_x_reflection(const ReferenceHandle& ref);

// ---- FlexPath ----
uint64_t flexpath_num_elements(const FlexPathHandle& path);
uint32_t flexpath_element_layer(const FlexPathHandle& path, uint64_t element_idx);
uint32_t flexpath_element_datatype(const FlexPathHandle& path, uint64_t element_idx);
uint64_t flexpath_spine_point_count(const FlexPathHandle& path);
// Fase 5.5 — extended accessors
Point2D flexpath_spine_point(const FlexPathHandle& path, uint64_t point_idx);
double flexpath_element_half_width(const FlexPathHandle& path, uint64_t element_idx,
                                   uint64_t spine_idx);
double flexpath_element_offset(const FlexPathHandle& path, uint64_t element_idx,
                               uint64_t spine_idx);
// Enums (end/join/bend) returned as uint8_t using gdstk::EndType etc. casts.
uint8_t flexpath_element_end_type(const FlexPathHandle& path, uint64_t element_idx);
uint8_t flexpath_element_join_type(const FlexPathHandle& path, uint64_t element_idx);
uint8_t flexpath_element_bend_type(const FlexPathHandle& path, uint64_t element_idx);
double flexpath_element_bend_radius(const FlexPathHandle& path, uint64_t element_idx);
Point2D flexpath_element_end_extensions(const FlexPathHandle& path, uint64_t element_idx);
bool flexpath_simple_path(const FlexPathHandle& path);
bool flexpath_scale_width(const FlexPathHandle& path);

// ---- RobustPath ----
uint64_t robustpath_num_elements(const RobustPathHandle& path);
uint32_t robustpath_element_layer(const RobustPathHandle& path, uint64_t element_idx);
uint32_t robustpath_element_datatype(const RobustPathHandle& path, uint64_t element_idx);
// Fase 5.5 — extended accessors
uint64_t robustpath_subpath_count(const RobustPathHandle& path);
Point2D robustpath_end_point(const RobustPathHandle& path);
double robustpath_tolerance(const RobustPathHandle& path);
uint64_t robustpath_max_evals(const RobustPathHandle& path);
double robustpath_element_end_width(const RobustPathHandle& path, uint64_t element_idx);
double robustpath_element_end_offset(const RobustPathHandle& path, uint64_t element_idx);
uint8_t robustpath_element_end_type(const RobustPathHandle& path, uint64_t element_idx);
bool robustpath_simple_path(const RobustPathHandle& path);
bool robustpath_scale_width(const RobustPathHandle& path);

// ---- Fase 8: gap-closing read accessors ----
// Polygon extended
double polygon_perimeter(const PolygonHandle& poly);
double polygon_signed_area(const PolygonHandle& poly);

// Repetition on Polygon/Label/Reference. count includes origin (>=1).
uint64_t polygon_repetition_count(const PolygonHandle& poly);
Point2D polygon_repetition_offset(const PolygonHandle& poly, uint64_t idx);
uint64_t label_repetition_count(const LabelHandle& label);
Point2D label_repetition_offset(const LabelHandle& label, uint64_t idx);
uint64_t reference_repetition_count(const ReferenceHandle& ref);
Point2D reference_repetition_offset(const ReferenceHandle& ref, uint64_t idx);

// Bounding boxes
BoundingBox cell_bbox(const CellHandle& cell);
BoundingBox reference_bbox(const ReferenceHandle& ref);

// RawCell iteration (Library-owned raw cells; not parsed)
uint64_t library_rawcell_count(const LibraryHandle& handle);
const RawCellHandle& library_rawcell_at(const LibraryHandle& handle, uint64_t idx);

rust::Str rawcell_name(const RawCellHandle& rc);
uint64_t rawcell_size(const RawCellHandle& rc);
uint64_t rawcell_dependency_count(const RawCellHandle& rc);
const RawCellHandle& rawcell_dependency_at(const RawCellHandle& rc, uint64_t idx);

// ---- Fase 8.5: Repetition struct detail ----
// Direct access to a Polygon/Label/Reference's Repetition field.
const RepetitionHandle& polygon_repetition(const PolygonHandle& poly);
const RepetitionHandle& label_repetition(const LabelHandle& label);
const RepetitionHandle& reference_repetition(const ReferenceHandle& ref);

// Repetition accessors (variant-specific fields).
// type: 0=None, 1=Rectangular, 2=Regular, 3=Explicit, 4=ExplicitX, 5=ExplicitY
uint8_t repetition_type(const RepetitionHandle& rep);
// Effective count (1 when type=None, >=1 otherwise).
uint64_t repetition_count(const RepetitionHandle& rep);

// Rectangular / Regular share columns/rows.
uint64_t repetition_columns(const RepetitionHandle& rep);
uint64_t repetition_rows(const RepetitionHandle& rep);

// Rectangular-only: spacing
Point2D repetition_spacing(const RepetitionHandle& rep);

// Regular-only: v1, v2 axes
Point2D repetition_v1(const RepetitionHandle& rep);
Point2D repetition_v2(const RepetitionHandle& rep);

// Note: the raw Explicit offset array (without origin) is intentionally
// NOT exposed separately — `repetition_generated_offset` already covers
// all kinds with origin included. Users can skip(1) to get the raw list.

// ExplicitX/ExplicitY: 1D coord array.
uint64_t repetition_coord_count(const RepetitionHandle& rep);
double repetition_coord(const RepetitionHandle& rep, uint64_t idx);

// Get all generated offsets (including origin (0,0)) via idx-based access.
Point2D repetition_generated_offset(const RepetitionHandle& rep, uint64_t idx);

// Extrema: 1-4 boundary points of the repetition pattern (incl. origin).
// Much cheaper than iterating all generated offsets for bbox queries.
// For RepetitionType::None, returns 1 extremum at origin.
uint64_t repetition_extrema_count(const RepetitionHandle& rep);
Point2D repetition_extremum(const RepetitionHandle& rep, uint64_t idx);

// ---- Flatten: Cell::get_polygons / Reference::get_polygons ----
// Returns a heap-owned view of polygons with transformations applied
// (rotation, magnification, x_reflection, origin, repetitions).
// `use_filter=false` ignores `layer`/`datatype`.
// `depth=-1` means unlimited recursion.
std::unique_ptr<FlattenedPolygonsHandle> cell_get_polygons_flat(
    const CellHandle& cell,
    bool apply_repetitions,
    bool include_paths,
    int64_t depth,
    bool use_filter,
    uint32_t layer,
    uint32_t datatype);

std::unique_ptr<FlattenedPolygonsHandle> reference_get_polygons_flat(
    const ReferenceHandle& ref,
    bool apply_repetitions,
    bool include_paths,
    int64_t depth,
    bool use_filter,
    uint32_t layer,
    uint32_t datatype);

uint64_t flattened_polygons_count(const FlattenedPolygonsHandle& view);
const PolygonHandle& flattened_polygons_at(
    const FlattenedPolygonsHandle& view, uint64_t idx);

// ---- Boolean XOR ----
// Canonical XOR: includes polygons + paths (converted to polygons internally).
// Correct choice for Miku diff — without this, GDS files with paths (common
// for wires/interconnects) have incomplete diffs.
XorMetrics cell_xor_with(const CellHandle& a, const CellHandle& b, uint32_t layer);

// Legacy XOR that ignores paths. Kept for compatibility / special cases where
// you only care about polygon-level diffs.
XorMetrics cell_xor_with_polygons_only(const CellHandle& a, const CellHandle& b,
                                       uint32_t layer);

// Directional XOR: returns polygons of (added = B\A) and (removed = A\B)
// separately, including path-derived polygons. Lets callers paint added
// vs removed differently in a diff visualization.
std::unique_ptr<XorSplitHandle> cell_xor_polygons_split(
    const CellHandle& a, const CellHandle& b, uint32_t layer);

uint64_t xor_split_added_count(const XorSplitHandle& h);
uint64_t xor_split_removed_count(const XorSplitHandle& h);

uint32_t xor_split_added_layer(const XorSplitHandle& h, uint64_t poly_idx);
uint32_t xor_split_added_datatype(const XorSplitHandle& h, uint64_t poly_idx);
uint64_t xor_split_added_point_count(const XorSplitHandle& h, uint64_t poly_idx);
Point2D xor_split_added_point(const XorSplitHandle& h, uint64_t poly_idx,
                              uint64_t point_idx);

uint32_t xor_split_removed_layer(const XorSplitHandle& h, uint64_t poly_idx);
uint32_t xor_split_removed_datatype(const XorSplitHandle& h, uint64_t poly_idx);
uint64_t xor_split_removed_point_count(const XorSplitHandle& h, uint64_t poly_idx);
Point2D xor_split_removed_point(const XorSplitHandle& h, uint64_t poly_idx,
                                uint64_t point_idx);

// Distinct (layer, datatype) tags present in the library's polygon arrays.
// Sorted ascending; cached on first call. Iterates only direct polygons
// (paths are not polygonized — fast path discovery).
uint64_t library_tag_count(const LibraryHandle& handle);
GdsTag library_tag_at(const LibraryHandle& handle, uint64_t idx);

}  // namespace gdstk_shim

#endif  // GDSTK_RS_SHIMS_H
