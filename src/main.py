from dotenv import load_dotenv
from prompts import TRANSLATION_SYSTEM_PROMPT, REPAIR_SYSTEM_PROMPT

load_dotenv()
from agent import Agno_Agent



def local_file_reader() -> str:
    with open("./inputs/two_sum.cpp", "r") as file:
        file_content = file.read()
        return file_content


def local_file_writer(content: str) -> None:
    with open("./outputs/output.rs", "w") as file:
        file.write(content)



if __name__ == "__main__":
    
    #  Initializing the repair and translation agents
    code_translation_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest", description=TRANSLATION_SYSTEM_PROMPT)  # OpenAI Model: gpt-5.4-2026-03-05
    repair_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest", description=REPAIR_SYSTEM_PROMPT)


    cpp_code = local_file_reader()

    # run the agent with a prompt
    agent_response = code_translation_agent.ask(
        f"""C++ input:
    
    {cpp_code}
    
    Rust output:"""
    )

    response = agent_response.content
    local_file_writer(response or "")