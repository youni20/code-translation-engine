from pathlib import Path

from dotenv import load_dotenv

from experiment import run_experiment
from visualize import visualize_results

load_dotenv()

from config import ExperimentConfig

if __name__ == "__main__":
    config = ExperimentConfig(
        projects_dir=Path("./inputs/projects"),
        conditions=("A", "B"),  # For only one condition do:  conditions=("A",),
        repetitions=3,
        max_iterations=3,
        translator_model="gpt-4o-mini",
        repair_model="gpt-4o-mini",
    )
    run_experiment(config)
    visualize_results(results_path=config.results_path, output_path=config.run_dir)
