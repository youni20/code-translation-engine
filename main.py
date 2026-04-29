from agno.run.agent import RunOutput
from dotenv import load_dotenv
import os
load_dotenv()

from agent import Agno_Agent


def local_file_reader() -> str:
    with open("./cpp_translation_samples/two_sum.cpp", "r") as file:
        file_content = file.read()
        return file_content


def local_file_writer(content: str) -> None:
    with open("./translated_rust_code/output.rs", "w") as file:
        file.write(content)


if __name__ == "__main__":
    code_translation_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest")
    # OpenAI Model: gpt-5.4-2026-03-05
    # repair_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest")

    cpp_code = local_file_reader()

    # run the agent with a prompt
    agent_response: RunOutput = code_translation_agent.ask(
        f"""You are an expert code translator specialising in C++ to Rust migration. You output only valid Rust source code. No explanations, no markdown, no commentary. Translate the following C++ code into idiomatic, compilable Rust.

Rules:
1. Translate faithfully. Do not add, remove, or alter any functionality.
2. Use idiomatic Rust: Option instead of nullptr, Result instead of exceptions, ownership and borrowing instead of raw pointers.
3. Preserve original function signatures as closely as Rust's type system allows. If a direct mapping is not possible, add a brief inline comment.
4. Do not introduce external crates. Use only the Rust standard library.
5. The output must be a complete, standalone .rs file that compiles with rustc. Include all necessary use statements, type definitions, and function definitions.
6. Your response must begin with the first line of Rust code and end with the last line. No markdown fences, no preamble, no explanation.

C++ input:

{cpp_code}

Rust output:"""
    )

    response = agent_response.content
    local_file_writer(response or "")