from agno.agent import Agent
from agno.models.openai import OpenAIChat
from agno.run.agent import RunOutput


class Agno_Agent:
    def __init__(self, model_id: str, description: str = "") -> None:
        self.instance: Agent = Agent(
            model=OpenAIChat(id=model_id),
            markdown=False,
            debug_mode=True,
            description=description,
        )

    def ask(self, question: str) -> RunOutput:
        return self.instance.run(question)
