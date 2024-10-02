from abc import ABC
from dataclasses import dataclass
from mal_error import MalSyntaxError

@dataclass()
class MalToken:
    value: str
    start: int
    end: int

class MalSymbol(MalToken):
    def __str__(self):
        return self.value

class MalNumber(MalToken):
    numeric_value: int

    def __init__(self, value: str, start: int, end: int):
        super().__init__(value, start, end)
        self.numeric_value = int(value)

    def __str__(self):
        return str(self.numeric_value)

class MalCollection(MalToken, ABC):
    elements: list[MalToken]
    start_token: str
    end_token: str

    def __init__(self, start_token: str, end_token: str, elements: list[MalToken], start: int, end: int):
        self.elements = elements
        self.start_token = start_token
        self.end_token = end_token
        super().__init__("(", start, end)

    def __str__(self):
        return f"{self.start_token}{' '.join(str(e) for e in self.elements)}{self.end_token}"

class MalList(MalCollection):
    def __init__(self, elements: list[MalToken], start: int, end: int):
        super().__init__("(", ")", elements, start, end)

class MalVector(MalCollection):
    def __init__(self, elements: list[MalToken], start: int, end: int):
        super().__init__("[", "]", elements, start, end)

class MalHashMap(MalToken):
    data: dict[str, MalToken]

    def __init__(self, elements: list[MalToken], start: int, end: int):
        if len(elements) % 2 != 0:
            raise MalSyntaxError("Keys and values must have the same length", start)
        keys = elements[0::2]
        values = elements[1::2]
        self.data = dict(zip(map(str, keys), values))
        

    def __str__(self):
        return "{" + " ".join(f"{k} {v}" for k, v in self.data.items()) + "}"

class MalBoolean(MalToken):
    boolean_value: bool

    def __init__(self, value: str, start: int, end: int):
        super().__init__(value, start, end)
        self.boolean_value = value == "true"

    def __str__(self):
        return "true" if self.boolean_value else "false"

class MalNil(MalToken):
    def __str__(self):
        return "nil"
    
class MalString(MalToken):
    def __str__(self, print_readably: bool = True):
        return f"\"{self.value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n")}\"" if print_readably else self.value