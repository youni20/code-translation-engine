from agno.run.agent import RunOutput
from dotenv import load_dotenv
import os
load_dotenv()

from agent import Agno_Agent

if __name__ == "__main__":
    code_translation_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest")
    repair_agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest")
    
    
    
    # run the agent with a prompt
    agent_response: RunOutput = code_translation_agent.ask("Test Prompt")
    
    print(agent_response.content)