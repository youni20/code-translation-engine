from pathlib import Path


def local_file_reader(path: str | Path) -> str:
    return Path(path).read_text(encoding="utf-8")


def local_file_writer(content: str, path: str | Path) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")