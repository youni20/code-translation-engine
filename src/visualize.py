from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import numpy as np
import pandas as pd
import seaborn as sns
from scipy import stats

CONDITION_LABELS = {
    "A": "A: compiler stderr",
    "B": "B: LSP diagnostics",
}

# Colourblind-safe palette (Wong, 2011). Avoid Seaborn defaults for print.
CONDITION_PALETTE = {
    "A: compiler stderr": "#0072B2",
    "B: LSP diagnostics": "#D55E00",
}

# IEEE / ACM two-column widths in inches.
SINGLE_COLUMN_IN = 3.5
DOUBLE_COLUMN_IN = 7.16

# Minimum sample sizes below which a plot is considered uninformative.
MIN_RUNS_FOR_DISTRIBUTION = 6
MIN_UNITS_FOR_HEATMAP = 3
MIN_RUNS_FOR_CI = 3


@dataclass(frozen=True)
class FigureSpec:
    """Container for the dimensions and naming of a single figure."""
    name: str
    width_in: float
    height_in: float


# --------------------------------------------------------------------------- #
# Matplotlib styling
# --------------------------------------------------------------------------- #

def _configure_publication_style() -> None:
    """Set rcParams for camera-ready figures.

    The font stack falls back gracefully when LaTeX is unavailable.
    Sizes target 9-10 pt captions, matching IEEE conference templates.
    """
    sns.set_theme(style="whitegrid", context="paper")
    plt.rcParams.update({
        "font.family": "serif",
        "font.serif": ["Times New Roman", "Nimbus Roman", "DejaVu Serif"],
        "mathtext.fontset": "stix",
        "axes.titlesize": 10,
        "axes.labelsize": 9,
        "xtick.labelsize": 8,
        "ytick.labelsize": 8,
        "legend.fontsize": 8,
        "legend.frameon": False,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "axes.linewidth": 0.6,
        "grid.linewidth": 0.4,
        "grid.alpha": 0.4,
        "lines.linewidth": 1.0,
        "savefig.bbox": "tight",
        "savefig.pad_inches": 0.02,
        "pdf.fonttype": 42,  # TrueType, required by most publishers
        "ps.fonttype": 42,
    })


def _save(fig: plt.Figure, output_path: Path, name: str) -> None:
    """Save in both PDF (vector) and PNG (300 dpi) for flexibility."""
    fig.savefig(output_path / f"{name}.pdf")
    fig.savefig(output_path / f"{name}.png", dpi=300)
    plt.close(fig)


# --------------------------------------------------------------------------- #
# Data preparation
# --------------------------------------------------------------------------- #

def _label_conditions(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()
    df["condition"] = df["condition"].map(lambda c: CONDITION_LABELS.get(c, c))
    return df


def _ordered_conditions(df: pd.DataFrame) -> list[str]:
    return [v for v in CONDITION_LABELS.values() if v in df["condition"].unique()]


def _annotate_n(ax: plt.Axes, df: pd.DataFrame, order: Iterable[str]) -> None:
    counts = df.groupby("condition").size()
    ax.set_xticks(range(len(list(order))))
    ax.set_xticklabels(
        [f"{lbl}\n$n={counts.get(lbl, 0)}$" for lbl in order]
    )


# --------------------------------------------------------------------------- #
# Statistical reporting
# --------------------------------------------------------------------------- #

def _per_unit_outcomes(df: pd.DataFrame) -> pd.DataFrame:
    """Aggregate replicate runs into per-unit, per-condition success rates."""
    return (
        df.groupby(["unit_id", "condition"])["success"]
        .mean()
        .unstack("condition")
    )


def _mcnemar_table(df: pd.DataFrame, order: list[str]) -> dict | None:
    """Build the 2x2 contingency table and run McNemar's test.

    Returns None if pairing is impossible (e.g. only one condition present
    or no unit appears in both conditions).
    """
    if len(order) < 2:
        return None

    per_unit = _per_unit_outcomes(df)
    paired = per_unit.dropna(subset=order)
    if paired.empty:
        return None

    # Binarise per-unit outcome by majority across replicates.
    a = (paired[order[0]] >= 0.5).astype(int)
    b = (paired[order[1]] >= 0.5).astype(int)

    b00 = int(((a == 0) & (b == 0)).sum())
    b01 = int(((a == 0) & (b == 1)).sum())  # B succeeds where A fails
    b10 = int(((a == 1) & (b == 0)).sum())  # A succeeds where B fails
    b11 = int(((a == 1) & (b == 1)).sum())

    # Exact binomial McNemar (preferred for small discordant counts).
    n_discordant = b01 + b10
    if n_discordant == 0:
        p_value = 1.0
    else:
        p_value = stats.binomtest(
            k=min(b01, b10), n=n_discordant, p=0.5, alternative="two-sided"
        ).pvalue

    return {
        "table": np.array([[b00, b01], [b10, b11]]),
        "n_units_paired": int(len(paired)),
        "p_value": float(p_value),
        "discordant": n_discordant,
        "b_better": b01,
        "a_better": b10,
        "order": order,
    }


def _write_summary(df: pd.DataFrame, output_path: Path, order: list[str]) -> None:
    summary = df.groupby("condition").agg(
        n_runs=("success", "size"),
        n_units=("unit_id", "nunique"),
        success_rate=("success", "mean"),
        success_rate_sd=("success", "std"),
        mean_iterations=("iterations_used", "mean"),
        median_iterations=("iterations_used", "median"),
        mean_wall_time_s=("wall_time_seconds", "mean"),
        median_wall_time_s=("wall_time_seconds", "median"),
    ).round(4)
    summary.to_csv(output_path / "summary.csv")

    mcnemar = _mcnemar_table(df, order)
    if mcnemar is not None:
        with (output_path / "mcnemar.txt").open("w") as f:
            f.write("McNemar's test (exact binomial)\n")
            f.write("=" * 40 + "\n")
            f.write(f"Paired units: {mcnemar['n_units_paired']}\n")
            f.write(f"A better than B: {mcnemar['a_better']}\n")
            f.write(f"B better than A: {mcnemar['b_better']}\n")
            f.write(f"Discordant pairs: {mcnemar['discordant']}\n")
            f.write(f"p-value: {mcnemar['p_value']:.4f}\n\n")
            f.write("Contingency table (rows = A outcome, cols = B outcome)\n")
            f.write(pd.DataFrame(
                mcnemar["table"],
                index=["A: fail", "A: pass"],
                columns=["B: fail", "B: pass"],
            ).to_string())


# --------------------------------------------------------------------------- #
# Plotting primitives
# --------------------------------------------------------------------------- #

def _plot_success_rate(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    fig, ax = plt.subplots(figsize=(SINGLE_COLUMN_IN, 2.6))

    n_per_cond = df.groupby("condition").size()
    use_ci = (n_per_cond >= MIN_RUNS_FOR_CI).all()

    sns.stripplot(
        data=df, x="condition", y="success",
        order=order, palette=CONDITION_PALETTE,
        jitter=0.15, alpha=0.4, size=4, ax=ax, legend=False,
    )
    sns.pointplot(
        data=df, x="condition", y="success",
        order=order, palette=CONDITION_PALETTE,
        errorbar=("ci", 95) if use_ci else None,
        capsize=0.1, markers="D", linestyle="none",
        err_kws={"linewidth": 1.2}, ax=ax,
    )

    means = df.groupby("condition")["success"].mean().reindex(order)
    for i, m in enumerate(means.values):
        ax.text(i, min(m + 0.08, 1.08), f"{m * 100:.0f}\\%",
                ha="center", va="bottom", fontsize=8, fontweight="bold")

    ax.set_ylim(-0.08, 1.18)
    ax.set_yticks([0, 0.25, 0.5, 0.75, 1.0])
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylabel("Compilation success rate")
    ax.set_xlabel("")
    _annotate_n(ax, df, order)
    fig.tight_layout()
    _save(fig, output_path, "success_rate")


def _plot_iterations(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    n_per_cond = df.groupby("condition").size()
    use_box = (n_per_cond >= MIN_RUNS_FOR_DISTRIBUTION).all()

    fig, ax = plt.subplots(figsize=(SINGLE_COLUMN_IN, 2.6))

    if use_box:
        sns.boxplot(
            data=df, x="condition", y="iterations_used",
            order=order, palette=CONDITION_PALETTE,
            width=0.45, fliersize=0, linewidth=0.8, ax=ax,
        )
        sns.stripplot(
            data=df, x="condition", y="iterations_used",
            order=order, color="black", size=2.5, alpha=0.6,
            jitter=0.12, ax=ax,
        )
    else:
        # Fall back to a bar of the mean with raw points overlaid.
        sns.barplot(
            data=df, x="condition", y="iterations_used",
            order=order, palette=CONDITION_PALETTE,
            errorbar=None, alpha=0.5, ax=ax,
        )
        sns.stripplot(
            data=df, x="condition", y="iterations_used",
            order=order, color="black", size=3.5, alpha=0.8,
            jitter=0.12, ax=ax,
        )

    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_ylim(bottom=-0.3)
    ax.set_ylabel("Repair iterations")
    ax.set_xlabel("")
    _annotate_n(ax, df, order)
    fig.tight_layout()
    _save(fig, output_path, "iterations")


def _plot_paired_deltas(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Per-unit paired comparison: the headline statistical figure.

    Each translation unit appears once, with a line connecting its
    success rate under A and under B. This makes the paired structure
    of McNemar's test visually explicit.
    """
    if len(order) < 2:
        return

    per_unit = _per_unit_outcomes(df).dropna(subset=order)
    if per_unit.empty:
        return

    fig, ax = plt.subplots(figsize=(SINGLE_COLUMN_IN, 3.0))

    for unit_id, row in per_unit.iterrows():
        a_val, b_val = row[order[0]], row[order[1]]
        colour = "#009E73" if b_val > a_val else "#CC79A7" if b_val < a_val else "grey"
        ax.plot([0, 1], [a_val, b_val], color=colour, alpha=0.6,
                linewidth=0.8, marker="o", markersize=3)

    ax.set_xticks([0, 1])
    ax.set_xticklabels(order)
    ax.set_ylabel("Per-unit success rate")
    ax.set_ylim(-0.05, 1.05)
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_xlim(-0.3, 1.3)
    fig.tight_layout()
    _save(fig, output_path, "paired_deltas")


def _plot_iterations_vs_loc(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    if len(df) < MIN_RUNS_FOR_DISTRIBUTION:
        return

    fig, ax = plt.subplots(figsize=(DOUBLE_COLUMN_IN * 0.6, 2.8))
    sns.scatterplot(
        data=df, x="loc", y="iterations_used",
        hue="condition", hue_order=order, palette=CONDITION_PALETTE,
        style="success", style_order=[True, False],
        markers={True: "o", False: "X"},
        s=55, alpha=0.85, ax=ax, edgecolor="white", linewidth=0.4,
    )
    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_ylim(bottom=-0.3)
    ax.set_xlabel("Source lines of code")
    ax.set_ylabel("Repair iterations")
    ax.legend(loc="best", fontsize=7)
    fig.tight_layout()
    _save(fig, output_path, "iterations_vs_loc")


def _plot_heatmap(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    if len(order) < 2:
        return
    pivot = _per_unit_outcomes(df).reindex(columns=order)
    if len(pivot) < MIN_UNITS_FOR_HEATMAP:
        return

    height = max(2.0, 0.28 * len(pivot) + 0.8)
    fig, ax = plt.subplots(figsize=(SINGLE_COLUMN_IN, height))
    sns.heatmap(
        pivot, annot=True, fmt=".0%",
        cmap="RdYlGn", vmin=0, vmax=1,
        cbar_kws={"label": "Success rate", "shrink": 0.7},
        linewidths=0.4, linecolor="white", ax=ax,
        annot_kws={"fontsize": 7},
    )
    ax.set_xlabel("")
    ax.set_ylabel("Translation unit")
    fig.tight_layout()
    _save(fig, output_path, "per_unit_heatmap")


# --------------------------------------------------------------------------- #
# Entry point
# --------------------------------------------------------------------------- #

def visualize_results(results_path: Path, output_path: Path) -> None:
    df = pd.read_json(results_path, lines=True)
    if df.empty:
        return

    output_path.mkdir(parents=True, exist_ok=True)
    df = _label_conditions(df)
    order = _ordered_conditions(df)

    _configure_publication_style()
    _write_summary(df, output_path, order)

    # Always-safe plots.
    _plot_success_rate(df, order, output_path)
    _plot_iterations(df, order, output_path)

    # Conditional plots that require enough data to be meaningful.
    _plot_paired_deltas(df, order, output_path)
    _plot_iterations_vs_loc(df, order, output_path)
    _plot_heatmap(df, order, output_path)

    # Diagnostic banner for small-sample runs.
    n_total = len(df)
    if n_total < MIN_RUNS_FOR_DISTRIBUTION or len(order) < 2:
        (output_path / "DIAGNOSTIC_RUN.txt").write_text(
            f"Sample size insufficient for publication figures.\n"
            f"Total runs: {n_total}\n"
            f"Conditions present: {order}\n"
            f"Some plots have been suppressed or simplified.\n"
        )