# Experiment Results — Full Summary
### Automated C++ → Rust Translation: Raw Compiler Errors vs. Combined stderr + LSP Diagnostics

**Projects tested:** 6 &nbsp;|&nbsp; **Total runs:** 140 (70 per condition) &nbsp;|&nbsp; **Translation units:** 35 files &nbsp;|&nbsp; **Model:** gpt-4o-2024-08-06

---

## What We Were Testing

The system translates a C++ file into Rust automatically. If the translated code fails to compile, it enters a **repair loop** — the model reads the error feedback and tries to fix it, up to **8 times**. If it still fails after 8 tries, that run is counted as a failure.

We compared two types of error feedback:

| | **Condition A** | **Condition B** |
|---|---|---|
| Name | Compiler stderr only | stderr + LSP diagnostics |
| What it gives the model | The plain error text `rustc` prints to the terminal | The same `rustc` stderr **plus** structured JSON from `rust-analyzer`: error codes, line/column numbers, severity |
| The idea behind B | *More information and structure should mean better repair guidance* | |

Each file was translated under both conditions, repeated twice, giving 2 reps × 2 conditions per file.

> **Note:** A previous experiment compared A against LSP-only B (no stderr). Those results are archived in `result_analysis_nostderr/`. This document covers the revised experiment where B receives both signals simultaneously.

---

## The Six Projects

| Project | What it is | Files tested | Runs per condition |
|---------|-----------|-------------|-------------------|
| **immediate2d** | 2D graphics header library + 12 example programs | 13 | 26 |
| **argh** | Command-line argument parser (single C++ header) | 5 | 10 |
| **debug_assert** | Assertion/debugging macro library | 3 | 6 |
| **poisson-disk-generator** | Poisson-disk point sampling algorithm + demo | 2 | 4 |
| **TinyRISCV64** | RISC-V 64-bit CPU emulator (instruction decoder + ELF loader) | 5 | 10 |
| **polypartition** | Polygon partitioning algorithm library | 7 | 14 |

---

## The Headline Result

![Project comparison](../summary_plots/project_comparison.png)

| Condition | Successes | Total runs | **Success rate** |
|-----------|-----------|-----------|-----------------|
| **A: compiler stderr** | 64 | 70 | **91.4%** |
| **B: stderr + LSP diagnostics** | 68 | 70 | **97.1%** |

**B compiles ~6 percentage points more often than A overall.** Looking at the bar chart, B beats or ties A in every single project — A does not lead on any project. The shaded POOLED bar on the right shows the overall picture: B 97%, A 91%.

This is a reversal from the previous experiment (LSP-only B), where A led 97.1% to 85.7%. Adding raw stderr to the B condition flipped the direction.

---

## Project-by-Project Breakdown

| Project | A | B | Who wins | Why |
|---------|---|---|----------|-----|
| **TinyRISCV64** | 80% | **90%** | **B** | Core emulator header solved by B both times; A loses two small files |
| **argh** | 80% | **100%** | **B** | B compiles all 10 runs; A fails once on argh.h and once on doctest.h |
| **debug_assert** | 100% | 100% | **Tie** | Both perfect — library too simple to show a difference |
| **immediate2d** | 92% | **96%** | **B (slight)** | Paint program fails once under A; raytracer equally hard for both |
| **poisson-disk-generator** | 100% | 100% | **Tie** | Both perfect |
| **polypartition** | 100% | 100% | **Tie** | Both perfect — old B failures eliminated by combined feedback |

**B is ahead or tied in all 6 projects.** The gaps range from 4 pp (immediate2d) to 20 pp (TinyRISCV64 and argh).

---

## The Files That Actually Mattered

Of 35 translation units across all projects, **28 were perfect ties** — both conditions compiled every single run. Only **7 files** showed any difference at all:

![Discriminating files](../summary_plots/discriminating_files.png)

Reading the chart from top to bottom (B-wins first, then ties, then A-wins):

| File | Project | A rate | B rate | Meaning |
|------|---------|--------|--------|---------|
| `example4_paint.cpp` | immediate2d | 50% | **100%** | A failed one rep; B consistent both times |
| `doctest.h` | argh | 50% | **100%** | A failed one rep on 6580-LOC testing framework; B compiled first-try on one rep |
| `argh.h` | argh | 50% | **100%** | A failed one rep; B consistent both times |
| `rv64im_stp_runner.cpp` | TinyRISCV64 | 50% | **100%** | A failed one rep; B succeeded both |
| `stdio_VM_runner.cpp` | TinyRISCV64 | 50% | **100%** | A failed one rep; B succeeded both |
| `example9_raytracer.cpp` | immediate2d | 50% | 50% | Both conditions failed one rep — equally hard |
| `stress.cpp` | TinyRISCV64 | **100%** | 50% | Only file where A wins; B failed one rep |

**Summary: B wins on 5 files, A wins on 1, both struggle equally on 1.** The remaining 28 files (80% of the dataset) are irrelevant to the comparison — both conditions succeed every time.

Compare to the previous experiment (LSP-only B) where A won on 7 files and B won on 1. The combined feedback completely reversed which condition dominates the discriminating files.

---

## How Quickly Do They Converge?

![Cumulative success](../summary_plots/cumulative_success.png)

This chart shows the percentage of all 70 runs that have successfully compiled by each repair iteration number:

- **B starts ahead at iteration 0** (~20% vs ~11%) — more runs compile first-try under B
- **B leads throughout** — at iteration 1: B=56%, A=50%; at iteration 2: B=73%, A=67%
- **B reaches 90% at iteration 3;** A doesn't reach 90% until iteration 6
- **B plateaus at 97.1%; A plateaus at 91.4%** — 2 B runs are permanently stuck, 6 A runs are permanently stuck
- The curves never converge. The B advantage is established early and maintained throughout

---

## Speed: A Mixed Picture

![Wall time](../summary_plots/wall_time_comparison.png)

Unlike the previous experiment where B was always slower, the wall-time comparison is now mixed:

| Project | A mean time | B mean time | Faster |
|---------|------------|------------|--------|
| TinyRISCV64 | 128s | 163s | A (+27%) |
| argh | 43s | **20s** | **B (−53%)** |
| debug_assert | 28s | 34s | A (+21%) |
| immediate2d | 48s | 57s | A (+19%) |
| poisson-disk-generator | 93s | **82s** | **B (−12%)** |
| polypartition | 64s | 70s | A (+9%) |

A is faster on 4 projects; B is faster on 2. The dramatic B advantage on `argh` (43s → 20s) comes from B needing far fewer repair iterations there (mean 0.6 vs A's 2.3) — the combined feedback helps the model fix argh's template errors in fewer rounds, more than offsetting the LSP query overhead. On TinyRISCV64, both conditions need many rounds on hard low-level code, and B's LSP overhead compounds to a meaningful time difference.

---

## Repair Iterations: B Uses Fewer in 5 of 6 Projects

![Iterations](../summary_plots/iterations_comparison.png)

| Project | A mean iters | B mean iters | B advantage |
|---------|-------------|-------------|-------------|
| TinyRISCV64 | 4.6 | 4.7 | None (tied) |
| argh | 2.3 | **0.6** | **B much fewer** |
| debug_assert | 3.0 | 2.7 | B slightly fewer |
| immediate2d | 1.9 | **1.6** | B fewer |
| poisson-disk-generator | 1.8 | **1.5** | B fewer |
| polypartition | 1.7 | **1.6** | B slightly fewer |

B uses fewer iterations in 5 of 6 projects. The `argh` gap (2.3 vs 0.6) is the most striking: B resolves argh files in under 1 repair round on average, while A needs 2.3. The TinyRISCV64 tie (4.6 vs 4.7) reflects that both conditions are equally strained by the difficulty of the hardware emulation domain.

---

## The Statistical Test

We used **McNemar's test** to formally compare the two conditions file-by-file. The test counts "discordant pairs" — files where one condition wins under the majority rule (≥50% of reps succeed). To reach statistical significance (p < 0.05), you need at least 4 discordant pairs.

| Project | A wins | B wins | p-value |
|---------|--------|--------|---------|
| immediate2d | 0 | 0* | 1.000 |
| argh | 0 | 0* | 1.000 |
| debug_assert | 0 | 0 | 1.000 |
| poisson-disk-generator | 0 | 0 | 1.000 |
| TinyRISCV64 | 0* | 0* | 1.000 |
| polypartition | 0 | 0 | 1.000 |
| **All pooled** | **0** | **0** | **1.000** |

\* Files where one condition is at 50% (one rep pass, one fail) still count as "pass" under the ≥50% majority rule, so they do not register as discordant pairs despite one condition being clearly better.

**All results are statistically inconclusive.** Every single A failure in this experiment is a 50%-success file (1 pass, 1 fail), and every B failure is also a 50%-success file. The majority rule converts all of them to "pass" — leaving 0 discordant pairs and p = 1.0 even when pooling all 35 files.

Why: with n=2 repetitions per file, the only way to register a discordant pair is a 100% vs 0% split on a file. No file in this run has that profile. The B advantage is real in the raw data but invisible to McNemar under this design.

---

## The Most Important Finding: TinyRISCV64.h

The single most consequential result in this experiment is the behaviour of `TinyRISCV64.h` — the core RISC-V instruction decoder, 625 lines of dense bitwise operations, union-typed registers, and large switch trees.

| Condition | Old experiment (LSP-only B) | New experiment (stderr+LSP B) |
|-----------|----------------------------|-------------------------------|
| **A** | ✅ PASS both reps (3 and 5 iters) | ✅ PASS both reps (4 and 4 iters) |
| **B** | ❌ FAIL both reps (8+8 iters exhausted) | ✅ PASS both reps (6 and 5 iters) |

Under LSP-only B, this file was the clearest evidence that raw stderr outperforms structured diagnostics for low-level systems code: A solved it every time, B never solved it. Under the combined condition, B solves it on both reps (albeit in more iterations and much more time than A). Having the raw `rustc` error text alongside the structured location data appears to give the model enough direct signal to navigate the dense type-casting and bitwise repair paths that LSP alone could not resolve.

---

## The Continued Hard Case: example9_raytracer.cpp

The one file that remains equally hard for both conditions is `example9_raytracer.cpp` (255 LOC, floating-point vector math and struct decomposition):

- **A**: failed one rep, succeeded one rep (7 iterations)
- **B**: failed one rep, succeeded one rep (4 iterations)

Neither feedback signal reliably translates this file within 8 iterations. The raytracer appears to hit a fundamental C++→Rust idiom barrier — complex floating-point arithmetic and nested struct decomposition — that neither feedback approach can consistently solve. It is the strongest candidate for qualitative inspection of what specific translation barriers neither signal overcomes.

---

## Summary

| | Condition A | Condition B |
|--|-------------|-------------|
| Overall success rate | 91.4% | **97.1%** |
| Projects where ahead or tied | 3 / 6 (ties only) | **6 / 6** |
| Files where condition wins | 1 (`stress.cpp`) | **5 files** |
| Files with 0% success rate | 0 | 0 |
| B faster in wall time? | — | On 2 of 6 projects (argh, poisson) |
| B uses fewer iterations? | — | On 5 of 6 projects |
| Statistically proven better? | No (p=1.000) | **No** (p=1.000) |

**Bottom line: B is better in practice — higher success rate, fewer repair iterations in most projects, and wins on 5 of 7 discriminating files. But the experiment cannot prove this statistically because all failures are 50%-success files (1 pass, 1 fail), which the majority-vote McNemar design counts as ties.**

---

## What Would Make This Conclusive

The core problem remains the same: with n=2 repetitions per file, the McNemar majority-vote design cannot detect condition differences unless one condition fails both reps and the other passes both. Every failure in this dataset is a single-rep failure — which registers as a 50% "pass."

Two changes would unlock the test:
1. **More repetitions per file** (3+ instead of 2) — this would let partial failures accumulate into real discordant pairs
2. **More hard files** — files like `TinyRISCV64.h` and `example9_raytracer.cpp` that push both conditions near the failure boundary. Of the 35 files tested, 28 are ceiling-level ties. Only the 7 discriminating files contribute any information.

The discriminating files share a common trait: **dense C++ idioms with no clean Rust equivalent** — floating-point vector math (raytracer), bitwise hardware operations (TinyRISCV64), template metaprogramming (argh), event-driven painting logic (example4_paint). More code from these categories, tested with 3 repetitions, would likely produce enough discordant pairs for McNemar to reach significance.

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
- `discriminating_files.png` — only the 7 files where conditions diverged
- `cumulative_success.png` — how fast each condition converges
- `combined_results.csv` — all 140 individual runs in one table
- `project_summary.csv` — per-project aggregates
- `file_summary.csv` — per-file success rates for both conditions

> The script filters to only include runs where Condition B is `"B: stderr + LSP diagnostics"`. Old LSP-only runs live in `outputs/runs_nostderr/` and are excluded automatically.
