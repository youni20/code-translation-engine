# Experiment Results — Full Summary
### Does the type of error feedback matter when auto-translating C++ to Rust?

**Projects tested:** 6 &nbsp;|&nbsp; **Total runs:** 140 (70 per condition) &nbsp;|&nbsp; **Model:** gpt-4o-2024-08-06

> **Short answer:** Condition A (raw compiler errors) succeeds more often and runs faster. The gap is real but not yet statistically proven — most files are too easy for either condition to fail on.

---

## What We Were Testing

The system takes a C++ source file and translates it to Rust automatically. If the translated code doesn't compile, it enters a **repair loop**: the model reads the error message and tries again, up to **8 times**. If it still hasn't compiled after 8 tries, the run counts as a failure.

We compared two error feedback signals:

| | **Condition A** — compiler stderr | **Condition B** — LSP diagnostics |
|---|---|---|
| What it is | The plain error text `rustc` prints to the terminal | Structured JSON from `rust-analyzer`: error codes, line/column, severity |
| Example | `error[E0308]: mismatched types` + code excerpt | Same error, wrapped in a machine-readable format with precise locations |
| Why we tested B | The hypothesis was: *more structure = better repair guidance* | |

---

## The Six Projects

| Project | Description | Files | Runs each |
|---------|-------------|-------|-----------|
| **immediate2d** | 2D graphics header library + 11 example programs | 13 | 26 |
| **argh** | Command-line argument parser (single header) | 5 | 10 |
| **debug_assert** | Assertion/debugging macro library | 3 | 6 |
| **poisson-disk-generator** | Poisson-disk sampling algorithm + demo | 2 | 4 |
| **TinyRISCV64** | RISC-V 64-bit CPU emulator | 5 | 10 |
| **polypartition** | Polygon partitioning algorithm library | 7 | 14 |

---

## The Headline Numbers

![Project comparison](../summary_plots/project_comparison.png)

| Condition | Successes | Total runs | **Success rate** |
|-----------|-----------|-----------|-----------------|
| **A: compiler stderr** | 68 | 70 | **97.1%** |
| **B: LSP diagnostics** | 60 | 70 | **85.7%** |

**A leads by ~11 percentage points.** The shaded "POOLED" bar on the right shows the combined picture across all six projects.

---

## Project-by-Project

| Project | A success | B success | Direction | What drove it |
|---------|-----------|-----------|-----------|---------------|
| immediate2d | 100% (26/26) | 85% (22/26) | **A > B** | 3 files (raytracer, smoke, paint) failed under B |
| argh | 90% (9/10) | 100% (10/10) | **B > A** | `argh.h` failed once under A |
| debug_assert | 100% (6/6) | 100% (6/6) | **Tie** | Both perfect; B was just slower |
| poisson-disk-generator | 100% (4/4) | 100% (4/4) | **Tie** | Both perfect |
| TinyRISCV64 | 90% (9/10) | 60% (6/10) | **A >> B** | Core emulator header failed under B both reps |
| polypartition | 100% (14/14) | 86% (12/14) | **A > B** | 2 test files failed under B |

The pattern is **A ≥ B in 5 of 6 projects.** The one exception is `argh`, where B solved a string-parsing argument library that A failed on once.

---

## Files That Actually Generated Any Difference

Of the **35 translation units** tested across all projects, **26 were perfect ceiling-ties** — both conditions compiled every single run, every time. Only **9 files** showed any difference:

![Discriminating files](../summary_plots/discriminating_files.png)

| File | Project | LOC | A rate | B rate | Result |
|------|---------|-----|--------|--------|--------|
| `example9_raytracer.cpp` | immediate2d | 255 | 100% | **0%** | A wins |
| `TinyRISCV64.h` | TinyRISCV64 | 625 | 100% | **0%** | A wins |
| `test/image.cpp` | polypartition | 390 | 100% | 50% | A partial win |
| `test/test.cpp` | polypartition | 415 | 100% | 50% | A partial win |
| `example8_smoke.cpp` | immediate2d | 306 | 100% | 50% | A partial win |
| `example4_paint.cpp` | immediate2d | 117 | 100% | 50% | A partial win |
| `stdio_VM_runner.cpp` | TinyRISCV64 | 64 | 100% | 50% | A partial win |
| `TinyElfRISCV64.h` | TinyRISCV64 | 658 | 50% | 50% | Tie (both hard) |
| `argh.h` | argh | 485 | 50% | 100% | **B wins** |

**A wins or partially wins on 7 of the 9 files with any disagreement. B wins on 1.**

The 2 files with a complete A-win / B-fail split (`raytracer.cpp` and `TinyRISCV64.h`) are the most informative: B exhausted all 8 repair iterations on both repetitions without ever compiling. A solved both in 3–8 iterations consistently.

---

## Speed Comparison

Even when both conditions succeed, they don't perform equally on time:

![Wall time](../summary_plots/wall_time_comparison.png)

![Iterations](../summary_plots/iterations_comparison.png)

| Project | A mean wall time | B mean wall time | A mean iters | B mean iters |
|---------|-----------------|-----------------|-------------|-------------|
| immediate2d | 45s | 83s | 1.6 | 2.8 |
| argh | 49s | 66s | 1.9 | 1.2 |
| debug_assert | 31s | 51s | 2.3 | 2.7 |
| poisson-disk-generator | 52s | 98s | 1.0 | 2.0 |
| TinyRISCV64 | 155s | 230s | 3.7 | 6.2 |
| polypartition | 52s | 91s | 1.4 | 2.2 |

**B is slower in every project except argh.** Two reasons:
1. Every repair iteration under B involves an extra `rust-analyzer` query, which adds ~25–30 seconds per round
2. B's repair trajectories tend to use more iterations even when they succeed

The most extreme example: `Poisson.cpp` rep 0 — A compiled first try (0 iterations, 15 seconds); B needed 5 iterations and 241 seconds for the same final outcome.

---

## How Quickly Do They Converge?

![Cumulative success](../summary_plots/cumulative_success.png)

This chart shows: out of all 70 runs per condition, what percentage have compiled by each repair iteration. A few things to notice:

- **A reaches ~93% by iteration 3;** B is still around 78% at that point
- **A plateaus at 97.1%;** B plateaus at 85.7% — four B failures (TinyRISCV64.h ×2 and two polypartition files) never resolve even at iteration 8
- The curves diverge from iteration 0 and never converge — B doesn't catch up to A even at the maximum number of attempts

---

## Statistical Test — Was the Difference Real?

We used **McNemar's test**, which compares conditions file-by-file. A "discordant pair" is a file where A succeeds and B fails (or vice versa) under the ≥50% majority rule.

| Project | Discordant pairs | A wins | B wins | p-value |
|---------|-----------------|--------|--------|---------|
| immediate2d | 1 | 1 | 0 | 1.000 |
| argh | 0 | 0 | 0 | 1.000 |
| debug_assert | 0 | 0 | 0 | 1.000 |
| poisson-disk-generator | 0 | 0 | 0 | 1.000 |
| TinyRISCV64 | 1 | 1 | 0 | 1.000 |
| polypartition | 0 | 0 | 0 | 1.000 |
| **All pooled** | **2** | **2** | **0** | **0.500** |

**All p-values are ≥ 0.05 — statistically inconclusive.** The test needs at least 4 discordant pairs to reach p < 0.05. We have 2.

Why so few discordant pairs despite the clear success rate gap? Two reasons:
- **The majority vote rule** counts a file as "pass" even if B failed once, as long as B succeeded on the other repetition (1/2 = 50% = pass). This makes partial failures invisible to the test.
- **Most files are too easy** — 26 of 35 units are perfect ties, contributing nothing to the count.

> The trend is clearly in A's favour, but we cannot yet claim statistical significance. The experiment needs more *hard* files and more repetitions per file.

---

## Two Opposing Failure Modes

The data reveals two opposite situations — one where A consistently beats B, and one where B beats A:

### When A wins: Low-level systems code
**Example: `TinyRISCV64.h`** (RISC-V instruction decoder, 625 LOC)
- A: solved both repetitions (3 and 5 iterations)
- B: failed both repetitions (8 iterations each, never compiled)
- The file contains dense bitwise operations, integer-width casts, and large switch trees — exactly where C++→Rust idiom translation is hardest. The plain `rustc` message ("expected u32, found i64") is direct and actionable. The LSP wrapping appears to add noise.

### When B wins: String/parser code
**Example: `argh.h`** (argument parser, 485 LOC)
- A: failed once (hit 8-iteration limit on rep 0)
- B: solved both repetitions (2 iterations each)
- The file is heavy on template metaprogramming and string iterator patterns — where structured error codes and precise line/column locations may help the model pinpoint what to fix.

These two files together suggest the answer may not be "A is always better" but rather **"it depends on the type of code"**.

---

## What the Data Says

| Question | Answer |
|----------|--------|
| Which condition has a higher success rate overall? | **A** (97.1% vs 85.7%) |
| Is the difference statistically significant? | **No** (McNemar p = 0.500 pooled) |
| Which condition is faster? | **A** (roughly 40% less wall time across all projects) |
| Is there a project where B is better? | **Yes** — argh (B = 100%, A = 90%) |
| Does file size predict which condition is better? | **No** — 6,580-line `doctest.h` compiled fine under both; 625-line `TinyRISCV64.h` broke B completely |
| What predicts difficulty? | **Code type** — bitwise/systems code is hard for B; most library APIs are easy for both |

---

## If You Had to Pick One Condition Today

**Use Condition A (raw compiler stderr).** It has a higher success rate, uses fewer iterations, and is faster — without any cases where it catastrophically underperforms relative to B. B adds overhead (every repair round takes longer) and introduces failure modes that A doesn't have, without delivering the improvement in accuracy that motivated it.

---

## What Would Make the Results More Definitive

1. **More hard files** — algorithmic code (ray tracing, physics, instruction decoding, complex parsers) rather than examples, test wrappers, or data files. 26 of 35 current units are too easy to distinguish the conditions.

2. **More repetitions per file** — currently 2 repetitions. The statistical test counts a file as "passed" even if B succeeded only once. With 3+ repetitions the test becomes sensitive to single-rep failures.

3. **Closer inspection of the 2 complete-divergence files** — reading the repair transcripts for `raytracer.cpp` and `TinyRISCV64.h` would show concretely what repair paths B takes that A avoids, and whether the LSP structure is causing the model to chase unproductive error chains.

---

## Generated Files

All plots and raw data can be regenerated at any time:

```bash
source .venv/bin/activate
python experiment-results/generate_summary.py
```

Outputs in `experiment-results/summary_plots/`:
- `project_comparison.png` — success rates per project
- `wall_time_comparison.png` — mean wall time per project
- `iterations_comparison.png` — mean iterations per project
- `discriminating_files.png` — only the files where conditions diverged
- `cumulative_success.png` — how fast each condition converges across all runs
- `combined_results.csv` — all 140 runs in one table
- `project_summary.csv` — per-project aggregates
- `file_summary.csv` — per-file success rates for both conditions
