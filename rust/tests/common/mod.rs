//! Helpers compartidos por todos los tests de integración.

use std::path::Path;

/// Path al fixture principal — proof_lib.gds vive en gdstk/tests/.
pub fn proof_lib_path() -> String {
    format!("{}/../tests/proof_lib.gds", env!("CARGO_MANIFEST_DIR"))
}

/// Path a tinytapeout.gds si está disponible (repo externo).
#[allow(dead_code)]
pub fn tinytapeout_path() -> Option<String> {
    let p = format!(
        "{}/../../tinytapeout_gds_viewer/public/tinytapeout.gds",
        env!("CARGO_MANIFEST_DIR")
    );
    if Path::new(&p).exists() {
        Some(p)
    } else {
        None
    }
}

/// Ejecuta un example compilado y captura stdout. Requiere
/// `cargo build --release --examples` previo. Con DLLs copiadas a
/// `target/release/examples/` en Windows.
#[allow(dead_code)]
pub fn run_example(name: &str, arg: &str) -> String {
    let exe = format!(
        "{}/target/release/examples/{}{}",
        env!("CARGO_MANIFEST_DIR"),
        name,
        std::env::consts::EXE_SUFFIX
    );
    let output = std::process::Command::new(&exe)
        .arg(arg)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {exe}: {e}"));
    assert!(
        output.status.success(),
        "{exe} exited non-zero: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("example stdout not UTF-8")
}

/// Normaliza CRLF a LF (Python en Windows escribe CRLF al redirect).
#[allow(dead_code)]
pub fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}
