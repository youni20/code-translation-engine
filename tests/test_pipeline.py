from pathlib import Path
from dotenv import load_dotenv
load_dotenv()

from agent import Agno_Agent
from dataset import TranslationUnit
from pipeline import run_translation_pipeline
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT
from io_utils import local_file_reader

cpp = local_file_reader("./tests/two_sum.cpp")
unit = TranslationUnit(
    project="tests",
    relative_path="two_sum.cpp",
    source_code=cpp,
    loc=len(cpp.splitlines()),
)

translator = Agno_Agent(model_id="gpt-4o-mini", description=TRANSLATION_SYSTEM_PROMPT)
repairer = Agno_Agent(model_id="gpt-4o-mini", description=REPAIR_SYSTEM_PROMPT)

result = run_translation_pipeline(
    unit=unit,
    translator=translator,
    repairer=repairer,
    condition="A",
    repetition=0,
    max_iterations=3,
)

print(f"Success: {result.success}")
print(f"Iterations: {result.iterations_used}")
print(f"Wall time: {result.wall_time_seconds:.2f}s")
print(f"Unit ID: {result.unit_id}")
print(f"Feedback rounds: {len(result.feedback_history)}")