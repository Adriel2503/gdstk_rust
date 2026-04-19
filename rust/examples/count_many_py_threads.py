"""Python con threading — un thread por archivo.

Como gdstk C++ no libera el GIL, esto NO paraleliza de verdad;
es la contraparte natural del count_many_parallel.rs."""
import sys
from concurrent.futures import ThreadPoolExecutor
import gdstk


def count(path: str) -> int:
    return len(gdstk.read_gds(path).cells)


def main() -> None:
    files = sys.argv[1:]
    with ThreadPoolExecutor(max_workers=len(files)) as ex:
        total = sum(ex.map(count, files))
    print(total)


if __name__ == "__main__":
    main()
