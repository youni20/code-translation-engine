# MicroPather — Clean Run Analysis
**Run ID:** `2026-05-14_19-20-29`  
**Date:** 2026-05-14  
**Project:** MicroPather (C++ pathfinding library)  
**Model:** gpt-4o-2024-08-06 (translator + repair)  
**Setup:** 3 files × 2 conditions × 2 repetitions × max 8 repair iterations

> **Note:** A previous MicroPather run (18:23:26) was invalidated because `outputs/rust_workspace/Cargo.toml` had been accidentally deleted during a project restructure 25 seconds before the run started. Without it, rust-analyzer could not build a project index and returned "No diagnostics" for every iteration of condition B. That file was restored and this is the clean re-run.

---

## The Two Conditions

| Label | What the repair agent receives after a failed compile |
|-------|------------------------------------------------------|
| **A: compiler stderr** | Raw `rustc` error output |
| **B: LSP diagnostics** | Structured JSON from rust-analyzer: error codes, line/col numbers, severity |

---

## Files Tested

| File | Lines of Code | Description |
|------|-------------|-------------|
| `dungeon.cpp` | 305 | Game demo using the pathfinder |
| `micropather.h` | 509 | Header file with class/struct definitions |
| `micropather.cpp` | 1 078 | Core pathfinding algorithm implementation |

---

## Overall Success Rates

![Success rate chart](../outputs/runs/micropatherRUN/plot1_success_rate.png)

| Condition | Successes / Runs | Success Rate | 95% Wilson CI |
|-----------|-----------------|-------------|--------------|
| A: compiler stderr | 3 / 6 | **50%** | 19% – 81% |
| B: LSP diagnostics | 3 / 6 | **50%** | 19% – 81% |

Both conditions performed identically at the aggregate level. The confidence intervals are very wide because there are only 6 runs per condition.

---

## Per-File Breakdown

### dungeon.cpp (305 LOC)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ❌ FAIL (8 iters, 124s) | ❌ FAIL (8 iters, 176s) | **0% — never succeeded** |
| **B** | ❌ FAIL (8 iters, 148s) | ✅ PASS (1 iter, 32s) | **50%** |

B won here. On rep 1, the LSP-guided agent fixed the translation in a single repair step — 32 seconds total. Rep 0 was a complete failure despite 8 attempts. Condition A could not fix this file at all in either attempt.

---

### micropather.cpp (1 078 LOC — largest file)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ❌ FAIL (8 iters, 379s) | ✅ PASS (4 iters, 130s) | **50%** |
| **B** | ✅ PASS (4 iters, 68s) | ❌ FAIL (8 iters, 391s) | **50%** |

Dead tie. Both conditions succeeded exactly once, both using 4 iterations when they did succeed. B's successful run was notably faster (68s vs 130s for A), suggesting LSP diagnostics may have helped find errors more efficiently in that run — but with n=1 success each this could easily be random.

---

### micropather.h (509 LOC — header file)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ✅ PASS (2 iters, 31s) | ✅ PASS (1 iter, 20s) | **100%** |
| **B** | ✅ PASS (4 iters, 93s) | ❌ FAIL (8 iters, 306s) | **50%** |

A won here. The header was easy enough that compiler stderr was sufficient to fix it quickly and consistently. Condition B succeeded once but failed the other rep, and used more iterations even when it succeeded (4 vs 1–2 for A).

---

## Statistical Test (McNemar)

![McNemar heatmap](../outputs/runs/micropatherRUN/plot7_mcnemar_heatmap.png)

McNemar's test checks whether one condition wins significantly more *head-to-head* match-ups than the other, treating each file as a paired unit.

**Majority vote per file (≥50% success across reps = unit pass):**

| File | A outcome | B outcome | Winner |
|------|-----------|-----------|--------|
| dungeon.cpp | FAIL (0/2) | PASS (1/2) | B |
| micropather.cpp | PASS (1/2) | PASS (1/2) | Tie |
| micropather.h | PASS (2/2) | PASS (1/2) | Tie |

- **B wins:** 1 unit, **A wins:** 0 units
- **Discordant pairs:** 1
- **p-value: 1.000** — completely inconclusive

With only 1 discordant pair out of 3 units, the test has no statistical power. This result cannot distinguish between "B is slightly better", "A is slightly better", or "they are equal".

---

## Cumulative Success Over Repair Iterations

![Cumulative success](../outputs/runs/micropatherRUN/plot2_cumulative_success.png)

This shows how quickly each condition accumulates successes as more repair iterations are allowed. The shaded area is a 95% confidence band across all runs.

Key observation: condition B's one fast success (dungeon.cpp rep 1 at iter 1) pulls its curve up early, while most of the other successes come later in the repair loop regardless of condition.

---

## Iterations to Success (Successful Runs Only)

![Iterations to success](../outputs/runs/micropatherRUN/plot4_iterations.png)

Among runs that succeeded:

| Condition | Iterations used |
|-----------|----------------|
| A successes | 4, 2, 1 (median ≈ 2) |
| B successes | 1, 4, 4 (median ≈ 4) |

A succeeded faster on average among successful runs, but B's fastest success (1 iter) was the single quickest fix in the entire experiment.

---

## Per-Unit Success Variability

![Per unit success](../outputs/runs/micropatherRUN/plot8_per_unit_success.png)

This shows the spread of per-unit success rates. With only 2 repetitions per unit, each unit lands at 0%, 50%, or 100% — no finer resolution is possible. The wide spread reflects genuine difficulty variation across files more than it reflects condition differences.

---

## Key Takeaways

1. **LSP is now working.** Unlike the invalidated earlier run where B always returned "No diagnostics", condition B here produces real variable results — failures and successes with different iteration counts.

2. **No clear winner at n=3 units.** Both conditions hit 50% overall. The experiment is underpowered: with 3 files and 2 reps you cannot draw statistical conclusions. McNemar requires multiple discordant pairs to return p < 0.05.

3. **B had one exceptional success** (dungeon.cpp fixed in 1 iteration), suggesting LSP diagnostics can sometimes pinpoint errors very efficiently. But B also failed more often on the header file where A was reliable.

4. **File difficulty dominates the variance.** micropather.cpp (1 078 LOC) was hard for both; micropather.h was easy for both. The condition effect is much smaller than the file-to-file variation.

5. **Compare with Swarmz:** The clean Swarmz run (n=15 per condition) showed A=80%, B=53% — a larger gap in A's favour. MicroPather here shows 50%/50%, which goes slightly in B's direction but is statistically meaningless. Together, neither dataset alone can confirm a reliable difference.

---

## What This Means for the Thesis

This run is **valid and usable** as a second data point alongside the Swarmz results. Honest framing for the thesis:

- Both projects show high variance and the McNemar test is underpowered in both cases (n=3 units each).
- The Swarmz result (A > B) and the MicroPather result (50%/50%, slight B edge) are consistent with a null hypothesis of no difference — or with a small A advantage that requires more units to detect.
- The primary contribution is the pipeline itself; the experiment results are preliminary/exploratory and should be presented as such.
