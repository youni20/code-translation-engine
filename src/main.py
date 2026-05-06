from dotenv import load_dotenv
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT
from io_utils import local_file_reader, local_file_writer
from pipeline import run_translation_pipeline
from metrics import compute_metrics, CompilationMetrics

load_dotenv()

from agent import Agno_Agent

N_RUNS = 5
MAX_ITERATIONS = 5
CONDITION = "A"

if __name__ == "__main__":
    code_translation_agent: Agno_Agent = Agno_Agent(
        model_id="gemma3:latest",  # gpt-5.4-mini
        description=TRANSLATION_SYSTEM_PROMPT  
    )
    repair_agent: Agno_Agent = Agno_Agent(
        model_id="gemma3:latest", 
        description=REPAIR_SYSTEM_PROMPT
    )

    cpp_code = local_file_reader("./inputs/two_sum.cpp")

    runs: list[tuple[str, bool, int]] = []
    rust_code: str = ""

    for run_index in range(N_RUNS):
        print(f"Run {run_index + 1}/{N_RUNS}...")
        rust_code, success, iterations = run_translation_pipeline(
            cpp_code=cpp_code,
            translator=code_translation_agent,
            repairer=repair_agent,
            condition=CONDITION,
            max_iterations=MAX_ITERATIONS,
        )
        runs.append((rust_code, success, iterations))
        print(f"  Success: {success}, Iterations: {iterations}")

    local_file_writer(rust_code, "./outputs/rust_project/src/output.rs")


    metrics: CompilationMetrics = compute_metrics(runs=runs)
    print(f"\n--- Condition {CONDITION} Results ---")
    print(f"Runs:              {metrics.n_runs}")
    print(f"Compiled:          {metrics.n_compiled}")
    print(f"Compilation rate:  {metrics.compilation_rate:.2%}")
    print(f"First-try:         {metrics.n_first_try}")
    print(f"Exhausted:         {metrics.n_exhausted}")
    print(f"Mean iterations:   {metrics.mean_iterations_to_compile}")
    print(f"Median iterations: {metrics.median_iterations_to_compile}")