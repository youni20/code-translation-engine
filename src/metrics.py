from dataclasses import dataclass
from statistics import mean, median


@dataclass(frozen=True)
class CompilationMetrics:
    n_runs: int
    n_compiled: int
    compilation_rate: float
    n_first_try: int
    n_exhausted: int
    mean_iterations_to_compile: float | None
    median_iterations_to_compile: float | None


def compute_metrics(runs: list[tuple[str, bool, int]]) -> CompilationMetrics:
    """Aggregate pipeline outputs into compilation metrics.

    Each run is `(rust_code, compiled, iterations_used)` as returned by
    `run_translation_pipeline`. `iterations_used` is 0 when the initial
    translation compiled, and equals `max_iterations` when the loop
    exhausted without success (or succeeded on the final attempt).
    """
    if not runs:
        raise ValueError("compute_metrics requires at least one run")

    n_runs = len(runs)
    compiled_iters = [iters for _, ok, iters in runs if ok]
    n_compiled = len(compiled_iters)
    n_first_try = sum(1 for _, ok, iters in runs if ok and iters == 0)
    n_exhausted = sum(1 for _, ok, _ in runs if not ok)

    return CompilationMetrics(
        n_runs=n_runs,
        n_compiled=n_compiled,
        compilation_rate=n_compiled / n_runs,
        n_first_try=n_first_try,
        n_exhausted=n_exhausted,
        mean_iterations_to_compile=mean(compiled_iters) if compiled_iters else None,
        median_iterations_to_compile=median(compiled_iters) if compiled_iters else None,
    )
