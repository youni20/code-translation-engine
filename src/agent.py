from agno.agent import Agent
from agno.run.agent import RunOutput
from typing import Optional

#  Models
from agno.models.ollama.chat import Ollama
from agno.models.openai import OpenAIChat

class Agno_Agent:
    def __init__(self, model_id: str, description: str = "") -> None:  # Initializes Agent 
        self.model_id: str = model_id
        self.description = description
        self._tool_registry: list = []
        self.instance: Optional[Agent] = None
        
    def _build_instance(self) -> Agent:
        return Agent(
            model = OpenAIChat(id=self.model_id),
            tools=self._tool_registry,
            markdown=False,
            debug_mode=True,
            description=self.description
        )
        
    def add_tool(self, new_tool) -> None:
        self._tool_registry.append(new_tool)
        self.instance = None  # Instance gets rebuilt on next "ask"
    
    def ask(self, question: str) -> RunOutput:
        if self.instance is None:
            self.instance = self._build_instance()
        return self.instance.run(question)
