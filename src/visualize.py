import seaborn as sns
from pathlib import Path
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import pandas as pd


CONDITION_LABELS = {
    "A": "A: compiler stderr",
    "B": "B: LSP diagnostics",
}
CONDITION_PALETTE = {
    "A: compiler stderr": "#4C72B0",
    "B: LSP diagnostics": "#DD8452",
}


def _label_conditions(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()
    df["condition"] = df["condition"].map(lambda c: CONDITION_LABELS.get(c, c))
    return df


def _annotate_n(ax, df: pd.DataFrame) -> None:
    """Write the per-condition sample size under each x-tick."""
    counts = df.groupby("condition").size()
    labels = [t.get_text() for t in ax.get_xticklabels()]
    ax.set_xticklabels([f"{lbl}\n(n={counts.get(lbl, 0)})" for lbl in labels])


def visualize_results(results_path: Path, output_path: Path) -> None:
    df: pd.DataFrame = pd.read_json(results_path, lines=True)
    if df.empty:
        return
    output_path.mkdir(parents=True, exist_ok=True)

    df = _label_conditions(df)
    order = [v for v in CONDITION_LABELS.values() if v in df["condition"].unique()]

    sns.set_theme(style="whitegrid", context="talk", font_scale=0.9)

    # Summary stats table — saved alongside plots so examiners get exact numbers.
    summary = df.groupby("condition").agg(
        n=("success", "size"),
        success_rate=("success", "mean"),
        mean_iterations=("iterations_used", "mean"),
        median_iterations=("iterations_used", "median"),
        mean_wall_time_s=("wall_time_seconds", "mean"),
        median_wall_time_s=("wall_time_seconds", "median"),
    ).round(3)
    summary.to_csv(output_path / "summary.csv")

    # 1. Success rate by condition (headline result)
    fig, ax = plt.subplots(figsize=(7, 5))
    sns.barplot(
        data=df, x="condition", y="success",
        order=order, palette=CONDITION_PALETTE,
        errorbar="ci", capsize=0.15, err_kws={"linewidth": 1.5}, ax=ax,
    )
    ax.set_ylim(0, 1.05)
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylabel("Compilation success rate")
    ax.set_xlabel("")
    ax.set_title("Compilation success by feedback condition")
    for container in ax.containers:
        ax.bar_label(container, fmt="%.0f%%",
                     labels=[f"{v*100:.0f}%" for v in container.datavalues],
                     padding=4, fontsize=11)
    _annotate_n(ax, df)
    fig.tight_layout()
    fig.savefig(output_path / "success_rate.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    # 2. Iterations used by condition
    fig, ax = plt.subplots(figsize=(7, 5))
    sns.boxplot(
        data=df, x="condition", y="iterations_used",
        order=order, palette=CONDITION_PALETTE,
        width=0.5, fliersize=0, ax=ax,
    )
    sns.stripplot(
        data=df, x="condition", y="iterations_used",
        order=order, color="black", size=4, alpha=0.6, jitter=0.15, ax=ax,
    )
    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_ylim(bottom=-0.5)
    ax.set_ylabel("Repair iterations used")
    ax.set_xlabel("")
    ax.set_title("Repair iterations by feedback condition")
    _annotate_n(ax, df)
    fig.tight_layout()
    fig.savefig(output_path / "iterations.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    # 3. Wall-clock time by condition
    fig, ax = plt.subplots(figsize=(7, 5))
    sns.boxplot(
        data=df, x="condition", y="wall_time_seconds",
        order=order, palette=CONDITION_PALETTE,
        width=0.5, fliersize=0, ax=ax,
    )
    sns.stripplot(
        data=df, x="condition", y="wall_time_seconds",
        order=order, color="black", size=4, alpha=0.6, jitter=0.15, ax=ax,
    )
    ax.set_ylabel("Wall-clock time (seconds)")
    ax.set_xlabel("")
    ax.set_title("Wall-clock time by feedback condition")
    _annotate_n(ax, df)
    fig.tight_layout()
    fig.savefig(output_path / "wall_time.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    # 4. Iterations vs source size, split by condition
    fig, ax = plt.subplots(figsize=(8, 5))
    sns.scatterplot(
        data=df, x="loc", y="iterations_used",
        hue="condition", hue_order=order, palette=CONDITION_PALETTE,
        style="success", style_order=[True, False],
        markers={True: "o", False: "X"},
        s=120, alpha=0.85, ax=ax,
    )
    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_ylim(bottom=-0.5)
    ax.set_xlabel("Source lines of code (C++)")
    ax.set_ylabel("Repair iterations used")
    ax.set_title("Repair iterations vs. program size")
    ax.legend(title="", loc="best", frameon=True)
    fig.tight_layout()
    fig.savefig(output_path / "iterations_vs_loc.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    # 5. Per-unit success heatmap (only meaningful with both conditions)
    if df["condition"].nunique() > 1:
        pivot = df.pivot_table(
            index="unit_id", columns="condition",
            values="success", aggfunc="mean",
        ).reindex(columns=order)
        fig, ax = plt.subplots(figsize=(6, max(3, 0.4 * len(pivot))))
        sns.heatmap(
            pivot, annot=True, fmt=".0%",
            cmap="RdYlGn", vmin=0, vmax=1,
            cbar_kws={"label": "Success rate"}, ax=ax,
        )
        ax.set_xlabel("")
        ax.set_ylabel("Translation unit")
        ax.set_title("Per-unit success rate by condition")
        fig.tight_layout()
        fig.savefig(output_path / "per_unit_heatmap.png", dpi=150, bbox_inches="tight")
        plt.close(fig)

    # 6. Distribution of iterations used
    fig, ax = plt.subplots(figsize=(8, 5))
    sns.histplot(
        data=df, x="iterations_used", hue="condition",
        hue_order=order, palette=CONDITION_PALETTE,
        multiple="dodge", discrete=True, shrink=0.85, ax=ax,
    )
    ax.xaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_xlabel("Repair iterations used")
    ax.set_ylabel("Number of runs")
    ax.set_title("Distribution of repair iterations")
    fig.tight_layout()
    fig.savefig(output_path / "iterations_hist.png", dpi=150, bbox_inches="tight")
    plt.close(fig)
