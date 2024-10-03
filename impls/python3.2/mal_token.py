from dataclasses import dataclass

@dataclass()
class MalToken:
    value: str
    start: int
    end: int

    def str(self, print_readably: bool = False) -> str:
        return self.value