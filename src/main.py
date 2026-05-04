from dotenv import load_dotenv
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT
from io_utils import local_file_reader, local_file_writer
from pipeline import run_translation_pipeline

load_dotenv()

from agent import Agno_Agent

if __name__ == "__main__":
    code_translation_agent: Agno_Agent = Agno_Agent(
        model_id="gemma3:latest",
        description=TRANSLATION_SYSTEM_PROMPT
    )
    repair_agent: Agno_Agent = Agno_Agent(
        model_id="gemma3:latest",
        description=REPAIR_SYSTEM_PROMPT
    )

    cpp_code = local_file_reader("./inputs/two_sum.cpp")

    rust_code, success, iterations = run_translation_pipeline(
        cpp_code=cpp_code,
        translator=code_translation_agent,
        repairer=repair_agent,
        condition="A"
    )

    local_file_writer(rust_code, "./outputs/rust_project/src/output.rs")
    print(f"Success: {success}, Iterations: {iterations}")