#include "shims.h"

#include <string>

#include "gdstk/gdstk.hpp"

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

// Out-of-line destructor — required by PIMPL so Impl is complete at destruction.
LibraryHandle::~LibraryHandle() = default;

std::unique_ptr<LibraryHandle> read_gds_shim(rust::Str filename) {
    auto handle = std::make_unique<LibraryHandle>();

    // rust::Str is not null-terminated; copy into std::string.
    std::string path(filename.data(), filename.size());

    gdstk::ErrorCode error_code = gdstk::ErrorCode::NoError;
    handle->impl->lib = gdstk::read_gds(path.c_str(),
                                        /*unit=*/0.0,
                                        /*tolerance=*/0.0,
                                        /*shape_tags=*/nullptr,
                                        &error_code);

    // Fase 1: errors are silently swallowed. A non-NoError code means the
    // library is empty/invalid; the caller will see cell_count == 0.
    // Future phases will expose ErrorCode to Rust.
    (void)error_code;

    return handle;
}

uint64_t library_cell_count(const LibraryHandle& handle) {
    return handle.impl->lib.cell_array.count;
}

}  // namespace gdstk_shim
