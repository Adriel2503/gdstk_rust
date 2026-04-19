# gdstk-rs

Rust bindings for [gdstk](https://github.com/heitzmann/gdstk) via the `cxx` crate.

**Estado:** Fase 2 — **completada 2026-04-19**.

Expone:
- `Library::open(path)` — parseo GDSII
- `Library::cells()` / `Library::cell(idx)` / `Library::cell_count()`
- `Cell::name()` / `Cell::polygons()` / `Cell::polygon_count()`
- `Polygon::area()` / `Polygon::layer()` / `Polygon::datatype()` / `Polygon::bbox()` / `Polygon::point_count()`
- `BoundingBox` (shared POD struct)

Paridad verificada con Python gdstk vía `examples/list_polygons.{rs,py}`:
ambos producen exactamente el mismo output (área por capa por celda) para
`proof_lib.gds` y `tinytapeout.gds` (diff vacío al normalizar CRLF→LF).

El plan completo vive en `../research/arquitectura/gdstk_rust_bindings_migracion.md`.

## Prerequisitos (Windows)

1. **vcpkg** con zlib y qhull:
   ```powershell
   git clone https://github.com/microsoft/vcpkg
   cd vcpkg
   .\bootstrap-vcpkg.bat
   .\vcpkg install qhull zlib --triplet x64-windows
   ```

2. Setear variable de entorno `VCPKG_ROOT` apuntando al clone de vcpkg:
   ```powershell
   setx VCPKG_ROOT "C:\path\to\vcpkg"
   ```

3. **Rust toolchain MSVC** (no MinGW):
   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   ```

## Build

```powershell
cd rust
cargo build
```

El `build.rs` compila los 18 archivos de `../src/`, vendored Clipper, los shims C++, y linkea zlib/qhull desde vcpkg.

## Uso

```powershell
$env:VCPKG_ROOT = "C:\vcpkg"
cargo build
# Copiar DLLs junto al exe (una vez):
Copy-Item C:\vcpkg\installed\x64-windows\bin\zlib1.dll target\debug\examples\
Copy-Item C:\vcpkg\installed\x64-windows\bin\qhull_r.dll target\debug\examples\
cargo run --example count_cells -- path/to/file.gds
```

Debe imprimir el mismo número que:
```python
import gdstk
print(len(gdstk.read_gds("file.gds").cells))
```

Nota: en una versión futura, `build.rs` copiará las DLLs automáticamente o
evaluaremos el triplet `x64-windows-static` para eliminar la dependencia runtime.

## Arquitectura

```
rust/
├── Cargo.toml
├── build.rs          # compila C++ + linkea vcpkg
├── src/
│   ├── lib.rs        # bridge cxx
│   ├── shims.h       # API no-templated para cxx
│   └── shims.cpp     # envuelve gdstk::Library
└── examples/
    └── count_cells.rs
```

cxx no puede exponer `Array<T>`, `Set<T>`, `Map<T>` directamente (templates C++). La capa `shims.cpp` los envuelve en APIs concretas que cxx sí entiende.
