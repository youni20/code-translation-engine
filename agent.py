from configparser import NoOptionError
from agno.agent import Agent
from agno.models.ollama.chat import Ollama
from agno.run.agent import RunOutput

from dotenv import load_dotenv
import os

load_dotenv()

class Agno_Agent:
    def __init__(self, model_id: str) -> None:  # Initializes Agent 
        self.model_id = model_id
        self._tool_registry = []
        self.instance = self._build_instance()
        
    def _build_instance(self) -> Agent:
        return Agent(
            model = Ollama(id=self.model_id),
            markdown=True,
            tools=[]
        )
        
    def add_tool(self, new_tool) -> None:
        self._tool_registry.append(new_tool)
        self.instance = self._build_instance()  # Rebuild the agent instace after adding new tool
    
    def ask(self, question: str) -> RunOutput:
        return self.instance.run(question)
