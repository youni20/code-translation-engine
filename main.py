from agno.run.agent import RunOutput
from dotenv import load_dotenv
import os
load_dotenv()

from agent import Agno_Agent

if __name__ == "__main__":
    agent: Agno_Agent = Agno_Agent(model_id="gemini-2.5-flash")    
    
    # run the agent with a prompt
    agent_response: RunOutput = agent.ask("Test Prompt")
    
    print(agent_response.content)