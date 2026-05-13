import seaborn as sn
from pathlib import Path
from results import RunResult, load_results
import pandas as pd

def visualize_results(results_path: Path, output_path: Path) -> None:
    #  results: list[RunResult] = load_results(results_path=results_path)
    df: pd.DataFrame = pd.read_json(results_path, lines=True)
    print(df)    