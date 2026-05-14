# Swarmz — Clean Run Analysis
**Run ID:** `2026-05-14_20-03-54`  
**Date:** 2026-05-14  
**Project:** Swarmz (header-only C++ boid/flocking library)  
**Model:** gpt-4o-2024-08-06 (translator + repair)  
**Setup:** 2 files × 2 conditions × 2 repetitions × max 8 repair iterations

> **Note on setup:** The project was flattened from its original subfolder structure (`debugging/main.cpp`, `debugging/timing.h`, `swarmz.h`) into a single directory. `timing.h` was dropped because it is only referenced by dead code in `main.cpp` (there is a literal `exit(0)` before the timing loop runs). `main.cpp` was cleaned to remove that dead code block and the `#include "timing.h"` line.

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
| `main.cpp` | 28 | Demo driver — creates 4 boids, runs one acceleration update, prints results |
| `swarmz.h` | 355 | Full library — `Vec3` math, custom hash map, voxel spatial cache, boid flocking algorithm |

---

## Overall Success Rates

![Success rate chart](../outputs/runs/swarmzRUN/plot1_success_rate.png)

| Condition | Successes / Runs | Success Rate | 95% Wilson CI |
|-----------|-----------------|-------------|--------------|
| A: compiler stderr | 4 / 4 | **100%** | 51% – 100% |
| B: LSP diagnostics | 2 / 4 | **50%** | 15% – 85% |

Condition A succeeded on every single run. Condition B succeeded on `main.cpp` both times but failed on `swarmz.h` both times. The confidence intervals are wide because there are only 4 runs per condition.

---

## Per-File Breakdown

### main.cpp (28 LOC)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ✅ PASS (0 iters, 12s) | ✅ PASS (0 iters, 11s) | **100% — compiled first try both times** |
| **B** | ✅ PASS (1 iter, 21s) | ✅ PASS (1 iter, 21s) | **100%** |

`main.cpp` is simple enough that condition A compiled without needing any repair at all (0 iterations = first translation attempt already compiles). Condition B needed exactly 1 LSP-guided repair in both reps, taking ~21s vs ~11s for A. The single repair step was enough. Both conditions succeed 100% here.

---

### swarmz.h (355 LOC — the full algorithm)

| | Rep 0 | Rep 1 | Unit result |
|-|-------|-------|------------|
| **A** | ✅ PASS (8 iters, 244s) | ✅ PASS (7 iters, 223s) | **100% — barely made it** |
| **B** | ❌ FAIL (8 iters, 269s) | ❌ FAIL (8 iters, 276s) | **0% — never succeeded** |

This is where the conditions diverge sharply. `swarmz.h` is genuinely difficult to translate — it uses C++ features that are hard to map to Rust: namespace-scoped operator overloads, a custom `std::unordered_map` hasher, raw pointers into a `vector`, and `std::mt19937` passed by reference. Condition A scraped through both times, needing the maximum or near-maximum repair iterations. Condition B exhausted all 8 iterations in both reps and never compiled.

---

## Statistical Test (McNemar)

![McNemar heatmap](../outputs/runs/swarmzRUN/plot7_mcnemar_heatmap.png)

McNemar's test checks which condition wins more head-to-head match-ups across files.

**Majority vote per file (≥50% success across reps = unit pass):**

| File | A outcome | B outcome | Winner |
|------|-----------|-----------|--------|
| main.cpp | PASS (2/2) | PASS (2/2) | Tie |
| swarmz.h | PASS (2/2) | FAIL (0/2) | **A** |

- **A wins:** 1 unit, **B wins:** 0 units
- **Discordant pairs:** 1
- **p-value: 1.000** — inconclusive

With only 2 units total and 1 discordant pair, the test has no statistical power. A wins the one file that matters (the actual algorithm), but this cannot be called statistically significant.

---

## Cumulative Success Over Repair Iterations

![Cumulative success](../outputs/runs/swarmzRUN/plot2_cumulative_success.png)

The divergence here is stark. Condition A's line climbs steadily and eventually reaches 100% — the `swarmz.h` runs kept improving with each repair iteration until they finally compiled. Condition B plateaus early because `main.cpp` was fixed in 1 iteration, but `swarmz.h` made no progress across all 8 iterations in either rep.

---

## Iterations to Success (Successful Runs Only)

![Iterations to success](../outputs/runs/swarmzRUN/plot4_iterations.png)

| Condition | Successful runs and iterations used |
|-----------|-------------------------------------|
| A | 0, 0, 8, 7 (median ≈ 3.5) |
| B | 1, 1 (median = 1) |

This plot only shows runs that succeeded. Because B failed on all `swarmz.h` runs, only its `main.cpp` successes appear here (1 iter each). A's two `main.cpp` wins (0 iters) and two `swarmz.h` wins (7–8 iters) give it a much wider spread. The median for B looks better here but that is misleading — B simply has no hard-file successes to include.

---

## Per-Unit Success Variability

![Per unit success](../outputs/runs/swarmzRUN/plot8_per_unit_success.png)

This shows each file's success rate across its 2 repetitions:
- `main.cpp`: A=100%, B=100% — trivial for both
- `swarmz.h`: A=100%, B=0% — complete split

The entire story of this experiment is the `swarmz.h` result. A's compiler stderr feedback was enough to guide the agent through 7–8 repair iterations to a successful translation. B's structured LSP diagnostics were not enough — despite receiving precise error codes and character-level positions, the repair agent could not resolve the remaining issues within 8 attempts.

---

## Key Takeaways

1. **A clean win for condition A on the hard file.** Condition A compiled `swarmz.h` both times (barely, at 7–8 iterations). Condition B failed both times completely. This is the clearest single-file result in the entire experiment.

2. **main.cpp is too easy to tell conditions apart.** At 28 LOC, it compiles first-try under A and in 1 repair under B. It contributes nothing to understanding which feedback type is better for difficult translations.

3. **swarmz.h is genuinely hard.** The C++→Rust translation challenge here involves custom hash maps, raw pointer aliasing, namespace enums, and operator overloads — all of which require non-trivial Rust idiom choices. It took A near the iteration limit just to succeed.

4. **LSP diagnostics were not more helpful on the hard file.** The structured format (error code, line, column) did not help the agent converge more than raw compiler output. If anything, the additional context may have fragmented the repair strategy.

5. **n=2 units is still the fundamental limitation.** With only 2 files, McNemar cannot produce a significant result. These results are consistent with the A-favours-direction seen in the original Swarmz run (A=80%, B=53%), but the numbers alone cannot be the conclusion of a thesis — they are evidence pointing in a direction.

---

## Context: Visualisation Bug Fixed

The original run crashed before producing any plots due to a floating-point edge case in the Wilson CI computation. When all `n=4` runs succeed (`k=4, n=4`), the computed upper bound is `0.9999999999999999` instead of exactly `1.0`, making the error bar value `-1.11e-16`. Matplotlib rejects any negative value in `yerr`. Fixed by clamping both error bar values with `max(0.0, ...)` in `visualize.py`.
