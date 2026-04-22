from agno.run.agent import RunOutput
from agent import Agno_Agent

from agno.tools.hackernews import HackerNewsTools


if __name__ == "__main__":
    agent: Agno_Agent = Agno_Agent(model_id="gemma3:latest")
    
    #  Testing adding a tool
    agent.add_tool(HackerNewsTools())
    
    # run the agent with a prompt
    agent_response: RunOutput = agent.ask("What is the latest on HackerNews? cite your source")
    
    print(agent_response.content)