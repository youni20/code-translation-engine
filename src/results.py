import json
from dataclasses import dataclass, asdict, field
from datetime import datetime, timezone
from pathlib import Path


@dataclass(frozen=True)
class RunResult:
    unit_id: str
    project: str
    relative_path: str
    loc: int

    # Experimental variables
    condition: str
    repetition: int

    # Outcomes
    success: bool
    iterations_used: int
    final_rust_code: str
    final_stderr: str
    feedback_history: list[str] = field(default_factory=list)

    # Telemetry
    wall_time_seconds: float = 0.0
    timestamp: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )

    def to_json_line(self) -> str:
        """Serialise as a single-line JSON string for JSONL output."""
        return json.dumps(asdict(self), ensure_ascii=False)


class ResultsWriter:
    """Append-only JSONL writer for RunResult records."""

    def __init__(self, results_path: Path) -> None:
        self.results_path = results_path
        results_path.parent.mkdir(parents=True, exist_ok=True)

    def append(self, result: RunResult) -> None:
        with self.results_path.open("a", encoding="utf-8") as f:
            f.write(result.to_json_line() + "\n")


def load_results(results_path: Path) -> list[RunResult]:
    """Read a JSONL file back into RunResult objects for analysis."""
    if not results_path.is_file():
        raise FileNotFoundError(f"No results file at {results_path}")

    results: list[RunResult] = []
    with results_path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            data = json.loads(line)
            results.append(RunResult(**data))
    return results