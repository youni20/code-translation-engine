from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path


# LSP workspace paths (used by lsp_tool.py)
LSP_PROJECT_DIR = str(Path("./outputs/rust_workspace").resolve())
LSP_RELATIVE_FILE = "src/output.rs"
LSP_OUTPUT_PATH = f"{LSP_PROJECT_DIR}/{LSP_RELATIVE_FILE}"


@dataclass(frozen=True)
class ExperimentConfig:
    """All parameters defining one experimental run."""

    # Dataset
    projects_dir: Path

    # Experimental variables
    conditions: tuple[str, ...] = ("A", "B")
    repetitions: int = 3
    max_iterations: int = 5

    # Models
    translator_model: str = "gpt-4o-mini"
    repair_model: str = "gpt-4o-mini"

    # Output
    output_root: Path = field(default_factory=lambda: Path("./outputs/runs"))
    run_id: str = field(default_factory=lambda: datetime.now().strftime("%Y-%m-%d_%H-%M-%S"))

    @property
    def run_dir(self) -> Path:
        """Directory for this specific run's outputs."""
        return self.output_root / self.run_id

    @property
    def results_path(self) -> Path:
        return self.run_dir / "results.jsonl"

    @property
    def config_snapshot_path(self) -> Path:
        return self.run_dir / "config.json"

    @property
    def translations_dir(self) -> Path:
        return self.run_dir / "translations"