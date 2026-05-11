from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class TranslationUnit:
    project: str
    relative_path: str
    source_code: str
    loc: int

    @property
    def unit_id(self) -> str:
        return f"{self.project}/{self.relative_path}"


def load_project(project_dir: Path) -> list[TranslationUnit]:
    """Walk a single project directory and return its translation units."""
    if not project_dir.is_dir():
        raise ValueError(f"Not a directory: {project_dir}")

    extensions = {".cpp", ".hpp", ".h", ".cc"}
    units: list[TranslationUnit] = []

    for path in sorted(project_dir.rglob("*")):
        if not path.is_file() or path.suffix not in extensions:
            continue
        source = path.read_text(encoding="utf-8", errors="replace")
        units.append(TranslationUnit(
            project=project_dir.name,
            relative_path=str(path.relative_to(project_dir)),
            source_code=source,
            loc=len(source.splitlines()),
        ))
    return units


def load_all_projects(projects_dir: Path) -> list[TranslationUnit]:
    """Iterate over every project subdirectory and concatenate their units."""
    if not projects_dir.is_dir():
        raise ValueError(f"Not a directory: {projects_dir}")

    all_units: list[TranslationUnit] = []
    for project_dir in sorted(projects_dir.iterdir()):
        if project_dir.is_dir():
            all_units.extend(load_project(project_dir))
    return all_units