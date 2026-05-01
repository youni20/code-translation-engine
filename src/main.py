from dotenv import load_dotenv
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT
from io_utils import local_file_reader, local_file_writer

load_dotenv()
from agent import Agno_Agent



if __name__ == "__main__":
    
    #  Initializing the repair and translation agents
    code_translation_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest", description=TRANSLATION_SYSTEM_PROMPT)  # OpenAI Model: gpt-5.4-2026-03-05
    repair_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest", description=REPAIR_SYSTEM_PROMPT)


    cpp_code = local_file_reader("./inputs/two_sum.cpp")

    # run the agent with a prompt
    agent_response = code_translation_agent.ask(
        f"""C++ input:
    
    {cpp_code}
    
    Rust output:"""
    )

    response = agent_response.content or ""
    local_file_writer(response, "./outputs/output.rs")