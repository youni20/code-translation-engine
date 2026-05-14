from math import erfc, sqrt
from pathlib import Path
from typing import cast

import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import numpy as np
import pandas as pd
import seaborn as sns
from matplotlib.figure import Figure

try:
    from scipy import stats as _scipy_stats
except ImportError:
    _scipy_stats = None  # type: ignore[assignment]


CONDITION_LABELS = {
    "A": "A: compiler stderr",
    "B": "B: LSP diagnostics",
}

# Wong (2011) colourblind-safe palette
CONDITION_PALETTE = {
    "A: compiler stderr": "#0072B2",
    "B: LSP diagnostics": "#D55E00",
}

Z_95 = 1.959964


# --------------------------------------------------------------------------- #
# Style + IO
# --------------------------------------------------------------------------- #

def _configure_style() -> None:
    sns.set_theme(style="whitegrid", context="paper", font_scale=1.1)
    plt.rcParams.update({
        "figure.dpi": 150,
        "savefig.dpi": 300,
        "savefig.bbox": "tight",
        "font.family": "serif",
        "axes.spines.top": False,
        "axes.spines.right": False,
    })


def _save(fig: Figure, output_path: Path, name: str) -> None:
    fig.savefig(output_path / f"{name}.png")
    fig.savefig(output_path / f"{name}.pdf")
    plt.close(fig)


# --------------------------------------------------------------------------- #
# Data prep
# --------------------------------------------------------------------------- #

def _label_conditions(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()
    df["condition"] = df["condition"].map(lambda c: CONDITION_LABELS.get(c, c))
    return df


def _ordered_conditions(df: pd.DataFrame) -> list[str]:
    return [v for v in CONDITION_LABELS.values() if v in df["condition"].unique()]


def _wilson_ci(k: int, n: int, z: float = Z_95) -> tuple[float, float]:
    """Wilson score confidence interval for a binomial proportion."""
    if n == 0:
        return 0.0, 0.0
    p = k / n
    denom = 1 + z ** 2 / n
    centre = (p + z ** 2 / (2 * n)) / denom
    half = (z * np.sqrt(p * (1 - p) / n + z ** 2 / (4 * n ** 2))) / denom
    return max(0.0, centre - half), min(1.0, centre + half)


def _per_unit_outcomes(df: pd.DataFrame) -> pd.DataFrame:
    grouped = cast(pd.Series, df.groupby(["unit_id", "condition"])["success"].mean())
    return cast(pd.DataFrame, grouped.unstack("condition"))


def _expand_per_iteration(df: pd.DataFrame) -> pd.DataFrame:
    """For run with iterations_used=k and success=s, emit rows 0..k.

    Iteration k is the final attempt and carries the run's success flag;
    iterations 0..k-1 are by definition failures (they triggered repair).
    """
    rows = []
    for _, r in df.iterrows():
        k = int(r["iterations_used"])
        succeeded = bool(r["success"])
        for i in range(k + 1):
            rows.append({
                "unit_id": r["unit_id"],
                "condition": r["condition"],
                "repetition": int(r["repetition"]),
                "iteration": i,
                "passed": (i == k) and succeeded,
            })
    return pd.DataFrame(rows)


# --------------------------------------------------------------------------- #
# Statistics
# --------------------------------------------------------------------------- #

def _mcnemar(df: pd.DataFrame, order: list[str]) -> dict | None:
    """Paired McNemar test on per-unit majority outcomes."""
    if len(order) < 2:
        return None
    per_unit = _per_unit_outcomes(df).dropna(subset=order)
    if per_unit.empty:
        return None

    a = (per_unit[order[0]] >= 0.5).astype(int)
    b = (per_unit[order[1]] >= 0.5).astype(int)
    b00 = int(((a == 0) & (b == 0)).sum())
    b01 = int(((a == 0) & (b == 1)).sum())
    b10 = int(((a == 1) & (b == 0)).sum())
    b11 = int(((a == 1) & (b == 1)).sum())
    n_disc = b01 + b10

    if n_disc == 0:
        p = 1.0
    elif _scipy_stats is not None:
        p = float(_scipy_stats.binomtest(
            min(b01, b10), n_disc, p=0.5, alternative="two-sided"
        ).pvalue)
    else:
        # Chi-square approximation (df=1), survival function via erfc.
        chi2 = (b01 - b10) ** 2 / n_disc
        p = float(erfc(sqrt(chi2 / 2)))

    return {
        "table": np.array([[b00, b01], [b10, b11]]),
        "n": int(len(per_unit)),
        "p": p,
        "discordant": n_disc,
        "b01_b_wins": b01,
        "b10_a_wins": b10,
    }


# --------------------------------------------------------------------------- #
# CSV / text exports
# --------------------------------------------------------------------------- #

def _export_csv(df: pd.DataFrame, output_path: Path) -> None:
    """Flat, spreadsheet-friendly view (drops bulky text fields)."""
    out = df[[
        "unit_id", "project", "relative_path", "loc",
        "condition", "repetition",
        "success", "iterations_used",
        "wall_time_seconds", "timestamp",
    ]].copy()
    out["n_feedback_rounds"] = df["feedback_history"].apply(len)
    out.to_csv(output_path / "results.csv", index=False)


def _export_per_iteration(df: pd.DataFrame, output_path: Path) -> None:
    _expand_per_iteration(df).to_csv(output_path / "per_iteration.csv", index=False)


def _export_summary(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    rows = []
    for cond in order:
        sub = df[df["condition"] == cond]
        n = len(sub)
        k = int(sub["success"].sum())
        rate = k / n if n else 0.0
        lo, hi = _wilson_ci(k, n)
        rows.append({
            "condition": cond,
            "n_runs": n,
            "n_units": sub["unit_id"].nunique(),
            "success_rate": round(rate, 4),
            "success_rate_wilson_low": round(lo, 4),
            "success_rate_wilson_high": round(hi, 4),
            "mean_iterations": round(float(sub["iterations_used"].mean()), 4),
            "median_iterations": float(sub["iterations_used"].median()),
            "mean_wall_time_s": round(float(sub["wall_time_seconds"].mean()), 4),
            "median_wall_time_s": round(float(sub["wall_time_seconds"].median()), 4),
        })
    pd.DataFrame(rows).to_csv(output_path / "summary.csv", index=False)

    mc = _mcnemar(df, order)
    if mc is not None:
        with (output_path / "mcnemar.txt").open("w") as f:
            f.write("McNemar's test (paired design)\n")
            f.write("=" * 40 + "\n")
            f.write(f"Paired units:                  {mc['n']}\n")
            f.write(f"A pass, B fail (A wins):       {mc['b10_a_wins']}\n")
            f.write(f"A fail, B pass (B wins):       {mc['b01_b_wins']}\n")
            f.write(f"Discordant pairs (b + c):      {mc['discordant']}\n")
            f.write(f"p-value (two-sided):           {mc['p']:.4f}\n\n")
            f.write("Contingency (rows = A, cols = B):\n")
            f.write(pd.DataFrame(
                mc["table"],
                index=["A: fail", "A: pass"],
                columns=["B: fail", "B: pass"],
            ).to_string())


# --------------------------------------------------------------------------- #
# Plots
# --------------------------------------------------------------------------- #

def _plot_success_rate(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Plot 1: final success rate per condition."""
    rates, ns = [], []
    for cond in order:
        sub = df[df["condition"] == cond]
        n = len(sub)
        k = int(sub["success"].sum())
        rates.append(k / n if n else 0.0)
        ns.append(n)

    fig, ax = plt.subplots(figsize=(6, 4))
    xpos = np.arange(len(order))
    colours = [CONDITION_PALETTE[c] for c in order]
    ax.bar(
        xpos, rates, color=colours,
        edgecolor="black", linewidth=0.8, alpha=0.9,
    )
    for i, r in enumerate(rates):
        ax.text(i, r + 0.02, f"{r * 100:.0f}%",
                ha="center", va="bottom", fontweight="bold")

    ax.set_ylim(0, 1.1)
    ax.set_xticks(xpos)
    ax.set_xticklabels([f"{c}\n$n={n}$" for c, n in zip(order, ns)])
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylabel("Compilation success rate")
    ax.set_title("Compilation success by feedback condition")

    fig.tight_layout()
    _save(fig, output_path, "plot1_success_rate")


def _plot_cumulative_success(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Plot 2: cumulative success rate vs. repair iteration."""
    per_iter = _expand_per_iteration(df).sort_values(
        ["condition", "unit_id", "repetition", "iteration"]
    )
    per_iter["cumulative_pass"] = (
        per_iter.groupby(["condition", "unit_id", "repetition"])["passed"]
        .cummax()
        .astype(int)
    )

    # Runs that finish early (success or failure before max_iter) have no rows
    # for later iterations, so seaborn's aggregation at those x-positions only
    # sees still-active runs — making the curve collapse instead of staying flat.
    # Pad every run out to max_iteration carrying its final cumulative_pass value.
    max_iter = int(per_iter["iteration"].max())
    pad_rows = []
    for (cond, uid, rep), grp in per_iter.groupby(
        ["condition", "unit_id", "repetition"], sort=False
    ):
        last_iter = int(grp["iteration"].max())
        carry = int(grp["cumulative_pass"].max())
        for it in range(last_iter + 1, max_iter + 1):
            pad_rows.append({
                "unit_id": uid, "condition": cond, "repetition": rep,
                "iteration": it, "passed": carry, "cumulative_pass": carry,
            })
    if pad_rows:
        per_iter = pd.concat([per_iter, pd.DataFrame(pad_rows)], ignore_index=True)

    fig, ax = plt.subplots(figsize=(7, 4.5))
    sns.lineplot(
        data=per_iter, x="iteration", y="cumulative_pass",
        hue="condition", hue_order=order, palette=CONDITION_PALETTE,
        errorbar=None, marker="o", ax=ax,
    )
    ax.xaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylim(-0.02, 1.05)
    ax.set_xlabel("Repair iteration")
    ax.set_ylabel("Cumulative success rate")
    ax.set_title("Cumulative success rate vs. repair iteration")
    ax.legend(title="")
    fig.tight_layout()
    _save(fig, output_path, "plot2_cumulative_success")


def _plot_iterations(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Plot 4: distribution of iterations-to-first-success (successes only)."""
    successes = df[df["success"]].copy()
    if successes.empty:
        return

    fig, ax = plt.subplots(figsize=(6, 4))
    sns.stripplot(
        data=successes, x="condition", y="iterations_used",
        order=order, hue="condition", hue_order=order,
        palette=CONDITION_PALETTE,
        size=5, alpha=0.6, jitter=0.15, ax=ax, legend=False,
    )
    means = successes.groupby("condition")["iterations_used"].mean()
    for i, cond in enumerate(order):
        if cond in means.index:
            ax.hlines(means[cond], i - 0.3, i + 0.3,
                      color="black", linewidth=2.5, zorder=10)

    ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True))
    ax.set_ylim(bottom=-0.3)
    ax.set_xlabel("")
    ax.set_ylabel("Iterations to first compile success")
    ax.set_title("Iterations to first success (successful runs only)\nBlack bar = mean")
    fig.tight_layout()
    _save(fig, output_path, "plot4_iterations")


def _plot_paired_slope(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Plot 7: per-unit paired slope chart (A success rate -> B success rate)."""
    if len(order) < 2:
        return
    per_unit = _per_unit_outcomes(df).dropna(subset=order)
    if per_unit.empty:
        return

    rng = np.random.default_rng(0)
    jitter = rng.uniform(-0.012, 0.012, size=(len(per_unit), 2))

    fig, ax = plt.subplots(figsize=(5.5, 5))

    n_b_better = n_a_better = n_tied = 0
    for (_uid, row), j in zip(per_unit.iterrows(), jitter):
        a, b = float(row[order[0]]), float(row[order[1]])
        if b > a:
            colour = CONDITION_PALETTE[order[1]]
            n_b_better += 1
        elif a > b:
            colour = CONDITION_PALETTE[order[0]]
            n_a_better += 1
        else:
            colour = "lightgrey"
            n_tied += 1
        ax.plot([0, 1], [a + j[0], b + j[1]],
                color=colour, alpha=0.45, marker="o", markersize=4, linewidth=1)

    mean_a = float(per_unit[order[0]].mean())
    mean_b = float(per_unit[order[1]].mean())
    ax.plot([0, 1], [mean_a, mean_b], color="black",
            marker="D", markersize=8, linewidth=2.5, label="Mean across units")

    ax.set_xticks([0, 1])
    ax.set_xticklabels(order)
    ax.set_xlim(-0.25, 1.25)
    ax.set_ylim(-0.05, 1.08)
    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylabel("Per-unit success rate")
    ax.set_title(
        f"Per-unit outcomes (paired)\n"
        f"B better: {n_b_better}    A better: {n_a_better}    tied: {n_tied}"
    )
    ax.legend(loc="lower right")
    fig.tight_layout()
    _save(fig, output_path, "plot7_paired_slope")


def _plot_per_unit_success(df: pd.DataFrame, order: list[str], output_path: Path) -> None:
    """Plot 8: per-unit empirical success rate distribution across repetitions."""
    rates = (
        df.groupby(["unit_id", "condition"])["success"]
        .mean()
        .reset_index()
        .rename(columns={"success": "per_unit_success"})
    )
    fig, ax = plt.subplots(figsize=(6, 4))
    sns.stripplot(
        data=rates, x="condition", y="per_unit_success",
        order=order, hue="condition", hue_order=order,
        palette=CONDITION_PALETTE,
        size=6, alpha=0.7, jitter=0.15, ax=ax, legend=False,
    )
    means = rates.groupby("condition")["per_unit_success"].mean()
    for i, cond in enumerate(order):
        if cond in means.index:
            ax.hlines(means[cond], i - 0.3, i + 0.3,
                      color="black", linewidth=2.5, zorder=10)

    ax.yaxis.set_major_formatter(mticker.PercentFormatter(xmax=1, decimals=0))
    ax.set_ylim(-0.05, 1.05)
    ax.set_xlabel("")
    ax.set_ylabel("Per-unit success rate (across repetitions)")
    ax.set_title("Per-translation-unit success variability\nBlack bar = mean")
    fig.tight_layout()
    _save(fig, output_path, "plot8_per_unit_success")


# --------------------------------------------------------------------------- #
# Entry point
# --------------------------------------------------------------------------- #

def visualize_results(results_path: Path, output_path: Path) -> None:
    """Read JSONL results, write CSV exports, and emit thesis plots."""
    df = pd.read_json(results_path, lines=True)
    if df.empty:
        print("[visualize] No results to plot.")
        return

    output_path.mkdir(parents=True, exist_ok=True)
    df = _label_conditions(df)
    order = _ordered_conditions(df)

    _configure_style()

    _export_csv(df, output_path)
    _export_per_iteration(df, output_path)
    _export_summary(df, order, output_path)

    _plot_success_rate(df, order, output_path)
    _plot_cumulative_success(df, order, output_path)
    _plot_iterations(df, order, output_path)
    _plot_paired_slope(df, order, output_path)
    _plot_per_unit_success(df, order, output_path)

    print(f"[visualize] Wrote CSVs and plots to {output_path}")
