import os
from agno.agent import Agent

# note that we have many more models
# to try if 70b-versatile runs out
from agno.models.groq import Groq
from agno.tools.hackernews import HackerNewsTools
from dotenv import load_dotenv

# external LLM API via Groq for now
load_dotenv()  # load from .env
groq_api_key = os.getenv("GROQ_API_KEY")

if __name__ == "__main__":
    # define agent
    agent = Agent(
        model=Groq(id="llama-3.3-70b-versatile"),
        # we will use tools for
        # LSP access in future
        tools=[
            HackerNewsTools(),  # just an example
        ],
    )

    # run the agent with a prompt
    agent.print_response("What are the top stories on HackerNews?", markdown=True)

