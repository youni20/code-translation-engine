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


    # 1. Success rate by condition (headline result)
    fig, ax = plt.subplots()
    sns.barplot(data=df, x="condition", y="success", errorbar="ci", ax=ax)
    ax.set_ylim(0, 1); ax.set_ylabel("Success rate")
    ax.set_title("Compilation success by feedback condition")
    fig.savefig(output_path / "success_rate.png", dpi=150, bbox_inches="tight")
    plt.close(fig)


    # 2. Iterations used by condition (how many repair rounds did each need)
    fig, ax = plt.subplots()
    sns.boxplot(data=df, x="condition", y="iterations_used", ax=ax)
    sns.stripplot(data=df, x="condition", y="iterations_used",
                  color="black", size=3, alpha=0.5, ax=ax)
    ax.set_ylabel("Iterations used")
    ax.set_title("Repair iterations by feedback condition")
    fig.savefig(output_path / "iterations.png", dpi=150, bbox_inches="tight")
    plt.close(fig)


    # 3. Wall-clock time by condition (compute cost)
    fig, ax = plt.subplots()
    sns.boxplot(data=df, x="condition", y="wall_time_seconds", ax=ax)
    ax.set_ylabel("Wall time (s)")
    ax.set_title("Wall-clock time by feedback condition")
    fig.savefig(output_path / "wall_time.png", dpi=150, bbox_inches="tight")
    plt.close(fig)


    # 4. Iterations vs program size, split by condition (does feedback help on harder code?)
    fig, ax = plt.subplots()
    sns.scatterplot(data=df, x="loc", y="iterations_used",
                    hue="condition", style="success", s=80, ax=ax)
    ax.set_xlabel("Source LOC"); ax.set_ylabel("Iterations used")
    ax.set_title("Iterations vs source size")
    fig.savefig(output_path / "iterations_vs_loc.png", dpi=150, bbox_inches="tight")
    plt.close(fig)


    # 5. Per-unit success heatmap (which specific files flip between conditions)
    if df["condition"].nunique() > 1:
        pivot = df.pivot_table(index="unit_id", columns="condition",
                               values="success", aggfunc="mean")
        fig, ax = plt.subplots(figsize=(6, max(3, 0.3 * len(pivot))))
        sns.heatmap(pivot, annot=True, fmt=".2f",
                    cmap="RdYlGn", vmin=0, vmax=1, ax=ax)
        ax.set_title("Per-unit success rate by condition")
        fig.savefig(output_path / "per_unit_heatmap.png", dpi=150, bbox_inches="tight")
        plt.close(fig)


    # 6. Distribution of iterations used (shape of the convergence)
    fig, ax = plt.subplots()
    sns.histplot(data=df, x="iterations_used", hue="condition",
                 multiple="dodge", discrete=True, ax=ax)
    ax.set_xlabel("Iterations used"); ax.set_ylabel("Count")
    ax.set_title("Distribution of repair iterations")
    fig.savefig(output_path / "iterations_hist.png", dpi=150, bbox_inches="tight")
    plt.close(fig)
