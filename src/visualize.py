import seaborn as sns
from pathlib import Path
from results import RunResult, load_results
from matplotlib import pyplot
import pandas as pd

def visualize_results(results_path: Path, output_path: Path) -> None:
    #  results: list[RunResult] = load_results(results_path=results_path)
    df: pd.DataFrame = pd.read_json(results_path, lines=True)
    output_path.mkdir(parents=True, exist_ok=True)
    #  print(df) 

    sns.set_theme()

    