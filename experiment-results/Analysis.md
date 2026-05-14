# Pilot Experiment Analysis
**Date:** 2026-05-14  
**Input:** `chess-engine.cpp` — a 1,111-line C++ chess engine (NanoChessTurbo) implementing bitboard move generation, alpha-beta search, quiescence, transposition tables, killer moves, history heuristic, null-move pruning, aspiration windows, and a UCI protocol loop.  
**Task:** Automated C++ → Rust translation with iterative LLM-driven repair.

---

## 1. Experimental Setup

The pipeline runs in three stages for each trial:

1. **Translate** — an LLM translates the C++ source to Rust in one pass.
2. **Compile** — `rustc` compiles the output. If it succeeds, the trial is done.
3. **Repair loop** — if compilation fails, a repair agent receives feedback and rewrites the file. This repeats up to 3 iterations.

The two conditions differ only in what feedback the repair agent sees:

| Condition | Feedback Signal |
|-----------|----------------|
| **A (control)** | Raw `rustc` compiler stderr — unstructured plain text error messages |
| **B (treatment)** | Structured LSP diagnostics — JSON objects with error codes, severity levels, affected character ranges, type hints, and fix suggestions |

Everything else (model, system prompt, max iterations, temperature) was held constant. Three models were tested: `gpt-4o-mini`, `gpt-4o`, and `gpt-5-mini`.

---

## 2. Results Overview

| Model | Cond A outcome | Cond A iters | Cond A time | Cond B outcome | Cond B iters | Cond B time |
|-------|---------------|-------------|------------|---------------|-------------|------------|
| gpt-4o-mini | **FAIL** | 3 (max) | 241s | **FAIL** | 3 (max) | 127s |
| gpt-4o | **SUCCESS** | 3 | 641s | **FAIL** | 3 (max) | 536s |
| gpt-5-mini | **SUCCESS** | 1 | 393s | **SUCCESS** | 1 | 561s |

**Summary:** Condition A produced 2 successes and 1 failure; Condition B produced 1 success and 2 failures. No model converged faster under LSP feedback.

---

## 3. What Each Condition Did — Per Model

### 3.1 gpt-4o-mini

#### Condition A — FAIL (3 iterations, 241s)
The initial translation produced a partial skeleton: all the data structures were declared (TTEntry, HistoryTable, KillerMoves, UCIOptions, SearchStats, Board), but **virtually all logic was absent**. The `Board` struct lacked an `init()` method. Critical functions — `generate_moves`, `make_move`, `search`, `iterative_deepening` — were entirely missing or commented out. `parse_move` contained an empty `Vec::new()` placeholder and had from/to reversed.

The most damaging initial mistake was importing the `rand` crate for Zobrist hashing, which violates the system prompt rule ("do not introduce external crates"). Despite this being called out explicitly in every repair round's stderr, the model never resolved it. After 3 iterations the code still depended on `rand::thread_rng()` and was still missing most of the chess logic.

The final output compiled against nothing — it would not run as a chess engine even if the `rand` error were manually fixed.

#### Condition B — FAIL (3 iterations, 127s)
The initial translation was structurally different: it included a proper `Board::new()` and `Board::init()` with piece placement, a `Board::evaluate()`, and `KillerMoves` backed by `Vec<Vec<Option<Box<Move>>>>`. However, a **single syntax error persisted through all 3 repair iterations**: a missing closing `>` in the nested generic type:

```rust
// what the model wrote (wrong):
killers: Vec<Vec<Option<Box<Move>>>,
// what it should be:
killers: Vec<Vec<Option<Box<Move>>>>,
```

The LSP clearly identified this at line 57, col 40 every single iteration: `expected R_ANGLE`. The model received this hint three times and failed to apply the one-character fix each time. As a result, the file never moved past the parsing stage and was still missing the full search logic (`generate_moves`, `search`, etc.).

Condition B was notably faster (127s vs 241s), likely because the structured LSP format is more concise per round, reducing token count and API latency.

---

### 3.2 gpt-4o

#### Condition A — SUCCESS (3 iterations, 641s)
The initial translation was substantially more complete: full `Board` struct with `init()`, `update()`, `evaluate()` (including king safety and passed pawn bonuses), `generate_moves` with castling and en passant, `make_move` with Zobrist incremental updates, `score_moves` with MVV-LVA and killer/history ordering, `quiescence`, `search` with alpha-beta/null-move/LMR/futility pruning, `iterative_deepening` with aspiration windows, and the full UCI command loop.

Global mutable state was managed with `LazyLock<T>` (stable since Rust 1.80), e.g.:
```rust
static mut HISTORY_TABLE: LazyLock<HistoryTable> = LazyLock::new(|| ...);
```

Initial errors were type-level: `Move` was defined twice, `LazyLock` on `static mut` caused issues, and some type mismatches. The repair agent progressively resolved these over 3 iterations. The final file compiled successfully (only warnings about `static_mut_refs`, which are non-fatal).

This is the most feature-complete output of any run — the translated code closely mirrors the original chess engine's architecture and includes all search optimizations.

#### Condition B — FAIL (3 iterations, 536s)
This translation was also structurally complete, but used `OnceLock<T>` instead of `LazyLock`:
```rust
static HISTORY_TABLE: OnceLock<HistoryTable> = OnceLock::new();
static KILLER_MOVES: OnceLock<KillerMoves> = OnceLock::new();
```

`OnceLock` requires calling `.set()` once and then `.get().unwrap()` for access. The model called `HISTORY_TABLE.get().unwrap().update(...)` and `KILLER_MOVES.get().unwrap().update(...)` — but these return shared references, making mutation impossible. This created irresolvable borrow checker conflicts that the repair agent could not fully fix.

Additional issues: `Instant` cannot be stored in a `static` without `LazyLock`; `b_new()` was introduced as a manual clone helper instead of deriving `Clone`; there was a subtle logic error using `KING_MOVES` to check knight attacks in `is_attacked`.

The LSP feedback correctly highlighted each error with precise locations, but the model repeatedly chose architectural patterns (`OnceLock`) that were fundamentally incompatible with the mutability requirements of a chess engine search loop. After 3 iterations the file still did not compile.

**Notable:** gpt-4o succeeded on the harder condition (raw stderr) but failed on the supposedly easier one (structured LSP). This is the most surprising result of the pilot.

---

### 3.3 gpt-5-mini

#### Condition A — SUCCESS (1 iteration, 393s)
The initial translation used `static mut Option<[U64; 64]>` for move tables and accessed them with `.as_ref().unwrap()`. After one repair iteration (which fixed a `Instant::now()` in a static context), the file compiled cleanly.

The code style was more C-like but idiomatic in its own way: global mutable state as flat `static mut` arrays, manual unsafe blocks, `unsafe fn` accessors for the history/killer/TT tables. All chess logic was present and complete: full board representation, all move generation rules, iterative deepening with aspiration windows, full search with LMR, null move, futility pruning.

A notable implementation detail: Zobrist randomness was generated via a simple LCG (`SimpleRng` with a constant seed) — this is deterministic and correct for the purpose of hashing positions.

#### Condition B — SUCCESS (1 iteration, 561s)
This translation took a slightly different approach for global state: plain `static mut` arrays initialized at compile time with `const fn` constructors:

```rust
static mut HISTORY_TABLE: HistoryTable = HistoryTable::new();
static mut KILLER_MOVES: KillerMoves = KillerMoves::new();
const TT_ENTRY_INIT: TTEntry = TTEntry::new();
static mut TRANSPOSITION_TABLE: [TTEntry; TT_SIZE] = [TT_ENTRY_INIT; TT_SIZE];
```

This is actually the cleanest approach of all runs — it avoids `LazyLock`, `OnceLock`, or `Option` wrappers, and uses `const fn` constructors to enable compile-time initialization of large static arrays.

The single repair iteration fixed the same `Instant::now()` in a static issue as Cond A, but here the LSP feedback additionally pointed out that `start_time` needed to be `Option<Instant>` (initialized to `None`). The fix was applied correctly and the file compiled with only warnings.

The final output for gpt-5-mini condB is arguably the highest-quality Rust translation of the six: complete, compiling, with all chess features, proper memory layout, and idiomatic Rust patterns.

---

## 4. Implications for the Research Questions

The thesis investigates two core questions:

**RQ1: Does LSP feedback improve compilation success rate?**  
Based on this pilot: **no, or possibly the opposite.** Condition A produced 2 successes; Condition B produced 1. The only model that benefited equally from both conditions was gpt-5-mini (success in both). For gpt-4o, raw compiler output led to success while LSP feedback did not. For gpt-4o-mini, neither worked.

This is a small-N pilot (1 input file, 1 repetition per cell), so no statistical conclusions can be drawn. The result is directionally surprising and warrants investigation with more files and repetitions.

**RQ2: Does LSP feedback reduce the number of repair iterations?**  
Based on this pilot: **no.** All models that reached max iterations did so under both conditions. Both gpt-5-mini runs completed in 1 iteration regardless of feedback type. There is no observable iteration-count advantage for LSP.

**RQ3 (implicit): Does feedback quality affect repair time?**  
gpt-4o-mini condB was about 47% faster than condA (127s vs 241s) despite the same outcome. This is tentatively consistent with the hypothesis that structured diagnostics are more efficiently processed per round. However, gpt-5-mini condB was slower than condA (561s vs 393s), so the pattern is not consistent across models.

---

## 5. Additional Observations

### 5.1 Model Capability Dominates Feedback Type
The most predictive variable for success was model capability, not feedback format. gpt-5-mini succeeded in 1 iteration regardless of condition; gpt-4o-mini failed in 3 regardless of condition. This suggests a capability floor below which feedback quality doesn't matter, and a capability ceiling above which it doesn't matter either — the interesting regime may be models in between.

### 5.2 The gpt-4o Reversal Is the Most Interesting Finding
gpt-4o is the only model where the conditions produced different outcomes, and those outcomes are the opposite of what the hypothesis predicts. Under raw compiler stderr, it succeeded. Under structured LSP, it failed. One possible explanation: the feedback signal shaped the repair choices, and the LSP's structured hints may have "nudged" the model toward architectural patterns (OnceLock-based global state) that were incompatible with the code's actual mutability needs. Raw stderr gave the model less guidance, forcing it to make less opinionated choices that turned out to be more compatible.

### 5.3 Persistent Simple Errors (gpt-4o-mini condB)
The fact that gpt-4o-mini failed to fix a single missing `>` across 3 iterations — despite the LSP error message being maximally precise (line, column, "expected R_ANGLE", suggested fix) — indicates a fundamental repair capability failure unrelated to feedback format. The model was receiving correct, specific guidance and still couldn't apply the fix. This is worth flagging as a baseline: structured feedback only helps if the model can act on it.

### 5.4 Initial Translation Quality Varied Within Models
Because only the repair feedback differs between conditions, the initial translation (iter 0) should ideally be similar within a model. In practice, the initial Rust code was visibly different between conditions for gpt-4o and gpt-4o-mini — different global state patterns, different struct designs. This is expected (temperature adds stochasticity), but it means some of the condition differences may reflect initial translation variance rather than feedback effects. Multiple repetitions are needed to control for this.

### 5.5 The Six Outputs Span a Wide Quality Range
Listing all six final outputs by functional completeness:

| Rank | Model | Cond | Status | Completeness |
|------|-------|------|--------|-------------|
| 1 | gpt-5-mini | B | ✅ | Full engine, all features, cleanest global state |
| 2 | gpt-5-mini | A | ✅ | Full engine, all features, Option-wrapped tables |
| 3 | gpt-4o | A | ✅ | Full engine, all features, LazyLock (some warnings) |
| 4 | gpt-4o | B | ❌ | Full engine structure, OnceLock borrow failures |
| 5 | gpt-4o-mini | B | ❌ | Skeleton with Board/eval only, syntax error blocks compile |
| 6 | gpt-4o-mini | A | ❌ | Very incomplete skeleton, missing all search logic, external crate dependency |

### 5.6 Code Style and Rust Idioms by Model
- **gpt-4o-mini** — struggled to translate idiomatically; missed the no-external-crates constraint; produced placeholder code.
- **gpt-4o** — produced idiomatic, structured Rust with modern features (`LazyLock`, `Default` impls, `sort_by`); succeeded when given room to self-correct with simple stderr.
- **gpt-5-mini** — produced the most practically viable Rust: chose patterns (`const fn` static initialization, direct `static mut` arrays) that avoid dynamic initialization entirely, which is both simpler and avoids the most common class of compile errors in this domain.

---

## 6. Caveats and Limitations

- **N=1 input file:** All runs used the same chess engine. Different codebases may interact differently with feedback types. Results may not generalize.
- **N=1 repetition per cell:** Each (model, condition) pair was run once. Stochastic variance in initial translation quality could account for outcome differences.
- **3-iteration cap:** Some models may have converged given more repair rounds. The failure/success boundary at exactly 3 iterations is a design artifact.
- **gpt-5-mini note:** The condA run log listed a path `project_a/test.cpp` rather than `pilot/chess-engine.cpp`; Results.md confirms all results are for the chess engine. The path discrepancy appears to be a labeling artifact from a configuration change.
- **No behavioral testing:** "Success" means `rustc` accepted the file — it does not mean the translated program produces correct chess moves. Semantic correctness was not evaluated.
