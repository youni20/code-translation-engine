"""
Generate summary plots and CSV files for the nostderr experiment (Condition B = LSP diagnostics only).

This script is the counterpart to generate_summary.py, which covers the current experiment
where Condition B = stderr + LSP diagnostics. Run this separately to regenerate plots for
the archived nostderr runs without touching the current experiment's summary_plots/.

Usage:
    python experiment-results/generate_summary_nostderr.py

Outputs written to experiment-results/summary_plots_nostderr/:
    project_comparison.png      — A vs B success rate per project + pooled
    wall_time_comparison.png    — Mean wall time per project
    iterations_comparison.png   — Mean iterations per project
    discriminating_files.png    — Only files where conditions diverged
    cumulative_success.png      — Pooled cumulative success over repair rounds
    combined_results.csv        — All runs merged into one table
    project_summary.csv         — Per-project aggregates (success rate, time, etc.)
    file_summary.csv            — Per-file success rates for both conditions

Data source: outputs/runs_nostderr/  (runs where B = LSP diagnostics only)
"""

import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# ── paths ────────────────────────────────────────────────────────────────────
ROOT = Path(__file__).parent.parent
RUNS_DIR = ROOT / "outputs" / "runs_nostderr"
OUT_DIR = Path(__file__).parent / "summary_plots_nostderr"
OUT_DIR.mkdir(exist_ok=True)

# Wong colourblind-safe palette
COL_A = "#E69F00"   # amber  — Condition A
COL_B = "#56B4E9"   # sky blue — Condition B
A_COND = "A: compiler stderr"
B_COND = "B: LSP diagnostics"

# ── load & merge all results.csv files ───────────────────────────────────────
dfs = []
for run_dir in sorted(RUNS_DIR.iterdir()):
    csv = run_dir / "results.csv"
    if csv.exists():
        dfs.append(pd.read_csv(csv))

if not dfs:
    sys.exit(f"No results.csv files found under {RUNS_DIR}")

df = pd.concat(dfs, ignore_index=True)
df["success"] = df["success"].astype(bool)

# Keep only recognised condition labels for this experiment
df = df[df["condition"].isin([A_COND, B_COND])].copy()

if df.empty:
    sys.exit("No rows matched the expected condition labels. Check results.csv files.")

# ── export CSVs ──────────────────────────────────────────────────────────────
df.to_csv(OUT_DIR / "combined_results.csv", index=False)
print(f"  combined_results.csv  ({len(df)} rows across {df['project'].nunique()} projects)")

proj_summary = (
    df.groupby(["project", "condition"])
    .agg(
        n_runs=("success", "count"),
        successes=("success", "sum"),
        success_rate=("success", "mean"),
        mean_iterations=("iterations_used", "mean"),
        median_iterations=("iterations_used", "median"),
        mean_wall_time_s=("wall_time_seconds", "mean"),
        median_wall_time_s=("wall_time_seconds", "median"),
    )
    .reset_index()
)
proj_summary["success_pct"] = (proj_summary["success_rate"] * 100).round(1)
proj_summary.to_csv(OUT_DIR / "project_summary.csv", index=False)
print(f"  project_summary.csv")

file_summary = (
    df.groupby(["project", "relative_path", "loc", "condition"])
    .agg(
        n_runs=("success", "count"),
        successes=("success", "sum"),
        success_rate=("success", "mean"),
        mean_iterations=("iterations_used", "mean"),
        mean_wall_time_s=("wall_time_seconds", "mean"),
    )
    .reset_index()
)
file_summary["success_pct"] = (file_summary["success_rate"] * 100).round(1)
file_summary.to_csv(OUT_DIR / "file_summary.csv", index=False)
print(f"  file_summary.csv")

# ── shared helpers ────────────────────────────────────────────────────────────
PROJECTS = sorted(df["project"].unique())


def rates_for(project=None):
    sub = df if project is None else df[df["project"] == project]
    a = sub[sub["condition"] == A_COND]["success"].mean() * 100
    b = sub[sub["condition"] == B_COND]["success"].mean() * 100
    return a, b


def add_labels(ax, bars, values, fmt="{:.0f}%", offset=1.5, fontsize=8):
    for bar, v in zip(bars, values):
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + offset,
            fmt.format(v),
            ha="center", va="bottom", fontsize=fontsize, fontweight="bold",
        )


# ── plot 1: success rate per project + pooled ────────────────────────────────
labels = PROJECTS + ["POOLED"]
a_rates = [rates_for(p)[0] for p in PROJECTS] + [rates_for()[0]]
b_rates = [rates_for(p)[1] for p in PROJECTS] + [rates_for()[1]]

x = np.arange(len(labels))
w = 0.35
fig, ax = plt.subplots(figsize=(11, 5))
ba = ax.bar(x - w / 2, a_rates, w, color=COL_A, label="A: compiler stderr", zorder=3)
bb = ax.bar(x + w / 2, b_rates, w, color=COL_B, label="B: LSP diagnostics", zorder=3)
add_labels(ax, ba, a_rates)
add_labels(ax, bb, b_rates)
ax.axvline(len(PROJECTS) - 0.5, color="grey", linestyle="--", linewidth=0.9)
ax.axvspan(len(PROJECTS) - 0.5, len(labels) - 0.5, color="lightgrey", alpha=0.25, zorder=0)
ax.set_xticks(x)
ax.set_xticklabels(labels, rotation=20, ha="right", fontsize=9)
ax.set_ylabel("Success rate (%)")
ax.set_ylim(0, 120)
ax.set_title("Compilation Success Rate — A vs B (LSP-only)\nCondition B = LSP diagnostics only (no stderr)", fontsize=11, fontweight="bold")
ax.legend(loc="lower right")
ax.grid(axis="y", alpha=0.3, zorder=0)
fig.tight_layout()
fig.savefig(OUT_DIR / "project_comparison.png", dpi=150)
plt.close()
print("  project_comparison.png")

# ── plot 2: mean wall time per project ───────────────────────────────────────
a_times, b_times = [], []
for p in PROJECTS:
    sub = df[df["project"] == p]
    a_times.append(sub[sub["condition"] == A_COND]["wall_time_seconds"].mean())
    b_times.append(sub[sub["condition"] == B_COND]["wall_time_seconds"].mean())

x2 = np.arange(len(PROJECTS))
fig, ax = plt.subplots(figsize=(10, 5))
ba = ax.bar(x2 - w / 2, a_times, w, color=COL_A, label="A: compiler stderr", zorder=3)
bb = ax.bar(x2 + w / 2, b_times, w, color=COL_B, label="B: LSP diagnostics", zorder=3)
add_labels(ax, ba, a_times, fmt="{:.0f}s")
add_labels(ax, bb, b_times, fmt="{:.0f}s")
ax.set_xticks(x2)
ax.set_xticklabels(PROJECTS, rotation=20, ha="right", fontsize=9)
ax.set_ylabel("Mean wall time (seconds)")
ax.set_title("Mean Wall Time per Run — A vs B (LSP-only)\n(includes failed runs)", fontsize=11)
ax.legend()
ax.grid(axis="y", alpha=0.3, zorder=0)
fig.tight_layout()
fig.savefig(OUT_DIR / "wall_time_comparison.png", dpi=150)
plt.close()
print("  wall_time_comparison.png")

# ── plot 3: mean iterations per project ──────────────────────────────────────
a_iters, b_iters = [], []
for p in PROJECTS:
    sub = df[df["project"] == p]
    a_iters.append(sub[sub["condition"] == A_COND]["iterations_used"].mean())
    b_iters.append(sub[sub["condition"] == B_COND]["iterations_used"].mean())

fig, ax = plt.subplots(figsize=(10, 5))
ba = ax.bar(x2 - w / 2, a_iters, w, color=COL_A, label="A: compiler stderr", zorder=3)
bb = ax.bar(x2 + w / 2, b_iters, w, color=COL_B, label="B: LSP diagnostics", zorder=3)
add_labels(ax, ba, a_iters, fmt="{:.1f}", offset=0.05)
add_labels(ax, bb, b_iters, fmt="{:.1f}", offset=0.05)
ax.set_xticks(x2)
ax.set_xticklabels(PROJECTS, rotation=20, ha="right", fontsize=9)
ax.set_ylabel("Mean repair iterations used")
ax.set_title("Mean Repair Iterations per Run — A vs B (LSP-only)\n(failed runs counted at their cap of 8)", fontsize=11)
ax.legend()
ax.grid(axis="y", alpha=0.3, zorder=0)
fig.tight_layout()
fig.savefig(OUT_DIR / "iterations_comparison.png", dpi=150)
plt.close()
print("  iterations_comparison.png")

# ── plot 4: discriminating files (anything below 100%) ───────────────────────
pivot = file_summary.pivot_table(
    index=["project", "relative_path", "loc"],
    columns="condition",
    values="success_pct",
).reset_index()
pivot.columns.name = None
pivot = pivot.rename(columns={A_COND: "A_pct", B_COND: "B_pct"})
disc = pivot[(pivot["A_pct"] < 100) | (pivot["B_pct"] < 100)].copy()
disc = disc.sort_values(["A_pct", "B_pct"], ascending=[False, True])
disc["label"] = (
    disc["relative_path"].apply(lambda p: Path(p).name)
    + "\n("
    + disc["project"]
    + ", "
    + disc["loc"].astype(str)
    + " LOC)"
)

fig, ax = plt.subplots(figsize=(9, max(4, len(disc) * 0.85 + 1.5)))
y = np.arange(len(disc))
ba = ax.barh(y - 0.2, disc["A_pct"], 0.35, color=COL_A, label="A: compiler stderr", zorder=3)
bb = ax.barh(y + 0.2, disc["B_pct"], 0.35, color=COL_B, label="B: LSP diagnostics", zorder=3)
ax.set_yticks(y)
ax.set_yticklabels(disc["label"], fontsize=9)
ax.set_xlabel("Success rate (%)")
ax.set_xlim(0, 120)
ax.set_title(
    "Files Where at Least One Condition Had a Failure (LSP-only B)\n"
    "All other files = 100% for both conditions",
    fontsize=11,
)
ax.axvline(100, color="grey", linestyle="--", linewidth=0.8, alpha=0.5)
ax.legend(loc="lower right")
ax.grid(axis="x", alpha=0.3, zorder=0)
for i, row in enumerate(disc.itertuples()):
    ax.text(row.A_pct + 1, i - 0.2, f"{row.A_pct:.0f}%", va="center", fontsize=8.5, color="#8B6914")
    ax.text(row.B_pct + 1, i + 0.2, f"{row.B_pct:.0f}%", va="center", fontsize=8.5, color="#1A6E8E")
fig.tight_layout()
fig.savefig(OUT_DIR / "discriminating_files.png", dpi=150)
plt.close()
print("  discriminating_files.png")

# ── plot 5: pooled cumulative success ─────────────────────────────────────────
max_iter = int(df["iterations_used"].max())
fig, ax = plt.subplots(figsize=(8, 5))
for cond, col, label in [(A_COND, COL_A, "A: compiler stderr"), (B_COND, COL_B, "B: LSP diagnostics")]:
    sub = df[df["condition"] == cond]
    n = len(sub)
    cum = [sub[(sub["success"]) & (sub["iterations_used"] <= i)].shape[0] / n * 100
           for i in range(max_iter + 1)]
    ax.plot(range(max_iter + 1), cum, color=col, label=label, linewidth=2.5,
            marker="o", markersize=5)
    ax.text(max_iter + 0.1, cum[-1], f"{cum[-1]:.1f}%", va="center", color=col, fontweight="bold", fontsize=9)

ax.set_xlabel("Number of repair iterations")
ax.set_ylabel("% of all runs that have compiled by this point")
ax.set_title(f"Cumulative Success Rate — {df['project'].nunique()} Projects (LSP-only B)", fontsize=12, fontweight="bold")
ax.set_ylim(0, 108)
ax.set_xlim(-0.2, max_iter + 1)
ax.legend()
ax.grid(alpha=0.3)
fig.tight_layout()
fig.savefig(OUT_DIR / "cumulative_success.png", dpi=150)
plt.close()
print("  cumulative_success.png")

# ── print terminal summary ────────────────────────────────────────────────────
print("\n" + "=" * 55)
print("POOLED RESULTS — LSP-ONLY B EXPERIMENT")
print("=" * 55)
for cond in [A_COND, B_COND]:
    sub = df[df["condition"] == cond]
    pct = sub["success"].mean() * 100
    print(f"  {cond}: {sub['success'].sum()}/{len(sub)} = {pct:.1f}%")

print("\nPER-PROJECT BREAKDOWN")
print(f"{'Project':<25} {'A rate':>8} {'B rate':>8} {'A runs':>8} {'B runs':>8}")
print("-" * 57)
for p in PROJECTS:
    sub = df[df["project"] == p]
    a = sub[sub["condition"] == A_COND]
    b = sub[sub["condition"] == B_COND]
    print(f"  {p:<23} {a['success'].mean()*100:>7.1f}% {b['success'].mean()*100:>7.1f}%"
          f"  {a['success'].sum()}/{len(a):>2}     {b['success'].sum()}/{len(b):>2}")

print(f"\nAll plots and CSVs saved to: {OUT_DIR}")
