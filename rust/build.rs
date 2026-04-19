//! Build script for gdstk-rs.
//!
//! Responsibilities:
//! 1. Compile gdstk's C++ core (../src/*.cpp) into a static library.
//! 2. Compile the vendored Clipper (../external/clipper/clipper.cpp).
//! 3. Compile the cxx bridge (src/lib.rs) and C++ shims (src/shims.cpp).
//! 4. Link system dependencies: zlib (required) and qhull (required).
//!
//! System deps are resolved via vcpkg on Windows and pkg-config on Unix.
//! Override with VCPKG_ROOT or ZLIB_DIR / QHULL_DIR environment variables.

use std::path::PathBuf;

const GDSTK_SOURCES: &[&str] = &[
    "cell.cpp",
    "clipper_tools.cpp",
    "curve.cpp",
    "flexpath.cpp",
    "gdsii.cpp",
    "label.cpp",
    "layername.cpp",
    "library.cpp",
    "oasis.cpp",
    "polygon.cpp",
    "property.cpp",
    "raithdata.cpp",
    "rawcell.cpp",
    "reference.cpp",
    "repetition.cpp",
    "robustpath.cpp",
    "style.cpp",
    "utils.cpp",
];

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/shims.h");
    println!("cargo:rerun-if-changed=src/shims.cpp");

    let gdstk_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust/ must be inside gdstk/")
        .to_path_buf();

    let src_dir = gdstk_root.join("src");
    let include_dir = gdstk_root.join("include");
    let external_dir = gdstk_root.join("external");
    let clipper_dir = external_dir.join("clipper");

    // Resolve vcpkg include directory (zlib.h, libqhull_r/*.h) at compile time.
    let vcpkg_include = vcpkg_include_dir();

    let mut build = cxx_build::bridge("src/lib.rs");

    // gdstk core sources
    for src in GDSTK_SOURCES {
        build.file(src_dir.join(src));
    }

    // Clipper (vendored)
    build.file(clipper_dir.join("clipper.cpp"));

    // Our C++ shim
    build.file("src/shims.cpp");

    // Includes: gdstk public headers + external/ (clipper_tools.cpp uses
    // `#include "clipper/clipper.hpp"` so the parent directory is needed)
    // + vcpkg includes (zlib.h, libqhull_r/*.h used by gdstk's src/utils.cpp).
    build.include(&include_dir);
    build.include(&external_dir);
    if let Some(vi) = &vcpkg_include {
        build.include(vi);
    }

    // C++17 — cxx requires it; gdstk itself is C++11 but forward-compatible.
    build.std("c++17");

    // Platform-specific flags mirroring gdstk's src/CMakeLists.txt.
    if cfg!(target_env = "msvc") {
        build.flag("/EHsc").flag("/wd4996").define("NOMINMAX", None);
    } else {
        build
            .flag_if_supported("-Wno-missing-field-initializers")
            .flag_if_supported("-Wno-missing-braces")
            .flag_if_supported("-Wno-cast-function-type")
            .flag_if_supported("-Wno-unused-parameter");
    }

    build.compile("gdstk_rs");

    // System library linking.
    link_system_deps();
}

fn vcpkg_include_dir() -> Option<PathBuf> {
    let vcpkg_root = std::env::var("VCPKG_ROOT").ok()?;
    let triplet = std::env::var("VCPKG_DEFAULT_TRIPLET").unwrap_or_else(|_| "x64-windows".into());
    Some(PathBuf::from(format!(
        "{}/installed/{}/include",
        vcpkg_root, triplet
    )))
}

fn link_system_deps() {
    // zlib / qhull via optional explicit dirs.
    if let Ok(zlib_dir) = std::env::var("ZLIB_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", zlib_dir);
    }
    if let Ok(qhull_dir) = std::env::var("QHULL_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", qhull_dir);
    }
    // vcpkg (Windows default)
    if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        let triplet =
            std::env::var("VCPKG_DEFAULT_TRIPLET").unwrap_or_else(|_| "x64-windows".into());
        let lib_dir = format!("{}/installed/{}/lib", vcpkg_root, triplet);
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    // Link names — vary by platform / vcpkg triplet.
    if cfg!(target_env = "msvc") {
        // vcpkg default names on Windows: zlib.lib, qhull_r.lib
        println!("cargo:rustc-link-lib=zlib");
        println!("cargo:rustc-link-lib=qhull_r");
    } else {
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=qhull_r");
        println!("cargo:rustc-link-lib=m");
    }
}
