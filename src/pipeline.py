from agent import Agno_Agent
from compile_rust import compile_rust
import re


# To remove redundent markdown from llm
def strip_markdown_fences(code: str) -> str:
    # Matches ```rust ... ``` or ``` ... ```
    pattern = r"^```(?:rust)?\n(.*?)```$"
    match = re.search(pattern, code, re.DOTALL)
    return match.group(1).strip() if match else code.strip()


def run_translation_pipeline(
    cpp_code: str,
    translator: Agno_Agent,
    repairer: Agno_Agent,
    condition: str = "A",
    max_iterations: int = 5,  # If we wanna change the number of itterations do so here
) -> tuple[str, bool, int]:
    """
    Translate C++ to Rust and iteratively repair compilation errors.

    Args:
        cpp_code: Source C++ code as a string.
        translator: Agent for the initial C++ to Rust translation.
        repairer: Agent for fixing compilation errors.
        condition: "A" for raw compiler stderr feedback, "B" for LSP diagnostics.
        max_iterations: Maximum number of repair attempts after the initial translation.

    Returns:
        (final_rust_code, success, iterations_used):
            final_rust_code: The last version of the Rust code produced.
            success: True if the final code compiles cleanly.
            iterations_used: Number of repair iterations executed (0 if first translation succeeded).
    """
    # Initial translation
    translation_prompt = f"C++ input:\n\n{cpp_code}\n\nRust output:"
    response = translator.ask(translation_prompt)
    if response is not None and response.content is not None:
        rust_code = strip_markdown_fences(response.content)
    else:
        raise RuntimeError(
            f"Translation agent returned no content. Full response: {response}"
        )

    # Repair loop
    for iteration in range(max_iterations):
        success, stderr = compile_rust(rust_code)
        if success:
            return rust_code, True, iteration

        if condition == "A":
            feedback = stderr
        elif condition == "B":
            raise NotImplementedError(
                "Condition B requires LSP integration (not yet implemented)."
            )
        else:
            raise ValueError(f"Unknown condition: {condition!r}. Use 'A' or 'B'.")

        repair_prompt = (
            f"The following Rust code failed to compile.\n\n"
            f"Code:\n{rust_code}\n\n"
            f"Compiler feedback:\n{feedback}\n\n"
            f"Corrected code:"
        )
        response = repairer.ask(repair_prompt)
        if response is not None and response.content is not None:
            new_code = strip_markdown_fences(response.content)
        else:
            raise RuntimeError(
                f"Repair agent returned no content on iteration {iteration}. "
                f"Full response: {response}"
            )
        rust_code = new_code

    # Final compile check after the last repair iteration
    success, _ = compile_rust(rust_code)
    return rust_code, success, max_iterations