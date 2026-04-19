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
// Full definitions come from the auto-generated lib.rs.h (included in
// shims.cpp, not here, to avoid circular include issues).
namespace gdstk_shim {
struct BoundingBox;
struct Point2D;
struct XorMetrics;
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

// ---- Library ----
std::unique_ptr<LibraryHandle> read_gds_shim(rust::Str filename);
uint64_t library_cell_count(const LibraryHandle& handle);
const CellHandle& library_cell_at(const LibraryHandle& handle, uint64_t idx);

// ---- Cell ----
rust::Str cell_name(const CellHandle& cell);
uint64_t cell_polygon_count(const CellHandle& cell);
const PolygonHandle& cell_polygon_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_label_count(const CellHandle& cell);
const LabelHandle& cell_label_at(const CellHandle& cell, uint64_t idx);
uint64_t cell_reference_count(const CellHandle& cell);
const ReferenceHandle& cell_reference_at(const CellHandle& cell, uint64_t idx);

// ---- Polygon ----
double polygon_area(const PolygonHandle& poly);
uint32_t polygon_layer(const PolygonHandle& poly);
uint32_t polygon_datatype(const PolygonHandle& poly);
BoundingBox polygon_bbox(const PolygonHandle& poly);
uint64_t polygon_point_count(const PolygonHandle& poly);

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
// Name of the referenced cell. Reads from ref->cell->name, ref->rawcell->name,
// or ref->name depending on ref->type. Returns empty string if NULL.
rust::Str reference_cell_name(const ReferenceHandle& ref);
Point2D reference_origin(const ReferenceHandle& ref);
double reference_rotation(const ReferenceHandle& ref);  // radians
double reference_magnification(const ReferenceHandle& ref);
bool reference_x_reflection(const ReferenceHandle& ref);

// ---- Boolean XOR ----
// Filters polygons of `a` and `b` by layer (matching get_layer(tag)), runs
// gdstk::boolean(... Operation::Xor ...), aggregates area + region count +
// bbox, frees the result polygons, returns XorMetrics.
// scaling=1000.0 → Clipper precision of 1e-3 µm (suitable for GDS).
XorMetrics cell_xor_with(const CellHandle& a, const CellHandle& b, uint32_t layer);

}  // namespace gdstk_shim

#endif  // GDSTK_RS_SHIMS_H
