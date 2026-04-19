#include "shims.h"

#include <cstring>
#include <limits>
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

// Cell/Polygon/Label/Reference handles are empty marker structs. We never
// construct them directly; we reinterpret_cast gdstk::* into them. The cxx
// bridge sees them as opaque types accessible only via references.
struct CellHandle {};
struct PolygonHandle {};
struct LabelHandle {};
struct ReferenceHandle {};

// Helper conversions between opaque handle refs and gdstk pointers.
static inline const gdstk::Cell* as_cell(const CellHandle& h) {
    return reinterpret_cast<const gdstk::Cell*>(&h);
}
static inline const gdstk::Polygon* as_polygon(const PolygonHandle& h) {
    return reinterpret_cast<const gdstk::Polygon*>(&h);
}
static inline const gdstk::Label* as_label(const LabelHandle& h) {
    return reinterpret_cast<const gdstk::Label*>(&h);
}
static inline const gdstk::Reference* as_reference(const ReferenceHandle& h) {
    return reinterpret_cast<const gdstk::Reference*>(&h);
}
static inline const CellHandle& from_cell(const gdstk::Cell* c) {
    return *reinterpret_cast<const CellHandle*>(c);
}
static inline const PolygonHandle& from_polygon(const gdstk::Polygon* p) {
    return *reinterpret_cast<const PolygonHandle*>(p);
}
static inline const LabelHandle& from_label(const gdstk::Label* l) {
    return *reinterpret_cast<const LabelHandle*>(l);
}
static inline const ReferenceHandle& from_reference(const gdstk::Reference* r) {
    return *reinterpret_cast<const ReferenceHandle*>(r);
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

uint64_t cell_label_count(const CellHandle& cell) {
    return as_cell(cell)->label_array.count;
}

const LabelHandle& cell_label_at(const CellHandle& cell, uint64_t idx) {
    const gdstk::Label* label = as_cell(cell)->label_array[idx];
    return from_label(label);
}

uint64_t cell_reference_count(const CellHandle& cell) {
    return as_cell(cell)->reference_array.count;
}

const ReferenceHandle& cell_reference_at(const CellHandle& cell, uint64_t idx) {
    const gdstk::Reference* ref = as_cell(cell)->reference_array[idx];
    return from_reference(ref);
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

// ---- Label ----

rust::Slice<const uint8_t> label_text_bytes(const LabelHandle& label) {
    const gdstk::Label* lbl = as_label(label);
    if (lbl->text == nullptr) {
        return rust::Slice<const uint8_t>(static_cast<const uint8_t*>(nullptr), 0);
    }
    const uint8_t* ptr = reinterpret_cast<const uint8_t*>(lbl->text);
    return rust::Slice<const uint8_t>(ptr, std::strlen(lbl->text));
}

uint32_t label_layer(const LabelHandle& label) {
    return gdstk::get_layer(as_label(label)->tag);
}

uint32_t label_texttype(const LabelHandle& label) {
    return gdstk::get_type(as_label(label)->tag);
}

Point2D label_origin(const LabelHandle& label) {
    const gdstk::Vec2& o = as_label(label)->origin;
    return Point2D{o.x, o.y};
}

uint8_t label_anchor(const LabelHandle& label) {
    // gdstk::Anchor uses sparse values: NW=0, N=1, NE=2, W=4, O=5, E=6,
    // SW=8, S=9, SE=10. Values fit in u8; we preserve them verbatim.
    return static_cast<uint8_t>(as_label(label)->anchor);
}

double label_rotation(const LabelHandle& label) {
    return as_label(label)->rotation;
}

double label_magnification(const LabelHandle& label) {
    return as_label(label)->magnification;
}

bool label_x_reflection(const LabelHandle& label) {
    return as_label(label)->x_reflection;
}

// ---- Reference ----

rust::Str reference_cell_name(const ReferenceHandle& ref) {
    const gdstk::Reference* r = as_reference(ref);
    const char* name = nullptr;
    switch (r->type) {
        case gdstk::ReferenceType::Cell:
            if (r->cell != nullptr) name = r->cell->name;
            break;
        case gdstk::ReferenceType::RawCell:
            if (r->rawcell != nullptr) name = r->rawcell->name;
            break;
        case gdstk::ReferenceType::Name:
            name = r->name;
            break;
    }
    if (name == nullptr) {
        return rust::Str("");
    }
    return rust::Str(name, std::strlen(name));
}

Point2D reference_origin(const ReferenceHandle& ref) {
    const gdstk::Vec2& o = as_reference(ref)->origin;
    return Point2D{o.x, o.y};
}

double reference_rotation(const ReferenceHandle& ref) {
    return as_reference(ref)->rotation;
}

double reference_magnification(const ReferenceHandle& ref) {
    return as_reference(ref)->magnification;
}

bool reference_x_reflection(const ReferenceHandle& ref) {
    return as_reference(ref)->x_reflection;
}

// ---- Boolean XOR ----

XorMetrics cell_xor_with(const CellHandle& a, const CellHandle& b, uint32_t layer) {
    const gdstk::Cell* cell_a = as_cell(a);
    const gdstk::Cell* cell_b = as_cell(b);

    // Collect pointers to polygons in `a` with matching layer. We do NOT own
    // these — they belong to the parent Library. filtered_* only holds views.
    gdstk::Array<gdstk::Polygon*> filtered_a = {};
    gdstk::Array<gdstk::Polygon*> filtered_b = {};

    for (uint64_t i = 0; i < cell_a->polygon_array.count; i++) {
        gdstk::Polygon* p = cell_a->polygon_array[i];
        if (gdstk::get_layer(p->tag) == layer) {
            filtered_a.append(p);
        }
    }
    for (uint64_t i = 0; i < cell_b->polygon_array.count; i++) {
        gdstk::Polygon* p = cell_b->polygon_array[i];
        if (gdstk::get_layer(p->tag) == layer) {
            filtered_b.append(p);
        }
    }

    XorMetrics metrics{0.0, 0, BoundingBox{0.0, 0.0, 0.0, 0.0}};

    // Fast path: empty layer on both sides → no diff.
    if (filtered_a.count == 0 && filtered_b.count == 0) {
        filtered_a.clear();
        filtered_b.clear();
        return metrics;
    }

    // Run XOR. scaling=1000.0 gives Clipper precision of 1e-3 µm,
    // adequate for typical GDS unit ranges (nm-scale layouts).
    gdstk::Array<gdstk::Polygon*> result = {};
    gdstk::ErrorCode err = gdstk::boolean(filtered_a, filtered_b,
                                          gdstk::Operation::Xor,
                                          /*scaling=*/1000.0, result);
    (void)err;  // Swallow error for Fase 4; future phases expose it.

    // Aggregate metrics across result polygons.
    double total_area = 0.0;
    double bbox_min_x = std::numeric_limits<double>::infinity();
    double bbox_min_y = std::numeric_limits<double>::infinity();
    double bbox_max_x = -std::numeric_limits<double>::infinity();
    double bbox_max_y = -std::numeric_limits<double>::infinity();

    for (uint64_t i = 0; i < result.count; i++) {
        gdstk::Polygon* poly = result[i];
        total_area += poly->area();

        gdstk::Vec2 p_min{}, p_max{};
        poly->bounding_box(p_min, p_max);
        if (p_min.x < bbox_min_x) bbox_min_x = p_min.x;
        if (p_min.y < bbox_min_y) bbox_min_y = p_min.y;
        if (p_max.x > bbox_max_x) bbox_max_x = p_max.x;
        if (p_max.y > bbox_max_y) bbox_max_y = p_max.y;
    }

    metrics.area = total_area;
    metrics.region_count = result.count;
    if (result.count > 0) {
        metrics.bbox = BoundingBox{bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y};
    }

    // Free all polygons allocated by boolean(). Each must be cleared then
    // deallocated — see gdstk::Cell::free_all for the canonical pattern.
    for (uint64_t i = 0; i < result.count; i++) {
        result[i]->clear();
        gdstk::free_allocation(result[i]);
    }
    result.clear();

    // filtered_a / filtered_b only hold pointers to Library-owned polygons,
    // so we only release the array capacity, not the polygons themselves.
    filtered_a.clear();
    filtered_b.clear();

    return metrics;
}

}  // namespace gdstk_shim
