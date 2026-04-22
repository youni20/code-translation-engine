from agno.agent import Agent
from agno.models.ollama.chat import Ollama
from agno.run.agent import RunOutput

from dotenv import load_dotenv
import os

load_dotenv()

class Agno_Agent:
    def __init__(self, model_id: str) -> None:  # Initializes Agent 
        self.model_id = model_id
        self.__agent = Agent(
            model = Ollama(id=self.model_id),
            markdown=True
        )
    
    def ask(self, question: str) -> RunOutput:
        return self.__agent.run(question)
