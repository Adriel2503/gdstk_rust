"""Compara Python gdstk: secuencial, threading (GIL), multiprocessing.

Mismo workload que count_many.rs: N archivos GDS, imprime celdas y tiempo.
"""
import sys
import time
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor

import gdstk


def count_one(path: str) -> int:
    return len(gdstk.read_gds(path).cells)


def main() -> None:
    files = sys.argv[1:]
    if not files:
        print("usage: bench_python.py <file1.gds> ...", file=sys.stderr)
        sys.exit(2)

    print(f"{len(files)} archivos GDS\n")

    # Secuencial
    t0 = time.perf_counter()
    counts = [count_one(f) for f in files]
    seq_elapsed = time.perf_counter() - t0
    for f, c in zip(files, counts):
        name = f.rsplit("/", 1)[-1]
        print(f"  {name:30} {c} celdas")
    print(f"\nSecuencial:     {seq_elapsed * 1000:.1f} ms")

    # Threading — bloqueado por el GIL si el C++ no lo libera
    t1 = time.perf_counter()
    with ThreadPoolExecutor(max_workers=len(files)) as ex:
        list(ex.map(count_one, files))
    thread_elapsed = time.perf_counter() - t1
    print(f"Threading:      {thread_elapsed * 1000:.1f} ms "
          f"(speedup {seq_elapsed / thread_elapsed:.2f}x)")

    # Multiprocessing — real, pero paga overhead de spawn de procesos
    t2 = time.perf_counter()
    with ProcessPoolExecutor(max_workers=len(files)) as ex:
        list(ex.map(count_one, files))
    mp_elapsed = time.perf_counter() - t2
    print(f"Multiprocessing:{mp_elapsed * 1000:.1f} ms "
          f"(speedup {seq_elapsed / mp_elapsed:.2f}x)")


if __name__ == "__main__":
    main()
