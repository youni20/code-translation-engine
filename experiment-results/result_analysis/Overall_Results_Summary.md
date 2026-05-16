# Experiment Results — Full Summary
### Automated C++ → Rust Translation: Raw Compiler Errors vs. Combined stderr + LSP Diagnostics

**Projects tested:** 7 &nbsp;|&nbsp; **Total runs:** 188 (94 per condition) &nbsp;|&nbsp; **Translation units:** 41 files &nbsp;|&nbsp; **Model:** gpt-4o-2024-08-06

---

## What We Were Testing

The system translates a C++ file into Rust automatically. If the translated code fails to compile, it enters a **repair loop** — the model reads the error feedback and tries to fix it, up to **8 times**. If it still fails after 8 tries, that run is counted as a failure.

We compared two types of error feedback:

| | **Condition A** | **Condition B** |
|---|---|---|
| Name | Compiler stderr only | stderr + LSP diagnostics |
| What it gives the model | The plain error text `rustc` prints to the terminal | The same `rustc` stderr **plus** structured JSON from `rust-analyzer`: error codes, line/column numbers, severity |
| The idea behind B | *More information and structure should mean better repair guidance* | |

Most files were translated under both conditions with **2 repetitions** each. PPK_ASSERT was run with **4 repetitions** to probe statistical sensitivity.

> **Note:** A previous experiment compared A against LSP-only B (no stderr). Those results are archived in `result_analysis_nostderr/`. This document covers the revised experiment where B receives both signals simultaneously.

---

## The Seven Projects

| Project | What it is | Files tested | Reps | Runs per condition |
|---------|-----------|-------------|------|--------------------|
| **PPK_ASSERT** | C++ assertion library with Google Test | 6 | 4 | 24 |
| **immediate2d** | 2D graphics header library + 12 example programs | 13 | 2 | 26 |
| **argh** | Command-line argument parser (single C++ header) | 5 | 2 | 10 |
| **debug_assert** | Assertion/debugging macro library | 3 | 2 | 6 |
| **poisson-disk-generator** | Poisson-disk point sampling algorithm + demo | 2 | 2 | 4 |
| **TinyRISCV64** | RISC-V 64-bit CPU emulator (instruction decoder + ELF loader) | 5 | 2 | 10 |
| **polypartition** | Polygon partitioning algorithm library | 7 | 2 | 14 |

---

## The Headline Result

![Project comparison](../summary_plots/project_comparison.png)

| Condition | Successes | Total runs | **Success rate** |
|-----------|-----------|-----------|-----------------|
| **A: compiler stderr** | 85 | 94 | **90.4%** |
| **B: stderr + LSP diagnostics** | 89 | 94 | **94.7%** |

**B compiles ~4 percentage points more often than A overall.** The bar chart shows B leading or tying A in 6 of 7 projects — A leads on none. The POOLED bar confirms: B 95%, A 90%.

This is a direction reversal from the previous experiment (LSP-only B), where A led 97.1% to 85.7%. Adding raw stderr to the B condition flipped the result.

---

## Project-by-Project Breakdown

| Project | A | B | Who wins | Why |
|---------|---|---|----------|-----|
| **PPK_ASSERT** | 88% | 88% | **Tie** | Both fail 3/24 runs; losses cancel across different files |
| **TinyRISCV64** | 80% | **90%** | **B** | Core emulator header solved by B both times; A loses two small files |
| **argh** | 80% | **100%** | **B** | B compiles all 10 runs; A fails on argh.h and doctest.h |
| **debug_assert** | 100% | 100% | **Tie** | Both perfect — library too simple to show a difference |
| **immediate2d** | 92% | **96%** | **B (slight)** | Paint program fails once under A; raytracer equally hard for both |
| **poisson-disk-generator** | 100% | 100% | **Tie** | Both perfect |
| **polypartition** | 100% | 100% | **Tie** | Both perfect |

**B is ahead or tied in all 7 projects.** PPK_ASSERT is a dead tie — the two signal files within it (B better on `ppk_assert.h`, A better on `gtest-all.cc`) exactly cancel each other at the project level.

---

## The Files That Actually Mattered

Of 41 translation units, **31 were perfect ties** — both conditions compiled every run. **10 files** showed any difference at all:

![Discriminating files](../summary_plots/discriminating_files.png)

Reading the chart from top to bottom (B-wins first, then ties, then A-wins):

| File | Project | A rate | B rate | Reps | Meaning |
|------|---------|--------|--------|------|---------|
| `example4_paint.cpp` | immediate2d | 50% | **100%** | 2 | A failed one rep; B consistent |
| `doctest.h` | argh | 50% | **100%** | 2 | A failed one rep; B first-try on one rep |
| `argh.h` | argh | 50% | **100%** | 2 | A failed one rep; B consistent |
| `rv64im_stp_runner.cpp` | TinyRISCV64 | 50% | **100%** | 2 | A failed one rep; B succeeded both |
| `stdio_VM_runner.cpp` | TinyRISCV64 | 50% | **100%** | 2 | A failed one rep; B succeeded both |
| `example9_raytracer.cpp` | immediate2d | 50% | 50% | 2 | Both failed one rep — equally hard |
| `ppk_assert.h` | PPK_ASSERT | 75% | **100%** | 4 | A failed 1/4 reps; B never failed |
| `ppk_assert.cpp` | PPK_ASSERT | 75% | 75% | 4 | Both failed 1/4 reps — different reps |
| `gtest-all.cc` | PPK_ASSERT | **75%** | 50% | 4 | A failed 1/4; B failed 2/4 — A wins |
| `stress.cpp` | TinyRISCV64 | **100%** | 50% | 2 | B failed one rep; only A-wins file in 2-rep set |

**Summary: B wins on 6 files, A wins on 2 (`gtest-all.cc`, `stress.cpp`), 1 hard tie (raytracer), 1 absolute tie (ppk_assert.cpp).** The remaining 31 files (76% of the dataset) are ceiling ties — irrelevant to the comparison.

---

## How Quickly Do They Converge?

![Cumulative success](../summary_plots/cumulative_success.png)

- **B starts clearly ahead at iteration 0** (~20% vs ~10%) — more runs compile first-try under B
- **Both climb together through iterations 1–3**, B holding a small lead (~79% vs ~78% at iteration 3)
- **B pulls ahead after iteration 3** and maintains the gap through to the plateau
- **B plateaus at 94.7%; A plateaus at 90.4%** — 5 B runs and 9 A runs are permanently stuck
- The gap is smaller than in the 6-project analysis because PPK_ASSERT (a tie) dilutes the pooled signal

---

## Speed: A Faster on Most Projects

![Wall time](../summary_plots/wall_time_comparison.png)

| Project | A mean time | B mean time | Faster |
|---------|------------|------------|--------|
| PPK_ASSERT | **27s** | 38s | A (+41%) |
| TinyRISCV64 | **128s** | 163s | A (+27%) |
| argh | 43s | **20s** | **B (−53%)** |
| debug_assert | **28s** | 34s | A (+21%) |
| immediate2d | **48s** | 57s | A (+19%) |
| poisson-disk-generator | 93s | **82s** | **B (−12%)** |
| polypartition | **64s** | 70s | A (+9%) |

A is faster on 5 of 7 projects; B is faster on 2 (argh and poisson-disk-generator). The argh advantage for B (43s → 20s) is the most dramatic — the combined feedback resolves template errors in far fewer rounds, more than offsetting the LSP overhead. PPK_ASSERT adds another data point for A being faster when both conditions have similar iteration counts but B pays the LSP query cost.

---

## Repair Iterations: B Uses Fewer in 5 of 7 Projects

![Iterations](../summary_plots/iterations_comparison.png)

| Project | A mean iters | B mean iters | Direction |
|---------|-------------|-------------|-----------|
| PPK_ASSERT | 2.5 | 2.5 | **Tied** |
| TinyRISCV64 | 4.6 | 4.7 | **Tied** |
| argh | 2.3 | **0.6** | B much fewer |
| debug_assert | 3.0 | **2.7** | B slightly fewer |
| immediate2d | 1.9 | **1.6** | B fewer |
| poisson-disk-generator | 1.8 | **1.5** | B fewer |
| polypartition | 1.7 | **1.6** | B slightly fewer |

B uses fewer iterations in 5 of 7 projects and ties on 2. The argh gap (2.3 vs 0.6) remains the most striking. PPK_ASSERT and TinyRISCV64 are the two domains where both conditions struggle equally — the combined feedback offers no iteration advantage on dense macro/hardware code.

---

## The Statistical Test

We used **McNemar's test** to formally compare the two conditions file-by-file. The test counts "discordant pairs" — files where one condition wins under the majority rule (≥50% of reps succeed = pass). To reach statistical significance (p < 0.05), you need at least 4 discordant pairs.

| Project | A wins | B wins | p-value |
|---------|--------|--------|---------|
| PPK_ASSERT | 0* | 0* | 1.000 |
| immediate2d | 0* | 0* | 1.000 |
| argh | 0* | 0* | 1.000 |
| debug_assert | 0 | 0 | 1.000 |
| poisson-disk-generator | 0 | 0 | 1.000 |
| TinyRISCV64 | 0* | 0* | 1.000 |
| polypartition | 0 | 0 | 1.000 |
| **All pooled** | **0** | **0** | **1.000** |

\* Files where at least one condition sits at 50% or 75% still pass the ≥50% majority rule and do not register as discordant pairs.

**All results are statistically inconclusive.** With 2-rep files, only a 0% vs 100% split registers as discordant. With 4-rep files (PPK_ASSERT), a 25% vs ≥50% split would register — but `gtest-all.cc` under B is 50% (2/4), exactly at the boundary. One more B failure on that file would have produced the **first discordant pair** in the entire experiment.

---

## The Most Important Findings

### TinyRISCV64.h — The Defining Reversal

| Condition | Old experiment (LSP-only B) | This experiment (stderr+LSP B) |
|-----------|----------------------------|-------------------------------|
| **A** | ✅ PASS both reps (3, 5 iters) | ✅ PASS both reps (4, 4 iters) |
| **B** | ❌ FAIL both reps (8+8 exhausted) | ✅ PASS both reps (6, 5 iters) |

The file that was the clearest "A wins, B fails" result under LSP-only B now compiles under B on both reps. Adding stderr to the B feedback was sufficient to break through the failure mode that 16 LSP-only repair attempts could not.

### gtest-all.cc — Closest Miss on a Discordant Pair

`gtest-all.cc` (10 410 LOC, Google Test amalgam) under 4 repetitions: A=75% (3/4), B=50% (2/4). Under the ≥50% majority rule, B=50% just barely passes. Had B failed one more rep (1/4 = 25% < 50%), this would have been the first McNemar-significant discordant pair in the experiment. The 4-rep design is working — it is exposing differences that 2-rep runs conceal.

### example9_raytracer.cpp — The Persistent Hard Case

The only file equally hard for both: A=50% (1/2), B=50% (1/2). Neither feedback signal reliably translates 255 lines of floating-point vector math within 8 iterations. This file is the strongest candidate for qualitative inspection of what fundamental C++→Rust barriers neither signal can overcome.

---

## Summary

| | Condition A | Condition B |
|--|-------------|-------------|
| Overall success rate | 90.4% (85/94) | **94.7%** (89/94) |
| Projects where ahead or tied | 4 / 7 (ties only) | **7 / 7** |
| Files where condition wins | 2 (`gtest-all.cc`, `stress.cpp`) | **6 files** |
| Files with 0% success rate | 0 | 0 |
| B faster in wall time? | — | On 2 of 7 projects (argh, poisson) |
| B uses fewer iterations? | — | On 5 of 7 projects |
| Statistically proven better? | No (p=1.000) | **No** (p=1.000) |

**Bottom line: B is better in practice — higher success rate, fewer repair iterations in most projects, and wins on 6 of 10 discriminating files. The experiment cannot yet prove this statistically: every 2-rep failure lands at 50% (which passes the majority rule), and the one 4-rep file where B is weakest (gtest-all.cc, B=50%) sits exactly at the pass/fail boundary.**

---

## What Would Make This Conclusive

The McNemar majority-vote test needs discordant pairs — files where one condition clearly fails (below 50%) while the other clearly passes. Two changes would generate them:

1. **More repetitions (4+) on hard files.** With 4 reps, a 1/4 = 25% rate registers as a "fail." `gtest-all.cc` under B is one failure away from this. Running more projects with 4 reps would expose partial failures that 2-rep designs hide entirely.

2. **More hard files.** 31 of 41 units are ceiling ties. Only files requiring complex C++ idioms with no clean Rust equivalent generate signal: bitwise hardware code (TinyRISCV64), floating-point math (raytracer), template metaprogramming (argh), assertion dispatch (ppk_assert.h). More files in these categories, under 4-rep designs, would accumulate discordant pairs quickly.

---

## Regenerating These Plots

All plots and CSVs can be regenerated from scratch at any time:

```bash
source .venv/bin/activate
python experiment-results/generate_summary.py
```

Output folder: `experiment-results/summary_plots/`
- `project_comparison.png` — success rates per project + pooled
- `wall_time_comparison.png` — mean wall time per project
- `iterations_comparison.png` — mean iterations per project
- `discriminating_files.png` — the 10 files where conditions diverged
- `cumulative_success.png` — how fast each condition converges
- `combined_results.csv` — all 188 individual runs in one table
- `project_summary.csv` — per-project aggregates
- `file_summary.csv` — per-file success rates for both conditions

> The script filters to only include runs where Condition B is `"B: stderr + LSP diagnostics"`. Old LSP-only runs live in `outputs/runs_nostderr/` and are excluded automatically.
