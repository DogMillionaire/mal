from dataclasses import dataclass

@dataclass()
class MalToken:
    value: str
    start: int = -1
    end: int = -1

    def str(self, print_readably: bool = False) -> str:
        return self.value