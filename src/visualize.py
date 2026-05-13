import seaborn as sns
from pathlib import Path
import matplotlib.pyplot as plt
import pandas as pd

def visualize_results(results_path: Path, output_path: Path) -> None:
    #  results: list[RunResult] = load_results(results_path=results_path)
    df: pd.DataFrame = pd.read_json(results_path, lines=True)
    if(df.empty):
        return
    output_path.mkdir(parents=True, exist_ok=True)
    #  print(df) 

    sns.set_theme(style="whitegrid")
    fig, ax = plt.subplots()
    sns.barplot(data=df, x="condition", y="success", errorbar="ci", ax=ax)
    ax.set_ylim(0, 1); ax.set_ylabel("Success rate")
    fig.savefig(output_path / "success_rate.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    