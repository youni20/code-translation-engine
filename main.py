from agent import Agno_Agent
from agno.tools.hackernews import HackerNewsTools


if __name__ == "__main__":
    agent = Agno_Agent(model_id="gemma3:latest")
    
    # run the agent with a prompt
    agent_response = agent.ask("Testing do you work?")
    
    print(agent_response.content)