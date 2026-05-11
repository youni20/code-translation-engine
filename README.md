# Code Translation Engine

Thesis pipeline for comparing two feedback signals in agentic **C++ → Rust** translation:

- **Condition A** — repair agent sees raw `rustc` stderr.
- **Condition B** — repair agent sees structured **LSP diagnostics** from rust-analyzer.

Everything else (model, prompts, iteration cap, input code) is held constant. The dependent variable is "does it compile?" after a fixed number of repair iterations.

## How it works

![Pipeline flow chart](images/cte_pipeline_diagram.png)

C++ source → translation agent → Rust code → compile with `rustc`. If it fails, the repair agent is handed the broken code plus feedback (Condition A = raw stderr, Condition B = formatted LSP diagnostics) and emits new code. Loop until it compiles or `MAX_ITERATIONS` is hit.

## Layout

| Path | What it does |
|---|---|
| `src/main.py` | Entry point. Sets `N_RUNS`, `MAX_ITERATIONS`, `CONDITION`, model. |
| `src/pipeline.py` | Translate-then-repair loop. Picks A or B feedback. |
| `src/agent.py` | Thin wrapper around `agno.Agent` for the LLM. |
| `src/compile_rust.py` | Compiles a Rust string with `rustc`, returns `(ok, stderr)`. |
| `src/lsp_tool.py` | Writes Rust to disk, asks rust-analyzer for diagnostics. |
| `src/prompts.py` | System prompts for the translator and repairer. |
| `src/metrics.py` | Aggregates compile rate + iteration stats over runs. |
| `tests/` | C++ inputs (`two_sum.cpp`, `welcome.cpp`) + a `rustc` smoke test. |
| `outputs/rust_project/` | Cargo project where Condition B writes `src/output.rs`. |

## Setup

```bash
git clone git@github.com:youni20/code-translation-engine.git
cd code-translation-engine
uv venv --python 3.12
uv sync                            # installs everything from pyproject.toml
source .venv/bin/activate          # fish: source .venv/bin/activate.fish
```

You also need:
- `rustc` and `cargo` (install via [rustup](https://rustup.rs))
- `rust-analyzer` on `PATH`
- An Ollama server with the model in `src/main.py` pulled (`ollama pull gemma3:latest`)

## Run

Edit the three knobs at the top of `src/main.py`:

```python
N_RUNS = 5
MAX_ITERATIONS = 5
CONDITION = "A"   # or "B"
```

Then:

```bash
uv run src/main.py
```

The final Rust output lands in `outputs/rust_project/src/output.rs` and metrics print to stdout.

## How A and B differ

| | Condition A | Condition B |
|---|---|---|
| Feedback to repair agent | `rustc` stderr (raw text) | LSP diagnostics (severity, code, line:col, message) |
| Compiler invoked? | Yes (`rustc`) | Yes (`rustc`), plus rust-analyzer for diagnostics |
| Code changes between conditions? | No | No |
