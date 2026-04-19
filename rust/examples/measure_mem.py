"""Mide memoria pico de un proceso externo.

Uso:
    python measure_mem.py <cmd> [args...]

Lanza el comando, hace polling cada 5 ms del working set
(y de cada hijo, útil para multiprocessing), e imprime el pico.
"""
import subprocess
import sys
import time
import psutil


def peak_rss(cmd: list[str]) -> tuple[float, float, int]:
    """Returns (wall_seconds, peak_mb, exit_code).

    Peak considers the main process PLUS any children it spawned
    (handles Python multiprocessing which forks N workers)."""
    t0 = time.perf_counter()
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    p = psutil.Process(proc.pid)
    peak = 0
    try:
        while proc.poll() is None:
            try:
                rss_total = p.memory_info().rss
                for child in p.children(recursive=True):
                    try:
                        rss_total += child.memory_info().rss
                    except psutil.NoSuchProcess:
                        pass
                if rss_total > peak:
                    peak = rss_total
            except psutil.NoSuchProcess:
                break
            time.sleep(0.005)
    finally:
        proc.wait()
    wall = time.perf_counter() - t0
    return wall, peak / (1024 * 1024), proc.returncode


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: measure_mem.py <cmd> [args...]", file=sys.stderr)
        sys.exit(2)
    wall, peak_mb, rc = peak_rss(sys.argv[1:])
    print(f"{wall * 1000:7.1f} ms  {peak_mb:7.1f} MB peak  (exit={rc})")


if __name__ == "__main__":
    main()
