all the results are for chess engine c++ file.
# gpt-4o-mini:
## condA: 
```
======================================================================
  Run ID:            2026-05-14_14-37-01
  Units:             1
  Conditions:        A
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_14-37-01/results.jsonl
======================================================================

[1/1] pilot/chess-engine.cpp  (cond=A, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending compiler stderr to repair agent ...
    [compile]   iter 1: FAIL
    [repair]    iter 2: sending compiler stderr to repair agent ...
    [compile]   iter 2: FAIL
    [repair]    iter 3: sending compiler stderr to repair agent ...
    [compile]   final check: FAIL
  [done]      FAILED   iters=3  time=241.4s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_14-37-01/results.jsonl
======================================================================
```
## condB: 
```
======================================================================
  Run ID:            2026-05-14_14-41-48
  Units:             1
  Conditions:        B
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_14-41-48/results.jsonl
======================================================================

[1/1] pilot/chess-engine.cpp  (cond=B, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending LSP diagnostics to repair agent ...
    [compile]   iter 1: FAIL
    [repair]    iter 2: sending LSP diagnostics to repair agent ...
    [compile]   iter 2: FAIL
    [repair]    iter 3: sending LSP diagnostics to repair agent ...
    [compile]   final check: FAIL
  [done]      FAILED   iters=3  time=126.5s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_14-41-48/results.jsonl
======================================================================
```

Here are "finished" rust files for condA and condB:
[[experiment-results/gpt-4o-mini/condA.rs|condA.rs]] [[experiment-results/gpt-4o/condB.rs]]

# gpt-4o:
## condA: 
```
======================================================================
  Run ID:            2026-05-14_14-46-39
  Units:             1
  Conditions:        A
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_14-46-39/results.jsonl
======================================================================

[1/1] pilot/chess-engine.cpp  (cond=A, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending compiler stderr to repair agent ...
    [compile]   iter 1: FAIL
    [repair]    iter 2: sending compiler stderr to repair agent ...
    [compile]   iter 2: FAIL
    [repair]    iter 3: sending compiler stderr to repair agent ...
    [compile]   final check: PASS
  [done]      SUCCESS  iters=3  time=640.9s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_14-46-39/results.jsonl
======================================================================
```
## condB: 
```
======================================================================
  Run ID:            2026-05-14_15-06-48
  Units:             1
  Conditions:        B
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_15-06-48/results.jsonl
======================================================================

[1/1] pilot/chess-engine.cpp  (cond=B, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending LSP diagnostics to repair agent ...
    [compile]   iter 1: FAIL
    [repair]    iter 2: sending LSP diagnostics to repair agent ...
    [compile]   iter 2: FAIL
    [repair]    iter 3: sending LSP diagnostics to repair agent ...
    [compile]   final check: FAIL
  [done]      FAILED   iters=3  time=535.8s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_15-06-48/results.jsonl
======================================================================
```

Here are "finished" rust files for condA and condB:
[[experiment-results/gpt-4o/condA.rs|condA.rs]] [[experiment-results/gpt-4o/condB.rs|condB.rs]]
# gpt-5-mini:
## condA: 
```
======================================================================
  Run ID:            2026-05-14_15-02-06
  Units:             1
  Conditions:        A
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_15-02-06/results.jsonl
======================================================================

[1/1] project_a/test.cpp  (cond=A, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending compiler stderr to repair agent ...
    [compile]   iter 1: PASS
  [done]      SUCCESS  iters=1  time=393.1s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_15-02-06/results.jsonl
======================================================================
```
## condB: 
```
======================================================================
  Run ID:            2026-05-14_15-18-44
  Units:             1
  Conditions:        B
  Repetitions:       1
  Total invocations: 1
  Results file:      outputs/runs/2026-05-14_15-18-44/results.jsonl
======================================================================

[1/1] pilot/chess-engine.cpp  (cond=B, rep=0)
    [translate] generating initial Rust translation ...
    [compile]   iter 0: FAIL
    [repair]    iter 1: sending LSP diagnostics to repair agent ...
    [compile]   iter 1: PASS
  [done]      SUCCESS  iters=1  time=560.7s

======================================================================
  Experiment complete. Results: outputs/runs/2026-05-14_15-18-44/results.jsonl
======================================================================
[visualize] Wrote CSVs and plots to outputs/runs/2026-05-14_15-18-44
```

Here are "finished" rust files for condA and condB:
[[experiment-results/gpt-5-mini/condA.rs|condA.rs]] [[experiment-results/gpt-5-mini/condB.rs|condB.rs]]
Note that the discrepancy in the output (gpt-5-mini vs the previous model runs) is because we changed the visualization logic a little bit. Nothing else though.
