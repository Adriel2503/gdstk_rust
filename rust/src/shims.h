#pragma once

#include <cstdint>
#include <memory>

#include "rust/cxx.h"

namespace gdstk_shim {

// Opaque handle wrapping gdstk::Library. Uses PIMPL so the full gdstk
// headers (with templated Array<T>, Set<T>, etc.) don't leak into the
// cxx bridge compilation unit. cxx requires the full type definition to
// be visible (for std::unique_ptr destructor generation), but we hide the
// actual gdstk::Library inside Impl.
struct LibraryHandle {
    struct Impl;
    std::unique_ptr<Impl> impl;

    LibraryHandle();
    ~LibraryHandle();

    LibraryHandle(const LibraryHandle&) = delete;
    LibraryHandle& operator=(const LibraryHandle&) = delete;
};

// Reads a GDSII file and returns a heap-allocated LibraryHandle.
// On failure (IO error, corrupt file), returns a handle with zero cells
// and the error is currently swallowed (Fase 1 — error propagation is future work).
std::unique_ptr<LibraryHandle> read_gds_shim(rust::Str filename);

// Returns the number of cells in the loaded library.
uint64_t library_cell_count(const LibraryHandle& handle);

}  // namespace gdstk_shim
