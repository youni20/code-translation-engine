import json
from dataclasses import asdict
from pathlib import Path

from agent import Agno_Agent
from config import ExperimentConfig
from dataset import TranslationUnit, load_all_projects
from pipeline import run_translation_pipeline
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT
from results import ResultsWriter, RunResult


def snapshot_config(config: ExperimentConfig) -> None:
    """Persist the experiment config to disk for reproducibility."""
    config.config_snapshot_path.parent.mkdir(parents=True, exist_ok=True)
    serialisable = {
        k: str(v) if isinstance(v, Path) else v
        for k, v in asdict(config).items()
    }
    config.config_snapshot_path.write_text(
        json.dumps(serialisable, indent=2), encoding="utf-8"
    )


def save_translation(config: ExperimentConfig, result: RunResult) -> None:
    """Write the final Rust code for one run to disk for inspection."""
    out_path = (
        config.translations_dir
        / result.project
        / f"{Path(result.relative_path).stem}.cond_{result.condition}.rep_{result.repetition}.rs"
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(result.final_rust_code, encoding="utf-8")


def run_experiment(config: ExperimentConfig) -> None:
    """Execute the full experiment defined by config."""
    snapshot_config(config)

    units: list[TranslationUnit] = load_all_projects(config.projects_dir)
    if not units:
        raise RuntimeError(f"No translation units found under {config.projects_dir}")

    translator = Agno_Agent(
        model_id=config.translator_model,
        description=TRANSLATION_SYSTEM_PROMPT,
    )
    repairer = Agno_Agent(
        model_id=config.repair_model,
        description=REPAIR_SYSTEM_PROMPT,
    )

    writer = ResultsWriter(config.results_path)

    total_invocations = len(units) * len(config.conditions) * config.repetitions
    invocation_idx = 0

    print("=" * 70)
    print(f"  Run ID:            {config.run_id}")
    print(f"  Units:             {len(units)}")
    print(f"  Conditions:        {', '.join(config.conditions)}")
    print(f"  Repetitions:       {config.repetitions}")
    print(f"  Total invocations: {total_invocations}")
    print(f"  Results file:      {config.results_path}")
    print("=" * 70)

    for unit in units:
        for condition in config.conditions:
            for repetition in range(config.repetitions):
                invocation_idx += 1
                prefix = f"[{invocation_idx}/{total_invocations}]"
                print()
                print(f"{prefix} {unit.unit_id}  (cond={condition}, rep={repetition+1})")

                try:
                    result = run_translation_pipeline(
                        unit=unit,
                        translator=translator,
                        repairer=repairer,
                        condition=condition,
                        repetition=repetition,
                        max_iterations=config.max_iterations,
                    )
                except Exception as e:
                    print(f"  [ERROR]     {type(e).__name__}: {e}")
                    continue

                writer.append(result)
                save_translation(config, result)
                outcome = "SUCCESS" if result.success else "FAILED "
                print(
                    f"  [done]      {outcome}  "
                    f"iters={result.iterations_used}  "
                    f"time={result.wall_time_seconds:.1f}s"
                )

    print()
    print("=" * 70)
    print(f"  Experiment complete. Results: {config.results_path}")
    print("=" * 70)