#include "shims.h"

#include <cstring>
#include <string>

#include "gdstk/gdstk.hpp"

// Auto-generated header from cxx-build containing the shared struct
// BoundingBox that both Rust and C++ agree on.
#include "gdstk-rs/src/lib.rs.h"

namespace gdstk_shim {

// PIMPL: the actual gdstk::Library lives here, isolated from shims.h.
struct LibraryHandle::Impl {
    gdstk::Library lib;

    Impl() {
        lib.name = nullptr;
        lib.unit = 0;
        lib.precision = 0;
        lib.cell_array = {};
        lib.rawcell_array = {};
        lib.layer_names = {};
        lib.properties = nullptr;
        lib.owner = nullptr;
    }

    ~Impl() {
        lib.free_all();
    }
};

LibraryHandle::LibraryHandle() : impl(std::make_unique<Impl>()) {}
LibraryHandle::~LibraryHandle() = default;

// CellHandle / PolygonHandle are empty marker structs. We never construct
// them directly; we reinterpret_cast gdstk::Cell* / gdstk::Polygon* into
// them. The cxx bridge sees them as opaque types with only references.
struct CellHandle {};
struct PolygonHandle {};

// Helper conversions between opaque handle refs and gdstk pointers.
static inline const gdstk::Cell* as_cell(const CellHandle& h) {
    return reinterpret_cast<const gdstk::Cell*>(&h);
}
static inline const gdstk::Polygon* as_polygon(const PolygonHandle& h) {
    return reinterpret_cast<const gdstk::Polygon*>(&h);
}
static inline const CellHandle& from_cell(const gdstk::Cell* c) {
    return *reinterpret_cast<const CellHandle*>(c);
}
static inline const PolygonHandle& from_polygon(const gdstk::Polygon* p) {
    return *reinterpret_cast<const PolygonHandle*>(p);
}

// ---- Library ----

std::unique_ptr<LibraryHandle> read_gds_shim(rust::Str filename) {
    auto handle = std::make_unique<LibraryHandle>();
    std::string path(filename.data(), filename.size());

    gdstk::ErrorCode error_code = gdstk::ErrorCode::NoError;
    handle->impl->lib = gdstk::read_gds(path.c_str(),
                                        /*unit=*/0.0,
                                        /*tolerance=*/0.0,
                                        /*shape_tags=*/nullptr,
                                        &error_code);
    (void)error_code;
    return handle;
}

uint64_t library_cell_count(const LibraryHandle& handle) {
    return handle.impl->lib.cell_array.count;
}

const CellHandle& library_cell_at(const LibraryHandle& handle, uint64_t idx) {
    // cell_array stores Cell* — dereference to get a gdstk::Cell we can cast.
    const gdstk::Cell* cell = handle.impl->lib.cell_array[idx];
    return from_cell(cell);
}

// ---- Cell ----

rust::Str cell_name(const CellHandle& cell) {
    const char* name = as_cell(cell)->name;
    if (name == nullptr) {
        return rust::Str("");
    }
    // rust::Str takes a (ptr, len). Length is strlen of the null-terminated name.
    return rust::Str(name, std::strlen(name));
}

uint64_t cell_polygon_count(const CellHandle& cell) {
    return as_cell(cell)->polygon_array.count;
}

const PolygonHandle& cell_polygon_at(const CellHandle& cell, uint64_t idx) {
    const gdstk::Polygon* poly = as_cell(cell)->polygon_array[idx];
    return from_polygon(poly);
}

// ---- Polygon ----

double polygon_area(const PolygonHandle& poly) {
    return as_polygon(poly)->area();
}

uint32_t polygon_layer(const PolygonHandle& poly) {
    return gdstk::get_layer(as_polygon(poly)->tag);
}

uint32_t polygon_datatype(const PolygonHandle& poly) {
    return gdstk::get_type(as_polygon(poly)->tag);
}

BoundingBox polygon_bbox(const PolygonHandle& poly) {
    gdstk::Vec2 min{}, max{};
    as_polygon(poly)->bounding_box(min, max);
    return BoundingBox{min.x, min.y, max.x, max.y};
}

uint64_t polygon_point_count(const PolygonHandle& poly) {
    return as_polygon(poly)->point_array.count;
}

}  // namespace gdstk_shim
