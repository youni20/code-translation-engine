from agno.agent import Agent
from agno.run.agent import RunOutput

#  Models
# from agno.models.ollama.chat import Ollama
from agno.models.groq import Groq


class Agno_Agent:
    def __init__(self, model_id: str) -> None:  # Initializes Agent 
        self.model_id: str = model_id
        self._tool_registry: list = []
        self.instance: Agent = self._build_instance()
        
    def _build_instance(self) -> Agent:
        return Agent(
            model = Groq(id=self.model_id),
            tools=self._tool_registry,
            markdown=True,
            debug_mode=True
        )
        
    def add_tool(self, new_tool) -> None:
        self._tool_registry.append(new_tool)
        self.instance = self._build_instance()  # Rebuild the agent instace after adding new tool
    
    def ask(self, question: str) -> RunOutput:
        return self.instance.run(question)
