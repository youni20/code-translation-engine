from pathlib import Path

from dotenv import load_dotenv

load_dotenv()

from config import ExperimentConfig
from experiment import run_experiment


if __name__ == "__main__":
    config = ExperimentConfig(
        projects_dir=Path("./tests/tests_cpp"),
        conditions=("A", "B"),  # For only one condition do:  conditions=("A",),  
        repetitions=1,
        max_iterations=3,
        translator_model="gpt-4o-mini",
        repair_model="gpt-4o-mini",
    )
    run_experiment(config)