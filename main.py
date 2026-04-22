from dotenv import load_dotenv
import os

load_dotenv()

from agno.run.agent import RunOutput
from agent import Agno_Agent
from agno.tools.hackernews import HackerNewsTools

if __name__ == "__main__":
    agent: Agno_Agent = Agno_Agent(model_id="llama-3.3-70b-versatile")
    
    #  Testing adding a tool
    agent.add_tool(HackerNewsTools())
    
    # run the agent with a prompt
    agent_response: RunOutput = agent.ask("What is the top story on HackerNews?")
    
    print(agent_response.content)