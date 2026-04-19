//! Rust bindings for gdstk — Fase 1 (Proof of Concept).
//!
//! Only exposes `read_gds` and cell count. Full API surface is planned
//! incrementally across Fases 2-7 (see research/arquitectura/gdstk_rust_bindings_migracion.md).

#[cxx::bridge(namespace = "gdstk_shim")]
mod ffi {
    unsafe extern "C++" {
        include!("gdstk-rs/src/shims.h");

        type LibraryHandle;

        fn read_gds_shim(filename: &str) -> UniquePtr<LibraryHandle>;
        fn library_cell_count(handle: &LibraryHandle) -> u64;
    }
}

/// Reads a GDSII file and returns an owned handle to the library.
///
/// In Fase 1 errors are swallowed — a corrupt or missing file returns a
/// handle with zero cells. Future versions will expose gdstk::ErrorCode.
pub fn read_gds(filename: &str) -> cxx::UniquePtr<ffi::LibraryHandle> {
    ffi::read_gds_shim(filename)
}

/// Returns the number of cells in a loaded library.
pub fn cell_count(handle: &ffi::LibraryHandle) -> u64 {
    ffi::library_cell_count(handle)
}

// Re-export the opaque handle type so users can hold references.
pub use ffi::LibraryHandle;
