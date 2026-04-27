#include "shims.h"

#include <cstring>
#include <limits>
#include <set>
#include <string>
#include <vector>

#include "gdstk/gdstk.hpp"

// Auto-generated header from cxx-build containing the shared struct
// BoundingBox that both Rust and C++ agree on.
#include "gdstk-rs/src/lib.rs.h"

namespace gdstk_shim {

// PIMPL: the actual gdstk::Library lives here, isolated from shims.h.
struct LibraryHandle::Impl {
    gdstk::Library lib;
    // Lazy cache for Library::layers(). Computed on first call to
    // library_tag_count/at; never invalidated (Library is read-only after
    // open()). `mutable` because the public shim functions take a
    // const LibraryHandle& reference.
    mutable std::vector<gdstk::Tag> cached_tags;
    mutable bool tags_cached = false;

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

// ---- TopLevelView PIMPL ----
// Holds pointers to Cells that are top-level in the Library. The Cells
// themselves are owned by the Library; this view only aggregates references.
struct TopLevelView::Impl {
    gdstk::Array<gdstk::Cell*> cells;
    Impl() : cells({}) {}
    ~Impl() { cells.clear(); }
};
TopLevelView::TopLevelView() : impl(std::make_unique<Impl>()) {}
TopLevelView::~TopLevelView() = default;

// ---- GdsInfoHandle PIMPL ----
// Owns a gdstk::LibraryInfo plus sorted Arrays<Tag> for deterministic
// index-based access to shape_tags and label_tags (since gdstk::Set<Tag>
// does not expose operator[]).
struct GdsInfoHandle::Impl {
    gdstk::LibraryInfo info;
    gdstk::Array<gdstk::Tag> sorted_shape_tags;
    gdstk::Array<gdstk::Tag> sorted_label_tags;

    Impl() {
        info.cell_names = {};
        info.shape_tags = {};
        info.label_tags = {};
        info.num_polygons = 0;
        info.num_paths = 0;
        info.num_references = 0;
        info.num_labels = 0;
        info.unit = 0;
        info.precision = 0;
        sorted_shape_tags = {};
        sorted_label_tags = {};
    }

    ~Impl() {
        info.clear();
        sorted_shape_tags.clear();
        sorted_label_tags.clear();
    }
};
GdsInfoHandle::GdsInfoHandle() : impl(std::make_unique<Impl>()) {}
GdsInfoHandle::~GdsInfoHandle() = default;

// ---- FlattenedPolygonsHandle PIMPL ----
// Owns an Array<Polygon*> with heap-allocated polygons. Destructor
// iterates every polygon and frees it individually before releasing
// the Array capacity.
struct FlattenedPolygonsHandle::Impl {
    gdstk::Array<gdstk::Polygon*> polygons;

    Impl() : polygons({}) {}

    ~Impl() {
        for (uint64_t i = 0; i < polygons.count; i++) {
            if (polygons[i] != nullptr) {
                polygons[i]->clear();
                gdstk::free_allocation(polygons[i]);
            }
        }
        polygons.clear();
    }
};
FlattenedPolygonsHandle::FlattenedPolygonsHandle()
    : impl(std::make_unique<Impl>()) {}
FlattenedPolygonsHandle::~FlattenedPolygonsHandle() = default;

// ---- XorSplitHandle PIMPL ----
// Stores added/removed polygons as flat (layer, datatype, points) records.
// We don't keep gdstk::Polygon* alive: the boolean() result polygons are
// freed inside cell_xor_polygons_split, with their geometry copied into
// these vectors first. Avoids leaking gdstk allocations across the cxx
// boundary and keeps the Rust side allocation-free of C++ types.
struct XorSplitHandle::Impl {
    struct OwnedPoly {
        uint32_t layer;
        uint32_t datatype;
        std::vector<Point2D> points;
    };
    std::vector<OwnedPoly> added;
    std::vector<OwnedPoly> removed;
};
XorSplitHandle::XorSplitHandle() : impl(std::make_unique<Impl>()) {}
XorSplitHandle::~XorSplitHandle() = default;

// All handle types are empty marker structs. We never construct them
// directly; we reinterpret_cast gdstk::* into them. The cxx bridge sees
// them as opaque types accessible only via references.
struct CellHandle {};
struct PolygonHandle {};
struct LabelHandle {};
struct ReferenceHandle {};
struct FlexPathHandle {};
struct RobustPathHandle {};
struct RawCellHandle {};
struct RepetitionHandle {};

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
static inline gdstk::FlexPath* as_flexpath_mut(const FlexPathHandle& h) {
    // to_polygons is not const-qualified on FlexPath, so we need mutable.
    // Safe: handle is only obtained from cell's flexpath_array, and the
    // underlying FlexPath is owned by the Library (not aliased).
    return reinterpret_cast<gdstk::FlexPath*>(const_cast<FlexPathHandle*>(&h));
}
static inline const gdstk::FlexPath* as_flexpath(const FlexPathHandle& h) {
    return reinterpret_cast<const gdstk::FlexPath*>(&h);
}
static inline const gdstk::RobustPath* as_robustpath(const RobustPathHandle& h) {
    return reinterpret_cast<const gdstk::RobustPath*>(&h);
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
static inline const FlexPathHandle& from_flexpath(const gdstk::FlexPath* fp) {
    return *reinterpret_cast<const FlexPathHandle*>(fp);
}
static inline const RobustPathHandle& from_robustpath(const gdstk::RobustPath* rp) {
    return *reinterpret_cast<const RobustPathHandle*>(rp);
}
static inline const gdstk::RawCell* as_rawcell(const RawCellHandle& h) {
    return reinterpret_cast<const gdstk::RawCell*>(&h);
}
static inline const RawCellHandle& from_rawcell(const gdstk::RawCell* rc) {
    return *reinterpret_cast<const RawCellHandle*>(rc);
}
static inline const gdstk::Repetition* as_repetition(const RepetitionHandle& h) {
    return reinterpret_cast<const gdstk::Repetition*>(&h);
}
static inline const RepetitionHandle& from_repetition(const gdstk::Repetition* r) {
    return *reinterpret_cast<const RepetitionHandle*>(r);
}

// Effective count: gdstk returns 0 when there's no repetition
// (RepetitionType::None); we report that as 1 (origin instance itself)
// for a consistent iteration model.
static inline uint64_t repetition_effective_count(const gdstk::Repetition& rep) {
    uint64_t c = rep.get_count();
    return c == 0 ? 1 : c;
}

// Helper: extract the idx-th offset from a gdstk::Repetition.
// If the repetition is None (count==0), returns (0,0) for idx==0 and
// (0,0) otherwise. For explicit/rectangular, uses gdstk's get_offsets.
static inline Point2D repetition_offset_at(const gdstk::Repetition& rep, uint64_t idx) {
    if (rep.get_count() == 0) {
        // No repetition: single instance at origin.
        return Point2D{0.0, 0.0};
    }
    gdstk::Array<gdstk::Vec2> offs = {};
    rep.get_offsets(offs);
    Point2D r{0.0, 0.0};
    if (idx < offs.count) {
        r.x = offs[idx].x;
        r.y = offs[idx].y;
    }
    offs.clear();
    return r;
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

rust::Str library_name(const LibraryHandle& handle) {
    const char* name = handle.impl->lib.name;
    if (name == nullptr) return rust::Str("");
    return rust::Str(name, std::strlen(name));
}

double library_unit(const LibraryHandle& handle) {
    return handle.impl->lib.unit;
}

double library_precision(const LibraryHandle& handle) {
    return handle.impl->lib.precision;
}

uint8_t library_write_gds(const LibraryHandle& handle, rust::Str path) {
    std::string p(path.data(), path.size());
    gdstk::ErrorCode err =
        handle.impl->lib.write_gds(p.c_str(), /*max_points=*/0, /*timestamp=*/nullptr);
    return static_cast<uint8_t>(err);
}

std::unique_ptr<TopLevelView> library_top_level(const LibraryHandle& handle) {
    auto view = std::make_unique<TopLevelView>();
    gdstk::Array<gdstk::RawCell*> rawcells = {};
    handle.impl->lib.top_level(view->impl->cells, rawcells);
    rawcells.clear();  // Miku no soporta RawCell; descartamos.
    return view;
}

uint64_t top_level_count(const TopLevelView& view) {
    return view.impl->cells.count;
}

const CellHandle& top_level_at(const TopLevelView& view, uint64_t idx) {
    return from_cell(view.impl->cells[idx]);
}

// ---- GdsInfo ----

std::unique_ptr<GdsInfoHandle> gds_info_read(rust::Str path, uint8_t& out_error) {
    auto h = std::make_unique<GdsInfoHandle>();
    std::string p(path.data(), path.size());
    gdstk::ErrorCode err = gdstk::gds_info(p.c_str(), h->impl->info);
    out_error = static_cast<uint8_t>(err);
    if (err != gdstk::ErrorCode::NoError) {
        return nullptr;
    }
    // gdstk::Set<Tag>::to_array appends live items to a dynamic array.
    h->impl->info.shape_tags.to_array(h->impl->sorted_shape_tags);
    h->impl->info.label_tags.to_array(h->impl->sorted_label_tags);

    // Sort for deterministic index-based access (Set iteration order
    // depends on hash bucket layout).
    auto tag_cmp = [](const void* a, const void* b) -> int {
        gdstk::Tag ta = *static_cast<const gdstk::Tag*>(a);
        gdstk::Tag tb = *static_cast<const gdstk::Tag*>(b);
        if (ta < tb) return -1;
        if (ta > tb) return 1;
        return 0;
    };
    if (h->impl->sorted_shape_tags.count > 1) {
        std::qsort(h->impl->sorted_shape_tags.items,
                   h->impl->sorted_shape_tags.count,
                   sizeof(gdstk::Tag), tag_cmp);
    }
    if (h->impl->sorted_label_tags.count > 1) {
        std::qsort(h->impl->sorted_label_tags.items,
                   h->impl->sorted_label_tags.count,
                   sizeof(gdstk::Tag), tag_cmp);
    }
    return h;
}

double gds_info_unit(const GdsInfoHandle& h) { return h.impl->info.unit; }
double gds_info_precision(const GdsInfoHandle& h) { return h.impl->info.precision; }
uint64_t gds_info_num_polygons(const GdsInfoHandle& h) { return h.impl->info.num_polygons; }
uint64_t gds_info_num_paths(const GdsInfoHandle& h) { return h.impl->info.num_paths; }
uint64_t gds_info_num_references(const GdsInfoHandle& h) { return h.impl->info.num_references; }
uint64_t gds_info_num_labels(const GdsInfoHandle& h) { return h.impl->info.num_labels; }

uint64_t gds_info_cell_count(const GdsInfoHandle& h) {
    return h.impl->info.cell_names.count;
}

rust::Str gds_info_cell_name(const GdsInfoHandle& h, uint64_t idx) {
    if (idx >= h.impl->info.cell_names.count) return rust::Str("");
    const char* name = h.impl->info.cell_names[idx];
    if (name == nullptr) return rust::Str("");
    return rust::Str(name, std::strlen(name));
}

uint64_t gds_info_shape_tag_count(const GdsInfoHandle& h) {
    return h.impl->sorted_shape_tags.count;
}

GdsTag gds_info_shape_tag(const GdsInfoHandle& h, uint64_t idx) {
    if (idx >= h.impl->sorted_shape_tags.count) return GdsTag{0, 0};
    gdstk::Tag t = h.impl->sorted_shape_tags[idx];
    return GdsTag{gdstk::get_layer(t), gdstk::get_type(t)};
}

uint64_t gds_info_label_tag_count(const GdsInfoHandle& h) {
    return h.impl->sorted_label_tags.count;
}

GdsTag gds_info_label_tag(const GdsInfoHandle& h, uint64_t idx) {
    if (idx >= h.impl->sorted_label_tags.count) return GdsTag{0, 0};
    gdstk::Tag t = h.impl->sorted_label_tags[idx];
    return GdsTag{gdstk::get_layer(t), gdstk::get_type(t)};
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

uint64_t cell_flexpath_count(const CellHandle& cell) {
    return as_cell(cell)->flexpath_array.count;
}

const FlexPathHandle& cell_flexpath_at(const CellHandle& cell, uint64_t idx) {
    const gdstk::FlexPath* fp = as_cell(cell)->flexpath_array[idx];
    return from_flexpath(fp);
}

uint64_t cell_robustpath_count(const CellHandle& cell) {
    return as_cell(cell)->robustpath_array.count;
}

const RobustPathHandle& cell_robustpath_at(const CellHandle& cell, uint64_t idx) {
    const gdstk::RobustPath* rp = as_cell(cell)->robustpath_array[idx];
    return from_robustpath(rp);
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

Point2D polygon_point_at(const PolygonHandle& poly, uint64_t idx) {
    const gdstk::Polygon* p = as_polygon(poly);
    if (idx >= p->point_array.count) {
        return Point2D{0.0, 0.0};
    }
    const gdstk::Vec2& v = p->point_array[idx];
    return Point2D{v.x, v.y};
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

// ---- FlexPath ----

uint64_t flexpath_num_elements(const FlexPathHandle& path) {
    return as_flexpath(path)->num_elements;
}

uint32_t flexpath_element_layer(const FlexPathHandle& path, uint64_t element_idx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (element_idx >= fp->num_elements) return 0;
    return gdstk::get_layer(fp->elements[element_idx].tag);
}

uint32_t flexpath_element_datatype(const FlexPathHandle& path, uint64_t element_idx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (element_idx >= fp->num_elements) return 0;
    return gdstk::get_type(fp->elements[element_idx].tag);
}

uint64_t flexpath_spine_point_count(const FlexPathHandle& path) {
    return as_flexpath(path)->spine.point_array.count;
}

Point2D flexpath_spine_point(const FlexPathHandle& path, uint64_t idx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (idx >= fp->spine.point_array.count) return Point2D{0.0, 0.0};
    const gdstk::Vec2& p = fp->spine.point_array[idx];
    return Point2D{p.x, p.y};
}

double flexpath_element_half_width(const FlexPathHandle& path, uint64_t eidx,
                                   uint64_t sidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0.0;
    const gdstk::FlexPathElement& el = fp->elements[eidx];
    if (sidx >= el.half_width_and_offset.count) return 0.0;
    return el.half_width_and_offset[sidx].x;  // .x = half_width
}

double flexpath_element_offset(const FlexPathHandle& path, uint64_t eidx,
                               uint64_t sidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0.0;
    const gdstk::FlexPathElement& el = fp->elements[eidx];
    if (sidx >= el.half_width_and_offset.count) return 0.0;
    return el.half_width_and_offset[sidx].y;  // .y = offset
}

uint8_t flexpath_element_end_type(const FlexPathHandle& path, uint64_t eidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0;
    return static_cast<uint8_t>(fp->elements[eidx].end_type);
}

uint8_t flexpath_element_join_type(const FlexPathHandle& path, uint64_t eidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0;
    return static_cast<uint8_t>(fp->elements[eidx].join_type);
}

uint8_t flexpath_element_bend_type(const FlexPathHandle& path, uint64_t eidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0;
    return static_cast<uint8_t>(fp->elements[eidx].bend_type);
}

double flexpath_element_bend_radius(const FlexPathHandle& path, uint64_t eidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return 0.0;
    return fp->elements[eidx].bend_radius;
}

Point2D flexpath_element_end_extensions(const FlexPathHandle& path, uint64_t eidx) {
    const gdstk::FlexPath* fp = as_flexpath(path);
    if (eidx >= fp->num_elements) return Point2D{0.0, 0.0};
    const gdstk::Vec2& ex = fp->elements[eidx].end_extensions;
    return Point2D{ex.x, ex.y};
}

bool flexpath_simple_path(const FlexPathHandle& path) {
    return as_flexpath(path)->simple_path;
}

bool flexpath_scale_width(const FlexPathHandle& path) {
    return as_flexpath(path)->scale_width;
}

// ---- RobustPath ----

uint64_t robustpath_num_elements(const RobustPathHandle& path) {
    return as_robustpath(path)->num_elements;
}

uint32_t robustpath_element_layer(const RobustPathHandle& path, uint64_t element_idx) {
    const gdstk::RobustPath* rp = as_robustpath(path);
    if (element_idx >= rp->num_elements) return 0;
    return gdstk::get_layer(rp->elements[element_idx].tag);
}

uint32_t robustpath_element_datatype(const RobustPathHandle& path, uint64_t element_idx) {
    const gdstk::RobustPath* rp = as_robustpath(path);
    if (element_idx >= rp->num_elements) return 0;
    return gdstk::get_type(rp->elements[element_idx].tag);
}

uint64_t robustpath_subpath_count(const RobustPathHandle& path) {
    return as_robustpath(path)->subpath_array.count;
}

Point2D robustpath_end_point(const RobustPathHandle& path) {
    const gdstk::Vec2& e = as_robustpath(path)->end_point;
    return Point2D{e.x, e.y};
}

double robustpath_tolerance(const RobustPathHandle& path) {
    return as_robustpath(path)->tolerance;
}

uint64_t robustpath_max_evals(const RobustPathHandle& path) {
    return as_robustpath(path)->max_evals;
}

double robustpath_element_end_width(const RobustPathHandle& path, uint64_t eidx) {
    const gdstk::RobustPath* rp = as_robustpath(path);
    if (eidx >= rp->num_elements) return 0.0;
    return rp->elements[eidx].end_width;
}

double robustpath_element_end_offset(const RobustPathHandle& path, uint64_t eidx) {
    const gdstk::RobustPath* rp = as_robustpath(path);
    if (eidx >= rp->num_elements) return 0.0;
    return rp->elements[eidx].end_offset;
}

uint8_t robustpath_element_end_type(const RobustPathHandle& path, uint64_t eidx) {
    const gdstk::RobustPath* rp = as_robustpath(path);
    if (eidx >= rp->num_elements) return 0;
    return static_cast<uint8_t>(rp->elements[eidx].end_type);
}

bool robustpath_simple_path(const RobustPathHandle& path) {
    return as_robustpath(path)->simple_path;
}

bool robustpath_scale_width(const RobustPathHandle& path) {
    return as_robustpath(path)->scale_width;
}

// ---- Boolean XOR helpers ----

// Collects into `filtered` all polygons (both direct and path-derived) from
// `cell` that are on the given layer. Polygons generated from paths are
// owned (heap-allocated) and are appended to `owned_temp` for later cleanup.
// Existing polygons from polygon_array are views (not owned).
static void collect_polygons_for_layer(
    const gdstk::Cell* cell,
    uint32_t layer,
    gdstk::Array<gdstk::Polygon*>& filtered,
    gdstk::Array<gdstk::Polygon*>& owned_temp) {
    // 1. Direct polygons from polygon_array (view, not owned).
    for (uint64_t i = 0; i < cell->polygon_array.count; i++) {
        gdstk::Polygon* p = cell->polygon_array[i];
        if (gdstk::get_layer(p->tag) == layer) {
            filtered.append(p);
        }
    }

    // 2. FlexPath → polygons. Only convert if any element matches the layer
    //    (performance: avoid polygonizing paths that don't contribute).
    for (uint64_t i = 0; i < cell->flexpath_array.count; i++) {
        gdstk::FlexPath* fp = cell->flexpath_array[i];
        bool any_match = false;
        for (uint64_t e = 0; e < fp->num_elements; e++) {
            if (gdstk::get_layer(fp->elements[e].tag) == layer) {
                any_match = true;
                break;
            }
        }
        if (!any_match) continue;

        gdstk::Array<gdstk::Polygon*> path_polys = {};
        fp->to_polygons(/*filter=*/false, /*tag=*/0, path_polys);
        for (uint64_t j = 0; j < path_polys.count; j++) {
            gdstk::Polygon* p = path_polys[j];
            if (gdstk::get_layer(p->tag) == layer) {
                filtered.append(p);
                owned_temp.append(p);
            } else {
                // Different layer → free immediately, not needed.
                p->clear();
                gdstk::free_allocation(p);
            }
        }
        path_polys.clear();
    }

    // 3. RobustPath → polygons (same pattern).
    for (uint64_t i = 0; i < cell->robustpath_array.count; i++) {
        gdstk::RobustPath* rp = cell->robustpath_array[i];
        bool any_match = false;
        for (uint64_t e = 0; e < rp->num_elements; e++) {
            if (gdstk::get_layer(rp->elements[e].tag) == layer) {
                any_match = true;
                break;
            }
        }
        if (!any_match) continue;

        gdstk::Array<gdstk::Polygon*> path_polys = {};
        rp->to_polygons(/*filter=*/false, /*tag=*/0, path_polys);
        for (uint64_t j = 0; j < path_polys.count; j++) {
            gdstk::Polygon* p = path_polys[j];
            if (gdstk::get_layer(p->tag) == layer) {
                filtered.append(p);
                owned_temp.append(p);
            } else {
                p->clear();
                gdstk::free_allocation(p);
            }
        }
        path_polys.clear();
    }
}

// Core XOR driver. `include_paths` controls whether FlexPath/RobustPath
// geometry is included (polygonized on the fly). The public wrappers
// `cell_xor_with` (include_paths=true) and `cell_xor_with_polygons_only`
// (false) just call into this.
static XorMetrics cell_xor_impl(const CellHandle& a, const CellHandle& b,
                                uint32_t layer, bool include_paths) {
    const gdstk::Cell* cell_a = as_cell(a);
    const gdstk::Cell* cell_b = as_cell(b);

    gdstk::Array<gdstk::Polygon*> filtered_a = {};
    gdstk::Array<gdstk::Polygon*> filtered_b = {};
    gdstk::Array<gdstk::Polygon*> owned_temp = {};

    if (include_paths) {
        collect_polygons_for_layer(cell_a, layer, filtered_a, owned_temp);
        collect_polygons_for_layer(cell_b, layer, filtered_b, owned_temp);
    } else {
        for (uint64_t i = 0; i < cell_a->polygon_array.count; i++) {
            gdstk::Polygon* p = cell_a->polygon_array[i];
            if (gdstk::get_layer(p->tag) == layer) filtered_a.append(p);
        }
        for (uint64_t i = 0; i < cell_b->polygon_array.count; i++) {
            gdstk::Polygon* p = cell_b->polygon_array[i];
            if (gdstk::get_layer(p->tag) == layer) filtered_b.append(p);
        }
    }

    XorMetrics metrics{0.0, 0, BoundingBox{0.0, 0.0, 0.0, 0.0}};

    if (filtered_a.count == 0 && filtered_b.count == 0) {
        filtered_a.clear();
        filtered_b.clear();
        // owned_temp is empty in this branch, but clear defensively.
        for (uint64_t i = 0; i < owned_temp.count; i++) {
            owned_temp[i]->clear();
            gdstk::free_allocation(owned_temp[i]);
        }
        owned_temp.clear();
        return metrics;
    }

    gdstk::Array<gdstk::Polygon*> result = {};
    gdstk::ErrorCode err = gdstk::boolean(filtered_a, filtered_b,
                                          gdstk::Operation::Xor,
                                          /*scaling=*/1000.0, result);
    (void)err;

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

    // Free result polygons (allocated by boolean()).
    for (uint64_t i = 0; i < result.count; i++) {
        result[i]->clear();
        gdstk::free_allocation(result[i]);
    }
    result.clear();

    // Free path-derived polygons we collected (owned_temp).
    for (uint64_t i = 0; i < owned_temp.count; i++) {
        owned_temp[i]->clear();
        gdstk::free_allocation(owned_temp[i]);
    }
    owned_temp.clear();

    // filtered_a/b held mixed views + owned pointers; we freed the owned ones
    // via owned_temp. Just release the array capacity.
    filtered_a.clear();
    filtered_b.clear();

    return metrics;
}

// ---- Fase 8: gap-closing read accessors ----

double polygon_perimeter(const PolygonHandle& poly) {
    return as_polygon(poly)->perimeter();
}

double polygon_signed_area(const PolygonHandle& poly) {
    return as_polygon(poly)->signed_area();
}

uint64_t polygon_repetition_count(const PolygonHandle& poly) {
    return repetition_effective_count(as_polygon(poly)->repetition);
}

Point2D polygon_repetition_offset(const PolygonHandle& poly, uint64_t idx) {
    return repetition_offset_at(as_polygon(poly)->repetition, idx);
}

uint64_t label_repetition_count(const LabelHandle& label) {
    return repetition_effective_count(as_label(label)->repetition);
}

Point2D label_repetition_offset(const LabelHandle& label, uint64_t idx) {
    return repetition_offset_at(as_label(label)->repetition, idx);
}

uint64_t reference_repetition_count(const ReferenceHandle& ref) {
    return repetition_effective_count(as_reference(ref)->repetition);
}

Point2D reference_repetition_offset(const ReferenceHandle& ref, uint64_t idx) {
    return repetition_offset_at(as_reference(ref)->repetition, idx);
}

BoundingBox cell_bbox(const CellHandle& cell) {
    gdstk::Vec2 min{}, max{};
    // Cell::bounding_box is non-const in gdstk; cast away const for
    // read-only use on a loaded Library.
    const_cast<gdstk::Cell*>(as_cell(cell))->bounding_box(min, max);
    if (min.x > max.x || min.y > max.y) {
        return BoundingBox{0.0, 0.0, 0.0, 0.0};
    }
    return BoundingBox{min.x, min.y, max.x, max.y};
}

BoundingBox reference_bbox(const ReferenceHandle& ref) {
    gdstk::Vec2 min{}, max{};
    const_cast<gdstk::Reference*>(as_reference(ref))->bounding_box(min, max);
    if (min.x > max.x || min.y > max.y) {
        return BoundingBox{0.0, 0.0, 0.0, 0.0};
    }
    return BoundingBox{min.x, min.y, max.x, max.y};
}

// RawCell

uint64_t library_rawcell_count(const LibraryHandle& handle) {
    return handle.impl->lib.rawcell_array.count;
}

const RawCellHandle& library_rawcell_at(const LibraryHandle& handle, uint64_t idx) {
    const gdstk::RawCell* rc = handle.impl->lib.rawcell_array[idx];
    return from_rawcell(rc);
}

rust::Str rawcell_name(const RawCellHandle& rc) {
    const char* name = as_rawcell(rc)->name;
    if (name == nullptr) return rust::Str("");
    return rust::Str(name, std::strlen(name));
}

uint64_t rawcell_size(const RawCellHandle& rc) {
    return as_rawcell(rc)->size;
}

uint64_t rawcell_dependency_count(const RawCellHandle& rc) {
    return as_rawcell(rc)->dependencies.count;
}

const RawCellHandle& rawcell_dependency_at(const RawCellHandle& rc, uint64_t idx) {
    const gdstk::RawCell* dep = as_rawcell(rc)->dependencies[idx];
    return from_rawcell(dep);
}

// ---- Fase 8.5: Repetition struct detail ----

const RepetitionHandle& polygon_repetition(const PolygonHandle& poly) {
    return from_repetition(&as_polygon(poly)->repetition);
}
const RepetitionHandle& label_repetition(const LabelHandle& label) {
    return from_repetition(&as_label(label)->repetition);
}
const RepetitionHandle& reference_repetition(const ReferenceHandle& ref) {
    return from_repetition(&as_reference(ref)->repetition);
}

uint8_t repetition_type(const RepetitionHandle& rep) {
    return static_cast<uint8_t>(as_repetition(rep)->type);
}

uint64_t repetition_count(const RepetitionHandle& rep) {
    return repetition_effective_count(*as_repetition(rep));
}

uint64_t repetition_columns(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type == gdstk::RepetitionType::Rectangular ||
        r->type == gdstk::RepetitionType::Regular) {
        return r->columns;
    }
    return 0;
}

uint64_t repetition_rows(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type == gdstk::RepetitionType::Rectangular ||
        r->type == gdstk::RepetitionType::Regular) {
        return r->rows;
    }
    return 0;
}

Point2D repetition_spacing(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type != gdstk::RepetitionType::Rectangular) {
        return Point2D{0.0, 0.0};
    }
    return Point2D{r->spacing.x, r->spacing.y};
}

Point2D repetition_v1(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type != gdstk::RepetitionType::Regular) {
        return Point2D{0.0, 0.0};
    }
    return Point2D{r->v1.x, r->v1.y};
}

Point2D repetition_v2(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type != gdstk::RepetitionType::Regular) {
        return Point2D{0.0, 0.0};
    }
    return Point2D{r->v2.x, r->v2.y};
}

uint64_t repetition_coord_count(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type != gdstk::RepetitionType::ExplicitX &&
        r->type != gdstk::RepetitionType::ExplicitY) {
        return 0;
    }
    return r->coords.count;
}

double repetition_coord(const RepetitionHandle& rep, uint64_t idx) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type != gdstk::RepetitionType::ExplicitX &&
        r->type != gdstk::RepetitionType::ExplicitY) {
        return 0.0;
    }
    if (idx >= r->coords.count) {
        return 0.0;
    }
    return r->coords[idx];
}

Point2D repetition_generated_offset(const RepetitionHandle& rep, uint64_t idx) {
    return repetition_offset_at(*as_repetition(rep), idx);
}

uint64_t repetition_extrema_count(const RepetitionHandle& rep) {
    const gdstk::Repetition* r = as_repetition(rep);
    // gdstk's get_extrema switch doesn't handle None; normalize to 1
    // (origin-only) for consistency with our repetition_effective_count.
    if (r->type == gdstk::RepetitionType::None) return 1;
    gdstk::Array<gdstk::Vec2> buf = {};
    r->get_extrema(buf);
    uint64_t n = buf.count;
    buf.clear();
    return n;
}

Point2D repetition_extremum(const RepetitionHandle& rep, uint64_t idx) {
    const gdstk::Repetition* r = as_repetition(rep);
    if (r->type == gdstk::RepetitionType::None) {
        // Single extremum = origin.
        return Point2D{0.0, 0.0};
    }
    gdstk::Array<gdstk::Vec2> buf = {};
    r->get_extrema(buf);
    Point2D out{0.0, 0.0};
    if (idx < buf.count) {
        out.x = buf[idx].x;
        out.y = buf[idx].y;
    }
    buf.clear();
    return out;
}

// ---- Flatten ----

std::unique_ptr<FlattenedPolygonsHandle> cell_get_polygons_flat(
    const CellHandle& cell,
    bool apply_repetitions,
    bool include_paths,
    int64_t depth,
    bool use_filter,
    uint32_t layer,
    uint32_t datatype) {
    auto view = std::make_unique<FlattenedPolygonsHandle>();
    gdstk::Tag tag = gdstk::make_tag(layer, datatype);
    // Cell::get_polygons is not const; const_cast is safe for read-only
    // traversal — same pattern used by cell_bbox.
    const_cast<gdstk::Cell*>(as_cell(cell))->get_polygons(
        apply_repetitions, include_paths, depth, use_filter, tag,
        view->impl->polygons);
    return view;
}

std::unique_ptr<FlattenedPolygonsHandle> reference_get_polygons_flat(
    const ReferenceHandle& ref,
    bool apply_repetitions,
    bool include_paths,
    int64_t depth,
    bool use_filter,
    uint32_t layer,
    uint32_t datatype) {
    auto view = std::make_unique<FlattenedPolygonsHandle>();
    gdstk::Tag tag = gdstk::make_tag(layer, datatype);
    const_cast<gdstk::Reference*>(as_reference(ref))->get_polygons(
        apply_repetitions, include_paths, depth, use_filter, tag,
        view->impl->polygons);
    return view;
}

uint64_t flattened_polygons_count(const FlattenedPolygonsHandle& view) {
    return view.impl->polygons.count;
}

const PolygonHandle& flattened_polygons_at(
    const FlattenedPolygonsHandle& view, uint64_t idx) {
    const gdstk::Polygon* p = view.impl->polygons[idx];
    return from_polygon(p);
}

XorMetrics cell_xor_with(const CellHandle& a, const CellHandle& b, uint32_t layer) {
    return cell_xor_impl(a, b, layer, /*include_paths=*/true);
}

XorMetrics cell_xor_with_polygons_only(const CellHandle& a, const CellHandle& b,
                                       uint32_t layer) {
    return cell_xor_impl(a, b, layer, /*include_paths=*/false);
}

// ---- Directional XOR (added / removed) ----

// Copies geometry from boolean() result polygons into `dest`, leaving
// the original allocations to be freed by the caller.
static void copy_into_owned(const gdstk::Array<gdstk::Polygon*>& result,
                            std::vector<XorSplitHandle::Impl::OwnedPoly>& dest) {
    dest.reserve(dest.size() + result.count);
    for (uint64_t i = 0; i < result.count; i++) {
        const gdstk::Polygon* p = result[i];
        XorSplitHandle::Impl::OwnedPoly op;
        op.layer = gdstk::get_layer(p->tag);
        op.datatype = gdstk::get_type(p->tag);
        op.points.reserve(p->point_array.count);
        for (uint64_t j = 0; j < p->point_array.count; j++) {
            op.points.push_back(Point2D{p->point_array[j].x, p->point_array[j].y});
        }
        dest.push_back(std::move(op));
    }
}

static void run_boolean_not(const gdstk::Array<gdstk::Polygon*>& lhs,
                            const gdstk::Array<gdstk::Polygon*>& rhs,
                            std::vector<XorSplitHandle::Impl::OwnedPoly>& dest) {
    if (lhs.count == 0) return;  // empty minus anything is empty
    gdstk::Array<gdstk::Polygon*> result = {};
    gdstk::ErrorCode err = gdstk::boolean(lhs, rhs, gdstk::Operation::Not,
                                          /*scaling=*/1000.0, result);
    (void)err;
    copy_into_owned(result, dest);
    for (uint64_t i = 0; i < result.count; i++) {
        result[i]->clear();
        gdstk::free_allocation(result[i]);
    }
    result.clear();
}

std::unique_ptr<XorSplitHandle> cell_xor_polygons_split(
    const CellHandle& a, const CellHandle& b, uint32_t layer) {
    const gdstk::Cell* cell_a = as_cell(a);
    const gdstk::Cell* cell_b = as_cell(b);

    gdstk::Array<gdstk::Polygon*> filtered_a = {};
    gdstk::Array<gdstk::Polygon*> filtered_b = {};
    gdstk::Array<gdstk::Polygon*> owned_temp = {};

    // Always include path-derived polygons — same correctness rationale
    // as cell_xor_with: GDS files with FlexPath/RobustPath would otherwise
    // miss diff geometry on wires.
    collect_polygons_for_layer(cell_a, layer, filtered_a, owned_temp);
    collect_polygons_for_layer(cell_b, layer, filtered_b, owned_temp);

    auto handle = std::make_unique<XorSplitHandle>();

    // added = polygons in B that are not in A.
    run_boolean_not(filtered_b, filtered_a, handle->impl->added);
    // removed = polygons in A that are not in B.
    run_boolean_not(filtered_a, filtered_b, handle->impl->removed);

    // Free path-derived polygons we collected.
    for (uint64_t i = 0; i < owned_temp.count; i++) {
        owned_temp[i]->clear();
        gdstk::free_allocation(owned_temp[i]);
    }
    owned_temp.clear();
    filtered_a.clear();
    filtered_b.clear();

    return handle;
}

uint64_t xor_split_added_count(const XorSplitHandle& h) {
    return h.impl->added.size();
}
uint64_t xor_split_removed_count(const XorSplitHandle& h) {
    return h.impl->removed.size();
}

uint32_t xor_split_added_layer(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->added.size() ? h.impl->added[poly_idx].layer : 0;
}
uint32_t xor_split_added_datatype(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->added.size() ? h.impl->added[poly_idx].datatype : 0;
}
uint64_t xor_split_added_point_count(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->added.size()
        ? h.impl->added[poly_idx].points.size() : 0;
}
Point2D xor_split_added_point(const XorSplitHandle& h, uint64_t poly_idx,
                              uint64_t point_idx) {
    if (poly_idx >= h.impl->added.size()) return Point2D{0.0, 0.0};
    const auto& pts = h.impl->added[poly_idx].points;
    if (point_idx >= pts.size()) return Point2D{0.0, 0.0};
    return pts[point_idx];
}

uint32_t xor_split_removed_layer(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->removed.size() ? h.impl->removed[poly_idx].layer : 0;
}
uint32_t xor_split_removed_datatype(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->removed.size() ? h.impl->removed[poly_idx].datatype : 0;
}
uint64_t xor_split_removed_point_count(const XorSplitHandle& h, uint64_t poly_idx) {
    return poly_idx < h.impl->removed.size()
        ? h.impl->removed[poly_idx].points.size() : 0;
}
Point2D xor_split_removed_point(const XorSplitHandle& h, uint64_t poly_idx,
                                uint64_t point_idx) {
    if (poly_idx >= h.impl->removed.size()) return Point2D{0.0, 0.0};
    const auto& pts = h.impl->removed[poly_idx].points;
    if (point_idx >= pts.size()) return Point2D{0.0, 0.0};
    return pts[point_idx];
}

// ---- Library tag discovery ----

static void ensure_tags_cached(const LibraryHandle& handle) {
    if (handle.impl->tags_cached) return;
    std::set<gdstk::Tag> seen;
    for (uint64_t i = 0; i < handle.impl->lib.cell_array.count; i++) {
        const gdstk::Cell* cell = handle.impl->lib.cell_array[i];
        for (uint64_t j = 0; j < cell->polygon_array.count; j++) {
            seen.insert(cell->polygon_array[j]->tag);
        }
    }
    handle.impl->cached_tags.assign(seen.begin(), seen.end());
    handle.impl->tags_cached = true;
}

uint64_t library_tag_count(const LibraryHandle& handle) {
    ensure_tags_cached(handle);
    return handle.impl->cached_tags.size();
}

GdsTag library_tag_at(const LibraryHandle& handle, uint64_t idx) {
    ensure_tags_cached(handle);
    if (idx >= handle.impl->cached_tags.size()) return GdsTag{0, 0};
    gdstk::Tag t = handle.impl->cached_tags[idx];
    return GdsTag{gdstk::get_layer(t), gdstk::get_type(t)};
}

}  // namespace gdstk_shim
