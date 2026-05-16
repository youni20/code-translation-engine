# Experiment Results — Full Summary
### Automated C++ → Rust Translation: Raw Compiler Errors vs. Combined stderr + LSP Diagnostics

**Projects tested:** 8 &nbsp;|&nbsp; **Total runs:** 332 (166 per condition) &nbsp;|&nbsp; **Translation units:** 59 files &nbsp;|&nbsp; **Model:** gpt-4o-2024-08-06

---

## What We Were Testing

The system translates a C++ file into Rust automatically. If the translated code fails to compile, it enters a **repair loop** — the model reads the error feedback and tries to fix it, up to **8 times**. If it still fails after 8 tries, that run is counted as a failure.

We compared two types of error feedback:

| | **Condition A** | **Condition B** |
|---|---|---|
| Name | Compiler stderr only | stderr + LSP diagnostics |
| What it gives the model | The plain error text `rustc` prints to the terminal | The same `rustc` stderr **plus** structured JSON from `rust-analyzer`: error codes, line/column numbers, severity |
| The idea behind B | *More information and structure should mean better repair guidance* | |

Most files were translated with **4 repetitions** each; some earlier projects used **2 repetitions**.

> **Note:** A previous experiment compared A against LSP-only B (no stderr). Those results are archived in `result_analysis_nostderr/`. This document covers the revised experiment where B receives both signals simultaneously.

---

## The Eight Projects

| Project | What it is | Files | Reps | Runs/condition |
|---------|-----------|-------|------|----------------|
| **hash-library** | MD5, SHA1, SHA256, SHA3, Keccak, CRC32 implementations | 18 | 4 | 72 |
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
| **A: compiler stderr** | 156 | 166 | **94.0%** |
| **B: stderr + LSP diagnostics** | 158 | 166 | **95.2%** |

**B leads A by 1.2 percentage points overall.** The bar chart shows B leading or tying A in 7 of 8 projects. Hash-library is the one project where A leads (99% vs 96%). POOLED: B 95%, A 94%.

The margin is smaller than in previous summaries because hash-library — a near-ceiling project where A edges B — now contributes 72 runs per condition, diluting the B advantage from the earlier projects.

---

## Project-by-Project Breakdown

| Project | A | B | Who wins | Why |
|---------|---|---|----------|-----|
| **hash-library** | **99%** | 96% | **A** | `tests.cpp` fails 3/4 reps under B; all algorithm files perfect for both |
| **PPK_ASSERT** | 88% | 88% | **Tie** | Losses cancel: B better on `ppk_assert.h`, A better on `gtest-all.cc` |
| **TinyRISCV64** | 80% | **90%** | **B** | Core emulator header solved by B both times; A loses two small files |
| **argh** | 80% | **100%** | **B** | B compiles all 10 runs; A fails on `argh.h` and `doctest.h` |
| **debug_assert** | 100% | 100% | **Tie** | Both perfect |
| **immediate2d** | 92% | **96%** | **B (slight)** | Paint program fails once under A; raytracer equally hard for both |
| **poisson-disk-generator** | 100% | 100% | **Tie** | Both perfect |
| **polypartition** | 100% | 100% | **Tie** | Both perfect |

**B leads or ties in 7 of 8 projects.** Hash-library is the first project where A leads in the combined-B experiment.

---

## The Files That Actually Mattered

Of 59 translation units, **47 were perfect ties**. **12 files** showed any difference at all:

![Discriminating files](../summary_plots/discriminating_files.png)

| File | Project | A rate | B rate | Reps | Meaning |
|------|---------|--------|--------|------|---------|
| `example4_paint.cpp` | immediate2d | 50% | **100%** | 2 | A failed one rep; B consistent |
| `doctest.h` | argh | 50% | **100%** | 2 | A failed one rep; B first-try on one rep |
| `argh.h` | argh | 50% | **100%** | 2 | A failed one rep; B consistent |
| `rv64im_stp_runner.cpp` | TinyRISCV64 | 50% | **100%** | 2 | A failed one rep; B succeeded both |
| `stdio_VM_runner.cpp` | TinyRISCV64 | 50% | **100%** | 2 | A failed one rep; B succeeded both |
| `example9_raytracer.cpp` | immediate2d | 50% | 50% | 2 | Both failed one rep — equally hard |
| `github-issue6.cpp` | hash-library | 75% | **100%** | 4 | A failed 1/4 reps (16-LOC file!); B perfect |
| `ppk_assert.h` | PPK_ASSERT | 75% | **100%** | 4 | A failed 1/4 reps; B never failed |
| `ppk_assert.cpp` | PPK_ASSERT | 75% | 75% | 4 | Both failed 1/4 reps — absolute tie |
| `gtest-all.cc` | PPK_ASSERT | **75%** | 50% | 4 | A failed 1/4; B failed 2/4 — A wins |
| `stress.cpp` | TinyRISCV64 | **100%** | 50% | 2 | B failed one rep |
| `tests.cpp` | hash-library | **100%** | 25% | 4 | **First McNemar discordant pair** — A wins |

**Summary: B wins on 7 files (raw rates), A wins on 3, 1 hard tie, 1 absolute tie.** Under the formal ≥50% majority rule: **A wins 1 discordant pair** (`tests.cpp`), B wins 0.

---

## How Quickly Do They Converge?

![Cumulative success](../summary_plots/cumulative_success.png)

Both curves track extremely closely throughout — this is the tightest cumulative comparison in the experiment:

- **Both start near-identical at iteration 0** (~12% A vs ~14% B)
- **Curves overlap through iterations 1–4**, both reaching ~89% by iteration 4
- **B pulls fractionally ahead from iteration 5 onward**, reaching 95.2% vs A's 94.0%
- **B plateaus at 95.2%; A at 94.0%** — 8 B runs and 10 A runs permanently stuck
- The addition of hash-library (near-ceiling for both) narrows the gap from prior summaries

---

## Speed: A Faster on Most Projects

![Wall time](../summary_plots/wall_time_comparison.png)

| Project | A mean time | B mean time | Faster |
|---------|------------|------------|--------|
| PPK_ASSERT | **27s** | 38s | A (+41%) |
| TinyRISCV64 | **128s** | 163s | A (+27%) |
| argh | 43s | **20s** | **B (−53%)** |
| debug_assert | **28s** | 34s | A (+21%) |
| hash-library | **32s** | 33s | **Tied** |
| immediate2d | **48s** | 57s | A (+19%) |
| poisson-disk-generator | 93s | **82s** | **B (−12%)** |
| polypartition | **64s** | 70s | A (+9%) |

A is faster on 5 of 8 projects; B is faster on 2 (argh and poisson-disk-generator); effectively tied on 1 (hash-library, 32s vs 33s). Hash-library is the only project where wall times are near-identical — with 4 reps and 1–2 iterations per run the LSP overhead is spread across many short jobs.

---

## Repair Iterations: B Uses Fewer in 5 of 8 Projects

![Iterations](../summary_plots/iterations_comparison.png)

| Project | A mean iters | B mean iters | Direction |
|---------|-------------|-------------|-----------|
| PPK_ASSERT | 2.5 | 2.5 | **Tied** |
| TinyRISCV64 | 4.6 | 4.7 | **Tied** |
| argh | 2.3 | **0.6** | B much fewer |
| debug_assert | 3.0 | **2.7** | B fewer |
| hash-library | **1.5** | 1.8 | A fewer |
| immediate2d | 1.9 | **1.6** | B fewer |
| poisson-disk-generator | 1.8 | **1.5** | B fewer |
| polypartition | 1.7 | **1.6** | B fewer |

B uses fewer iterations in 5 of 8 projects. Hash-library is the only project in the entire experiment where A uses meaningfully fewer iterations than B (1.5 vs 1.8) — B's high-iteration outliers on `sha256.cpp`, `md5.cpp`, and `tests.cpp` pull its mean up.

---

## The Statistical Test

McNemar's test counts discordant pairs — files where one condition passes (≥50% of reps succeed) while the other fails (<50%). Target: 4 pairs for p < 0.05.

| Project | A wins | B wins | p-value |
|---------|--------|--------|---------|
| hash-library | **1** (`tests.cpp`) | 0 | 1.000 |
| PPK_ASSERT | 0\* | 0\* | 1.000 |
| immediate2d | 0\* | 0\* | 1.000 |
| argh | 0\* | 0\* | 1.000 |
| debug_assert | 0 | 0 | 1.000 |
| poisson-disk-generator | 0 | 0 | 1.000 |
| TinyRISCV64 | 0\* | 0\* | 1.000 |
| polypartition | 0 | 0 | 1.000 |
| **All pooled** | **1** | **0** | **1.000** |

\* Files at 50% or 75% still pass the ≥50% majority rule and do not register as discordant pairs.

**Results remain statistically inconclusive** (p=1.000). However, this is the first time a discordant pair has been registered: `tests.cpp` under B=25% (1/4 fails) clearly falls below the 50% threshold, giving A a formal win. Three more discordant pairs are needed to reach significance.

---

## Key Findings

### First McNemar Discordant Pair: tests/tests.cpp (hash-library)

`tests/tests.cpp` (361 LOC, test harness exercising all hash algorithms): A=4/4=100%, B=1/4=25%.

This is the first file in the combined-B experiment where a condition clearly fails under the formal majority rule. B failed reps 1, 2, and 3 — all with 8-iteration exhaustion. The test harness involves string comparisons, hex encoding, and virtual dispatch through the base `Hash` class; these constructs appear harder for the combined feedback to repair than the pure bitwise algorithm files (which both conditions handle perfectly).

### Emerging Pattern: B Struggles with Test Infrastructure

Across two separate projects now, B underperforms on test framework code:
- **hash-library** `tests.cpp`: B=25% (first discordant pair)
- **PPK_ASSERT** `gtest-all.cc`: B=50% (one failure from a discordant pair)

In both cases the algorithm/library files are perfect ceiling ties. The test infrastructure — string assertions, output formatting, virtual dispatch — appears to be the domain where the combined B feedback does not reliably converge, while raw stderr alone (A) handles it more consistently.

### TinyRISCV64.h: The Defining Reversal Holds

Under LSP-only B this file was A=100%, B=0%. Under combined B it is 100%/100% — both conditions solve it. This reversal continues to be the most consequential result across both experiments.

### example9_raytracer.cpp: Still the Persistent Hard Case

A=50%, B=50% across 2 reps — equally hard for both. Neither feedback signal reliably translates 255 lines of floating-point vector math.

---

## Summary

| | Condition A | Condition B |
|--|-------------|-------------|
| Overall success rate | 94.0% (156/166) | **95.2%** (158/166) |
| Projects where ahead or tied | 5 / 8 (4 ties + hash-lib win) | **7 / 8** |
| Files where condition wins (raw) | 3 | **7** |
| McNemar discordant pairs won | **1** (`tests.cpp`) | 0 |
| B faster in wall time? | — | On 2 of 8 projects (argh, poisson) |
| B uses fewer iterations? | — | On 5 of 8 projects |
| Statistically proven better? | No (p=1.000) | **No** (p=1.000) |

**Bottom line: B leads overall (95.2% vs 94.0%) and wins on more discriminating files, but A has recorded the first formal McNemar win on `tests.cpp`. Neither condition is statistically proven better — 3 more discordant pairs are needed for p < 0.05.**

---

## What Would Make This Conclusive

One discordant pair recorded; three more needed. The two most promising paths:

1. **More 4-rep runs on hard projects.** With 4 reps, a 1/4=25% rate is detectable. `gtest-all.cc` under B is 2/4=50% — one more failure produces a second discordant pair. Running more test-infrastructure-heavy projects with 4 reps would likely accumulate pairs quickly given the emerging B-struggles-with-tests pattern.

2. **More projects in the bitwise/template-heavy domain.** The algorithm files themselves (hash functions, instruction decoders, parsers) generate the largest raw gaps. More projects like TinyRISCV64 or argh, with 4 reps, would push partial failures past the majority threshold.

---

## Regenerating These Plots

```bash
source .venv/bin/activate
python experiment-results/generate_summary.py
```

Output folder: `experiment-results/summary_plots/`
- `project_comparison.png` — success rates per project + pooled
- `wall_time_comparison.png` — mean wall time per project
- `iterations_comparison.png` — mean iterations per project
- `discriminating_files.png` — the 12 files where conditions diverged
- `cumulative_success.png` — how fast each condition converges
- `combined_results.csv` — all 332 individual runs in one table
- `project_summary.csv` — per-project aggregates
- `file_summary.csv` — per-file success rates for both conditions

> The script filters to only include runs where Condition B is `"B: stderr + LSP diagnostics"`. Old LSP-only runs live in `outputs/runs_nostderr/` and are excluded automatically.
