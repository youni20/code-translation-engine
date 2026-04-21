from agno.agent import Agent
from agno.models.ollama import Ollama

if __name__ == "__main__":
    agent = Agent(
        model=Ollama(id="gemma3:latest"),
        markdown=True
    )
    
    agent.print_response("Testing: can you read this?")