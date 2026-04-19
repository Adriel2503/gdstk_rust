"""Python con multiprocessing — un proceso por archivo.

Única forma de paralelizar de verdad en Python con gdstk."""
import sys
from concurrent.futures import ProcessPoolExecutor
import gdstk


def count(path: str) -> int:
    return len(gdstk.read_gds(path).cells)


def main() -> None:
    files = sys.argv[1:]
    with ProcessPoolExecutor(max_workers=len(files)) as ex:
        total = sum(ex.map(count, files))
    print(total)


if __name__ == "__main__":
    main()
