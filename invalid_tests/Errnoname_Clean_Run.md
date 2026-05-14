# errnoname — Clean Run Analysis
**Run ID:** `2026-05-14_21-17-48`
**Date:** 2026-05-14
**Project:** errnoname (C library mapping `errno` integer values to their symbolic names)
**Model:** gpt-4o-2024-08-06 (translator + repair)
**Setup:** 1 file × 2 conditions × 2 repetitions × max 8 repair iterations

> **Important caveat:** Only `errnoname.h` (9 LOC, a single function declaration) was discovered as a translation unit. The actual implementation file `errnoname.c` (2 864 LOC, the large `switch` returning the name for every `errno` value) was **not** translated in this run. Treat this result as a smoke test of the pipeline on a new dataset rather than a comparison of conditions.

---

## The Two Conditions

| Label | What the repair agent receives after a failed compile |
|-------|------------------------------------------------------|
| **A: compiler stderr** | Raw `rustc` error output |
| **B: LSP diagnostics** | Structured output from rust-analyzer: error codes, line/col numbers, severity |

---

## Files Tested

| File | Lines of Code | Description |
|------|-------------|-------------|
| `errnoname.h` | 9 | Header file — declares `char const * errnoname(int);` and an include guard |

`errnoname.c` (2 864 LOC) is present in `inputs/projects/errnoname/` but was not picked up by the run.

---

## Overall Success Rates

![Success rate chart](../../outputs/runs/2026-05-14_21-17-48/plot1_success_rate.png)

| Condition | Successes / Runs | Success Rate | 95% Wilson CI |
|-----------|-----------------|-------------|--------------|
| A: compiler stderr | 2 / 2 | **100%** | 34% – 100% |
| B: LSP diagnostics | 2 / 2 | **100%** | 34% – 100% |

Both conditions compiled the 9-line header on every attempt. With only 2 runs per condition the Wilson confidence intervals are extremely wide.

---

## Per-File Breakdown

### errnoname.h (9 LOC)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ✅ PASS (1 iter, 8.1s) | ✅ PASS (1 iter, 5.0s) | **100%** |
| **B** | ✅ PASS (1 iter, 30.7s) | ✅ PASS (1 iter, 12.8s) | **100%** |

Every run failed on the initial translation (iter 0) and was fixed by a single repair round (iter 1). The likely first-pass failure mode for a header with only an `extern "C"`-style declaration is that the agent emits a `.rs` file that needs to be wired into a Rust workspace — one repair iteration is enough to land it in a compilable shape.

Wall-time differences are stark even at this scale: B took roughly **3× longer per run on average** (21.8s vs 6.6s) despite identical iteration counts. This is the round-trip cost of starting rust-analyzer and waiting for diagnostics rather than capturing stderr.

---

## Statistical Test (McNemar)

![Paired slope chart](../../outputs/runs/2026-05-14_21-17-48/plot7_paired_slope.png)

**Majority vote per file (≥50% success across reps = unit pass):**

| File | A outcome | B outcome | Winner |
|------|-----------|-----------|--------|
| errnoname.h | PASS (2/2) | PASS (2/2) | Tie |

- **A wins:** 0 units, **B wins:** 0 units
- **Discordant pairs:** 0
- **p-value: 1.000** — uninformative

With a single paired unit and zero disagreement, McNemar has no power at all. The paired-slope plot is a flat line at 100% for the one unit.

---

## Cumulative Success Over Repair Iterations

![Cumulative success](../../outputs/runs/2026-05-14_21-17-48/plot2_cumulative_success.png)

Both curves jump from 0% at iteration 0 to 100% at iteration 1 and stay flat. There is no useful shape to compare because every run resolves in exactly one repair step.

---

## Iterations to Success (Successful Runs Only)

![Iterations to success](../../outputs/runs/2026-05-14_21-17-48/plot4_iterations.png)

| Condition | Successful runs and iterations used |
|-----------|-------------------------------------|
| A | 1, 1 (median = 1) |
| B | 1, 1 (median = 1) |

Identical distributions — no signal here.

---

## Per-Unit Success Variability

![Per unit success](../../outputs/runs/2026-05-14_21-17-48/plot8_per_unit_success.png)

Both conditions are at 100% on the single unit, so the plot collapses to two points at the top. No variability to inspect.

---

## Key Takeaways

1. **This is a smoke test, not a comparison.** With one trivial 9-line unit, every plot in the standard suite degenerates. The pipeline runs end-to-end on the errnoname dataset, but no condition-level claim can be made from this data.

2. **`errnoname.c` should be added.** The large implementation file is where condition A vs. B would actually be exercised — translating a 2 864-line dense `switch` statement is non-trivial and would generate the kind of repair-loop trajectory the experiment is designed to measure. Worth checking the file-discovery configuration before re-running.

3. **B's per-run latency overhead is visible even on trivial inputs.** Average wall time was 6.6s for A and 21.8s for B (≈3× slower). This is consistent with the overhead pattern observed in other runs and is the cost of querying rust-analyzer rather than capturing stderr.

4. **Both conditions resolved in 1 iteration.** A 9-line header is essentially the floor of difficulty; any feedback signal is sufficient at this level. Use a harder unit to see condition divergence.

---

## What This Means for the Thesis

This run does not contribute evidence for or against either condition. It is useful as:

- A confirmation that the pipeline accepts a new dataset without modification.
- A reminder to verify which files in a project are discovered as translation units before running an experiment.
- A data point on the latency overhead of LSP feedback in the simplest possible case.

A re-run that includes `errnoname.c` would turn this into a meaningful third project alongside Swarmz and MicroPather.
