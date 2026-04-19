# gdstk-rs

Rust bindings for [gdstk](https://github.com/heitzmann/gdstk) via the `cxx` crate.

**Estado:** Fase 8.5 — **completada 2026-04-19**. Bindings Rust con `Repetition` completo (kind + columns/rows/spacing/v1/v2/coords).

Expone:
- `Library::open(path)` / `Library::find_cell(name)` — parseo + lookup
- `Library::name()` / `unit()` / `precision()` — metadata GDSII
- `Library::write_gds(path) -> Result<(), Error>` — escribir GDS
- `Library::top_level() -> TopLevel<'_>` — cells top-level
- `gds_info(path) -> Result<GdsInfo, Error>` — peek rápido de metadata sin
  full parse (~1.5-2× más rápido que `Library::open`)
- `GdsInfo::{unit, precision, num_polygons, num_paths, num_references,
  num_labels, cell_names, shape_tags, label_tags}` — counts y tags sin parsear
- `Error` / `ErrorCode` — error handling idiomático Rust
- `Library::cells()` / `Library::cell(idx)` / `Library::cell_count()`
- `Cell::name()` / `Cell::polygons()` / `Cell::labels()` / `Cell::references()` /
  `Cell::flexpaths()` / `Cell::robustpaths()` / counts
- `Cell::xor_with(other, layer) -> XorMetrics` — **diff geométrico completo**
  (incluye polygons + paths convertidos a polygons; correcto para GDS reales)
- `Cell::xor_with_polygons_only(other, layer)` — XOR legacy solo de polygons
- `Polygon::area()` / `layer()` / `datatype()` / `bbox()` / `point_count()` /
  `perimeter()` / `signed_area()` / `repetition_count()` / `repetition_offsets()`
- `Cell::bbox()` — bounding box de toda la celda
- `Reference::bbox()` — bbox con transformaciones aplicadas
- `Library::rawcells()` / `rawcell_count()` — iterar RawCells (librerías externas
  pre-compiladas)
- `RawCell::name()` / `size()` / `dependencies()`
- `Label::text()` (Cow<str> UTF-8 lossy) / `layer()` / `texttype()` / `origin()`
  / `anchor()` / `rotation()` / `magnification()` / `x_reflection()`
- `Reference::cell_name()` / `origin()` / `rotation()` / `magnification()` /
  `x_reflection()`
- `FlexPath::num_elements()` / `element_layer(i)` / `element_datatype(i)` /
  `spine_point_count()` / `spine_point(i)` / `element_half_width(e, s)` /
  `element_offset(e, s)` / `element_end_type(e)` / `element_join_type(e)` /
  `element_bend_type(e)` / `element_bend_radius(e)` / `element_end_extensions(e)` /
  `simple_path()` / `scale_width()`
- `RobustPath::num_elements()` / `element_layer(i)` / `element_datatype(i)` /
  `subpath_count()` / `end_point()` / `tolerance()` / `max_evals()` /
  `element_end_width(e)` / `element_end_offset(e)` / `element_end_type(e)` /
  `simple_path()` / `scale_width()`
- Enums `EndType`, `JoinType`, `BendType` (valores contiguos matching gdstk)
- Shared POD structs: `BoundingBox`, `Point2D`, `XorMetrics`
- Enum `Anchor` (NW/N/NE/W/O/E/SW/S/SE con valores sparse de gdstk)

Paridad verificada contra Python gdstk:
- `list_polygons`, `list_labels`, `list_references`: diff byte-a-byte (post-CRLF)
- `diff_gds`: XOR numérico idéntico en pruebas sintéticas
  (rectángulo 5×3 agregado → detectado como 15.00 µm² en 1 región)
- Invariante: `diff_gds a.gds a.gds` reporta 0 cambios

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

## Testing

```powershell
$env:VCPKG_ROOT = "C:\vcpkg"

# Tests de integración (16 tests, ~0.5 s)
cargo test --release

# Benchmarks estadísticos con criterion
cargo bench

# Flujo completo (build + test + snapshots)
./run_tests.sh

# Flujo completo + benchmarks
RUN_BENCH=1 ./run_tests.sh

# Regenerar snapshots cuando el output cambia intencionalmente
REGENERATE_SNAPSHOTS=1 cargo test --release
```

Los tests usan `proof_lib.gds` (incluido en gdstk/tests/). Algunos tests
opcionales requieren `tinytapeout.gds` del repo `tinytapeout_gds_viewer/`
— se saltan si no está presente.

**Benchmarks de referencia** (proof_lib.gds, i7-8th gen):
- `read_gds`: ~125 µs
- `gds_info`: ~85 µs (1.5× más rápido que read_gds)
- `cell_xor_with`: ~73 µs
- `iterate_polygons`: ~10 µs

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
