//! Una `Library` se lee desde varios hilos a la vez: aplanar, bounding box,
//! capas y XOR dan lo mismo que en un solo hilo. El GDS de prueba tiene
//! jerarquía (SREF y AREF) y paths con puntos repetidos, el único caso en
//! que gdstk escribe al aplanar (ver `finish_load` en shims.cpp).

mod common;

use common::proof_lib_path;
use gdstk_rs::{xor_split_flat, GdsTag, Library, OwnedPolygon};

const THREADS: usize = 12;
const ROUNDS: usize = 40;

fn rec(out: &mut Vec<u8>, kind: u16, data: &[u8]) {
    out.extend_from_slice(&((4 + data.len()) as u16).to_be_bytes());
    out.extend_from_slice(&kind.to_be_bytes());
    out.extend_from_slice(data);
}

fn i2(v: &[i16]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_be_bytes()).collect()
}

fn i4(v: &[i32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_be_bytes()).collect()
}

fn name(s: &str) -> Vec<u8> {
    let mut b = s.as_bytes().to_vec();
    if b.len() % 2 == 1 {
        b.push(0);
    }
    b
}

fn boundary(o: &mut Vec<u8>, layer: i16, pts: &[(i32, i32)]) {
    rec(o, 0x0800, &[]);
    rec(o, 0x0D02, &i2(&[layer]));
    rec(o, 0x0E02, &i2(&[0]));
    let xy: Vec<i32> = pts.iter().chain(pts.first()).flat_map(|&(x, y)| [x, y]).collect();
    rec(o, 0x1003, &i4(&xy));
    rec(o, 0x1100, &[]);
}

/// PATH de ancho 200 con el punto del medio repetido.
fn path(o: &mut Vec<u8>, layer: i16, pts: &[(i32, i32)]) {
    rec(o, 0x0900, &[]);
    rec(o, 0x0D02, &i2(&[layer]));
    rec(o, 0x0E02, &i2(&[0]));
    rec(o, 0x2102, &i2(&[0]));
    rec(o, 0x0F03, &i4(&[200]));
    let xy: Vec<i32> = pts.iter().flat_map(|&(x, y)| [x, y]).collect();
    rec(o, 0x1003, &i4(&xy));
    rec(o, 0x1100, &[]);
}

/// `SUB` (un rectángulo y un path en la capa 3, que solo tiene paths) y `TOP`
/// con una SREF, una AREF de 4×3 y un path propio. `dx` corre el rectángulo
/// de `TOP` para tener dos versiones distintas.
fn gds(dx: i32) -> Vec<u8> {
    let mut o = Vec::new();
    rec(&mut o, 0x0002, &i2(&[600]));
    rec(&mut o, 0x0102, &i2(&[0; 12]));
    rec(&mut o, 0x0206, &name("LIB"));
    let units = [0x3E41_8937_4BC6_A7EFu64, 0x3944_B82F_A09B_5A51u64];
    rec(&mut o, 0x0305, &units.iter().flat_map(|u| u.to_be_bytes()).collect::<Vec<u8>>());

    rec(&mut o, 0x0502, &i2(&[0; 12]));
    rec(&mut o, 0x0606, &name("SUB"));
    boundary(&mut o, 1, &[(0, 0), (1000, 0), (1000, 500), (0, 500)]);
    path(&mut o, 3, &[(0, 1000), (500, 1000), (500, 1000), (500, 1500), (1500, 1500)]);
    rec(&mut o, 0x0700, &[]);

    rec(&mut o, 0x0502, &i2(&[0; 12]));
    rec(&mut o, 0x0606, &name("TOP"));
    boundary(&mut o, 1, &[(dx, -3000), (dx + 2000, -3000), (dx + 2000, -2000), (dx, -2000)]);
    path(&mut o, 2, &[(-1000, 0), (-1000, 4000), (-1000, 4000), (3000, 4000)]);
    rec(&mut o, 0x0A00, &[]);
    rec(&mut o, 0x1206, &name("SUB"));
    rec(&mut o, 0x1003, &i4(&[10_000, 0]));
    rec(&mut o, 0x1100, &[]);
    rec(&mut o, 0x0B00, &[]);
    rec(&mut o, 0x1206, &name("SUB"));
    rec(&mut o, 0x1302, &i2(&[4, 3]));
    rec(&mut o, 0x1003, &i4(&[0, 20_000, 8000, 20_000, 0, 26_000]));
    rec(&mut o, 0x1100, &[]);
    rec(&mut o, 0x0700, &[]);

    rec(&mut o, 0x0400, &[]);
    o
}

type Flat = Vec<(u32, u32, Vec<(u64, u64)>)>;

fn flat(lib: &Library, cell: &str) -> Flat {
    let c = lib.find_cell(cell).expect("cell");
    c.get_polygons()
        .build()
        .polygons()
        .map(|p| (p.layer(), p.datatype(), p.points().map(|q| (q.x.to_bits(), q.y.to_bits())).collect()))
        .collect()
}

fn owned_bits(v: &[OwnedPolygon]) -> Flat {
    v.iter()
        .map(|p| (p.layer, p.datatype, p.points.iter().map(|q| (q.x.to_bits(), q.y.to_bits())).collect()))
        .collect()
}

/// Todo lo que se puede leer de `TOP`, en el orden en que sale.
fn snapshot(a: &Library, b: &Library) -> (Flat, [u64; 4], Vec<GdsTag>, Flat, Flat) {
    let (ca, cb) = (a.find_cell("TOP").unwrap(), b.find_cell("TOP").unwrap());
    let bb = ca.bbox();
    let mut xor = (Vec::new(), Vec::new());
    for tag in a.layers() {
        let fa = ca.get_polygons().with_filter(tag.layer, tag.datatype).build();
        let fb = cb.get_polygons().with_filter(tag.layer, tag.datatype).build();
        let split = xor_split_flat(&fa, &fb);
        xor.0.extend(owned_bits(&split.added));
        xor.1.extend(owned_bits(&split.removed));
    }
    (
        flat(a, "TOP"),
        [bb.min_x.to_bits(), bb.min_y.to_bits(), bb.max_x.to_bits(), bb.max_y.to_bits()],
        a.layers(),
        xor.0,
        xor.1,
    )
}

#[test]
fn layers_include_path_only_layers() {
    let lib = Library::from_bytes(&gds(0)).expect("gds");
    let tags: Vec<(u32, u32)> = lib.layers().iter().map(|t| (t.layer, t.datatype)).collect();
    assert_eq!(tags, vec![(1, 0), (2, 0), (3, 0)]);
}

#[test]
fn repeated_path_points_are_removed_at_load() {
    // El path de SUB tiene 5 puntos, uno repetido: queda con 4 al cargar,
    // antes de aplanar nada.
    let lib = Library::from_bytes(&gds(0)).expect("gds");
    let sub = lib.find_cell("SUB").unwrap();
    let fp = sub.flexpaths().next().expect("path");
    assert_eq!(fp.spine_point_count(), 4);
}

#[test]
fn flatten_and_xor_from_many_threads_match_one_thread() {
    let (a, b) = (Library::from_bytes(&gds(0)).unwrap(), Library::from_bytes(&gds(500)).unwrap());
    let expected = snapshot(&a, &b);
    // 1 SREF + 12 de la AREF, cada una con un rectángulo y un path, más lo
    // propio de TOP; y el rectángulo movido aparece en el XOR.
    assert_eq!(expected.0.len(), 2 * 13 + 2);
    assert!(!expected.3.is_empty() && !expected.4.is_empty());

    std::thread::scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|| {
                for _ in 0..ROUNDS {
                    assert_eq!(snapshot(&a, &b), expected);
                }
            });
        }
    });
}

#[test]
fn proof_lib_flattens_the_same_from_many_threads() {
    let lib = Library::open(&proof_lib_path());
    let names: Vec<String> = lib.cells().map(|c| c.name().to_string()).collect();
    let expected: Vec<Flat> = names.iter().map(|n| flat(&lib, n)).collect();
    std::thread::scope(|s| {
        for t in 0..THREADS {
            let (lib, names, expected) = (&lib, &names, &expected);
            s.spawn(move || {
                for r in 0..ROUNDS / 4 {
                    // Cada hilo recorre las celdas desde otro punto.
                    for k in 0..names.len() {
                        let i = (k + t + r) % names.len();
                        assert_eq!(flat(lib, &names[i]), expected[i], "{}", names[i]);
                    }
                }
            });
        }
    });
}

#[test]
fn library_moves_to_another_thread() {
    let lib = Library::from_bytes(&gds(0)).unwrap();
    let n = std::thread::spawn(move || flat(&lib, "TOP").len()).join().unwrap();
    assert_eq!(n, 2 * 13 + 2);
}
