from pathlib import Path

from dotenv import load_dotenv

load_dotenv()

from config import ExperimentConfig
from experiment import run_experiment


if __name__ == "__main__":
    config = ExperimentConfig(
        projects_dir=Path("./inputs/projects"),
        conditions=("A", "B"),
        repetitions=3,
        max_iterations=5,
        translator_model="gpt-4o-mini",
        repair_model="gpt-4o-mini",
    )
    run_experiment(config)