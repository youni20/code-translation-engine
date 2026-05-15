import re
import time

from agent import Agno_Agent
from compile_rust import compile_rust
from dataset import TranslationUnit
from lsp_tool import get_lsp_diagnostics
from results import RunResult


# To remove redundant markdown from llm output
def strip_markdown_fences(code: str) -> str:
    # Matches ```rust ... ``` or ``` ... ```
    pattern = r"^```(?:rust)?\n(.*?)```$"
    match = re.search(pattern, code, re.DOTALL)
    return match.group(1).strip() if match else code.strip()


def run_translation_pipeline(
    unit: TranslationUnit,
    translator: Agno_Agent,
    repairer: Agno_Agent,
    condition: str,
    repetition: int,
    max_iterations: int = 5,
) -> RunResult:
    """
    Translate one C++ translation unit to Rust and iteratively repair compilation errors.

    Args:
        unit: The C++ source unit to translate.
        translator: Agent for the initial C++ to Rust translation.
        repairer: Agent for fixing compilation errors.
        condition: "A" for raw compiler stderr feedback, "B" for stderr + LSP diagnostics.
        repetition: Which repetition of this (unit, condition) pair this run represents.
        max_iterations: Maximum number of repair attempts after the initial translation.

    Returns:
        A fully-populated RunResult capturing the outcome and provenance.
    """
    start_time = time.monotonic()
    feedback_history: list[str] = []
    feedback_source = "compiler stderr" if condition == "A" else "compiler stderr + LSP diagnostics"

    # Initial translation
    print("    [translate] generating initial Rust translation ...")
    translation_prompt = f"C++ input:\n\n{unit.source_code}\n\nRust output:"
    response = translator.ask(translation_prompt)
    if response is None or response.content is None:
        raise RuntimeError(
            f"Translation agent returned no content for {unit.unit_id}. "
            f"Full response: {response}"
        )
    rust_code = strip_markdown_fences(response.content)

    # Repair loop
    final_stderr = ""
    success = False
    iterations_used = 0

    for iteration in range(max_iterations):
        success, stderr = compile_rust(rust_code)
        final_stderr = stderr
        status = "PASS" if success else "FAIL"
        print(f"    [compile]   iter {iteration}: {status}")
        if success:
            iterations_used = iteration
            break

        if condition == "A":
            feedback = stderr
        elif condition == "B":
            lsp = get_lsp_diagnostics(rust_code=rust_code)
            feedback = f"Compiler stderr:\n{stderr}\n\nLSP diagnostics:\n{lsp}"
        else:
            raise ValueError(f"Unknown condition: {condition!r}. Use 'A' or 'B'.")

        feedback_history.append(feedback)

        print(f"    [repair]    iter {iteration + 1}: sending {feedback_source} to repair agent ...")
        repair_prompt = (
            f"The following Rust code failed to compile.\n\n"
            f"Code:\n{rust_code}\n\n"
            f"Compiler feedback:\n{feedback}\n\n"
            f"Corrected code:"
        )
        response = repairer.ask(repair_prompt)
        if response is None or response.content is None:
            raise RuntimeError(
                f"Repair agent returned no content on iteration {iteration} "
                f"for {unit.unit_id}. Full response: {response}"
            )
        rust_code = strip_markdown_fences(response.content)
        iterations_used = iteration + 1
    else:
        # Loop completed without break: run one final compile check
        success, final_stderr = compile_rust(rust_code)
        status = "PASS" if success else "FAIL"
        print(f"    [compile]   final check: {status}")

    wall_time = time.monotonic() - start_time

    return RunResult(
        unit_id=unit.unit_id,
        project=unit.project,
        relative_path=unit.relative_path,
        loc=unit.loc,
        condition=condition,
        repetition=repetition,
        success=success,
        iterations_used=iterations_used,
        final_rust_code=rust_code,
        final_stderr=final_stderr,
        feedback_history=feedback_history,
        wall_time_seconds=wall_time,
    )