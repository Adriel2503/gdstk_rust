//! Build script for gdstk-rs.
//!
//! Responsibilities:
//! 1. Compile gdstk's C++ core (../src/*.cpp) into a static library.
//! 2. Compile the vendored Clipper (../external/clipper/clipper.cpp).
//! 3. Compile the cxx bridge (src/lib.rs) and C++ shims (src/shims.cpp).
//! 4. Resolve and link system dependencies: zlib and qhull.
//!
//! Windows uses vcpkg. Unix uses pkg-config. Users may override either
//! dependency with ZLIB_DIR / QHULL_DIR pointing at an installation prefix.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Default)]
struct NativeDeps {
    include_paths: Vec<PathBuf>,
    link_search_paths: Vec<PathBuf>,
    link_libs: Vec<String>,
}

impl NativeDeps {
    fn push_include(&mut self, path: impl Into<PathBuf>) {
        self.include_paths.push(path.into());
    }

    fn push_link_search(&mut self, path: impl Into<PathBuf>) {
        self.link_search_paths.push(path.into());
    }

    fn push_link_lib(&mut self, lib: impl Into<String>) {
        self.link_libs.push(lib.into());
    }

    fn extend(&mut self, mut other: NativeDeps) {
        self.include_paths.append(&mut other.include_paths);
        self.link_search_paths.append(&mut other.link_search_paths);
        self.link_libs.append(&mut other.link_libs);
    }

    fn extend_pkg_config(&mut self, lib: pkg_config::Library) {
        self.include_paths.extend(lib.include_paths);
        self.link_search_paths.extend(lib.link_paths);
        self.link_libs.extend(lib.libs);
    }

    fn emit_link_directives(&self) {
        for path in &self.link_search_paths {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        for lib in &self.link_libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}

fn main() {
    rerun_if_env_changed();

    let gdstk_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust/ must be inside gdstk/")
        .to_path_buf();

    let src_dir = gdstk_root.join("src");
    let include_dir = gdstk_root.join("include");
    let external_dir = gdstk_root.join("external");
    let clipper_dir = external_dir.join("clipper");

    let deps = resolve_native_deps();

    let mut build = cxx_build::bridge("src/lib.rs");

    // gdstk core sources.
    for src in GDSTK_SOURCES {
        build.file(src_dir.join(src));
    }

    // Vendored Clipper.
    build.file(clipper_dir.join("clipper.cpp"));

    // Our C++ shim.
    build.file("src/shims.cpp");

    // Include paths from system deps first, then gdstk/public headers.
    for include_path in &deps.include_paths {
        build.include(include_path);
    }
    build.include(&include_dir);
    build.include(&external_dir);

    // cxx requires C++17; gdstk itself is C++11 but forward-compatible.
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
    deps.emit_link_directives();

    // Keep the Unix link to libm explicit; qhull often needs it and the extra
    // directive is harmless on platforms where it is unnecessary.
    if !cfg!(target_env = "msvc") {
        println!("cargo:rustc-link-lib=m");
    }
}

fn rerun_if_env_changed() {
    for var in [
        "VCPKG_ROOT",
        "VCPKG_DEFAULT_TRIPLET",
        "ZLIB_DIR",
        "QHULL_DIR",
        "PKG_CONFIG_PATH",
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_SYSROOT_DIR",
    ] {
        println!("cargo:rerun-if-env-changed={}", var);
    }
}

fn resolve_native_deps() -> NativeDeps {
    if !cfg!(windows) {
        ensure_pkg_config_available();
    }
    let mut deps = NativeDeps::default();
    deps.extend(resolve_zlib());
    deps.extend(resolve_qhull());
    deps
}

fn resolve_zlib() -> NativeDeps {
    if let Ok(prefix) = env::var("ZLIB_DIR") {
        return NativeDeps::from_prefix(
            prefix,
            "zlib",
            zlib_link_name(),
            "a zlib installation prefix with `include/` and `lib/`",
            if cfg!(windows) {
                Some("zlib.lib")
            } else {
                None
            },
        );
    }

    if cfg!(windows) {
        return NativeDeps::from_vcpkg(
            "x64-windows",
            "zlib",
            "zlib",
            &["zlib", "zlibstatic"],
            "install `zlib` in vcpkg (for example: `vcpkg install zlib --triplet x64-windows`)",
        );
    }

    probe_pkg_config(
        &["zlib", "z"],
        "zlib",
        "ZLIB_DIR",
        "a zlib development package (for example: `zlib1g-dev`, `zlib-devel`, or Homebrew `zlib`)",
    )
}

fn resolve_qhull() -> NativeDeps {
    if let Ok(prefix) = env::var("QHULL_DIR") {
        return NativeDeps::from_prefix(
            prefix,
            "qhull",
            "qhull_r",
            "a qhull installation prefix with `include/` and `lib/`",
            if cfg!(windows) {
                Some("qhull_r.lib")
            } else {
                None
            },
        );
    }

    if cfg!(windows) {
        return NativeDeps::from_vcpkg(
            "x64-windows",
            "qhull_r",
            "qhull",
            &["qhull_r", "qhullstatic_r"],
            "install `qhull` in vcpkg (for example: `vcpkg install qhull --triplet x64-windows`)",
        );
    }

    probe_pkg_config(
        &["qhull_r", "qhull", "libqhull_r", "libqhull"],
        "qhull",
        "QHULL_DIR",
        "a qhull development package (for example: `libqhull-dev`, `qhull-devel`, `qhull`, or Homebrew `qhull`)",
    )
}

fn ensure_pkg_config_available() {
    match Command::new("pkg-config").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            panic!(
                "Could not run `pkg-config --version`. pkg-config is installed but returned a failure status. stdout: `{}` stderr: `{}`",
                stdout.trim(),
                stderr.trim()
            );
        }
        Err(err) => {
            panic!(
                "Could not run `pkg-config --version`. Install `pkg-config` first so Rust can resolve zlib and qhull ({err})."
            );
        }
    }
}

fn zlib_link_name() -> &'static str {
    if cfg!(target_env = "msvc") {
        "zlib"
    } else {
        "z"
    }
}

fn probe_pkg_config(
    candidates: &[&str],
    dep_name: &str,
    env_hint: &str,
    install_hint: &str,
) -> NativeDeps {
    let mut last_error = None;
    for candidate in candidates {
        let mut config = pkg_config::Config::new();
        config.cargo_metadata(false);
        match config.probe(candidate) {
            Ok(lib) => {
                let mut deps = NativeDeps::default();
                deps.extend_pkg_config(lib);
                return deps;
            }
            Err(err) => {
                last_error = Some((candidate.to_string(), err.to_string()));
            }
        }
    }

    let candidate_list = candidates
        .iter()
        .map(|candidate| format!("`{candidate}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let detail = last_error
        .map(|(candidate, err)| {
            format!(" The last attempted package `{candidate}` failed with: {err}")
        })
        .unwrap_or_default();
    panic!(
        "Could not find {dep_name} via pkg-config. Tried {candidate_list}. Install {install_hint}, or set {env_hint} to an installation prefix and expose its .pc files via PKG_CONFIG_PATH or PKG_CONFIG_LIBDIR.{detail}"
    );
}

impl NativeDeps {
    fn from_prefix(
        prefix: impl AsRef<Path>,
        dep_name: &str,
        link_lib: &str,
        install_hint: &str,
        expected_lib: Option<&str>,
    ) -> Self {
        let prefix = prefix.as_ref();
        let include_dir = prefix.join("include");
        let lib_dir = prefix.join("lib");

        if !include_dir.exists() {
            panic!(
                "Missing include directory `{}` for {dep_name}. Expected {install_hint}.",
                include_dir.display(),
            );
        }
        if !lib_dir.exists() {
            panic!(
                "Missing lib directory `{}` for {dep_name}. Expected {install_hint}.",
                lib_dir.display(),
            );
        }
        if let Some(expected_lib) = expected_lib {
            let expected_lib_path = lib_dir.join(expected_lib);
            if !expected_lib_path.exists() {
                panic!(
                    "Missing `{}` in `{}` for {dep_name}. Expected {install_hint}.",
                    expected_lib,
                    lib_dir.display(),
                );
            }
        }

        let mut deps = NativeDeps::default();
        deps.push_include(include_dir);
        deps.push_link_search(lib_dir);
        deps.push_link_lib(link_lib);
        deps
    }

    fn from_vcpkg(
        triplet: &str,
        package: &str,
        dep_name: &str,
        expected_lib_candidates: &[&str],
        install_hint: &str,
    ) -> Self {
        let vcpkg_root = env::var("VCPKG_ROOT").unwrap_or_else(|_| {
            panic!(
                "Set VCPKG_ROOT to a vcpkg checkout and {install_hint}, or use {package} via ZLIB_DIR/QHULL_DIR"
            )
        });
        let triplet = env::var("VCPKG_DEFAULT_TRIPLET").unwrap_or_else(|_| triplet.into());
        let prefix = PathBuf::from(vcpkg_root).join("installed").join(&triplet);
        if !prefix.exists() {
            panic!(
                "Could not find vcpkg triplet `{}` at `{}` for {dep_name}. Install {install_hint} and ensure `VCPKG_DEFAULT_TRIPLET` points to an installed triplet.",
                triplet,
                prefix.display(),
            );
        }
        let lib_dir = prefix.join("lib");
        for expected_lib in expected_lib_candidates {
            let expected_lib_name = if expected_lib.ends_with(".lib") {
                expected_lib.to_string()
            } else {
                format!("{expected_lib}.lib")
            };
            if lib_dir.join(&expected_lib_name).exists() {
                let link_lib = expected_lib_name.trim_end_matches(".lib");
                return Self::from_prefix(
                    prefix,
                    dep_name,
                    link_lib,
                    install_hint,
                    Some(&expected_lib_name),
                );
            }
        }

        let expected_list = expected_lib_candidates
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        panic!(
            "Could not find any of {expected_list} in `{}` for {dep_name}. Install {install_hint}.",
            lib_dir.display(),
        );
    }
}
