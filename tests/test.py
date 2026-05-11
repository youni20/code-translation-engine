from pathlib import Path
from results import RunResult, ResultsWriter, load_results

writer = ResultsWriter(Path("./tmp_test_results.jsonl"))
r = RunResult(
    unit_id="test_project/file.cpp",
    project="test_project",
    relative_path="file.cpp",
    loc=42,
    condition="A",
    repetition=0,
    success=True,
    iterations_used=2,
    final_rust_code="fn main() {}",
    final_stderr="",
    feedback_history=["error: x", "error: y"],
    wall_time_seconds=5.3,
)
writer.append(r)

loaded = load_results(Path("./tmp_test_results.jsonl"))
print(loaded[0])

Path("./tmp_test_results.jsonl").unlink()  # cleanup