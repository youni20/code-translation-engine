import os
from agno.agent import Agent
from agno.models.groq import Groq
from dotenv import load_dotenv

# external LLM API via Groq for now
load_dotenv()  # load from .env
groq_api_key = os.getenv("GROQ_API_KEY")

if __name__ == "__main__":
    agent = Agent(model=Groq(id="llama-3.3-70b-versatile"), markdown=False)
    agent.print_response("Give me a programming joke.")

