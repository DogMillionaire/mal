from abc import ABC
from mal_error import MalSyntaxError
from mal_token import MalToken

class MalSymbol(MalToken):
    def __str__(self):
        return self.value
    
    def __eq__(self, other):
        if isinstance(other, MalToken):
            return MalBoolean(self.value == other.value, self.start, other.end)
        return MalBoolean(False)

class MalNumber(MalToken):
    numeric_value: int

    def __init__(self, value: str, start: int = -1, end: int = -1):
        super().__init__(value, start, end)
        self.numeric_value = int(value)

    def __str__(self):
        return str(self.numeric_value)
    
    def __add__(self, other):
        if isinstance(other, MalNumber):
            return MalNumber(str(self.numeric_value + other.numeric_value), self.start, other.end)
        raise MalSyntaxError(f"Cannot add {type(other)} to a number", self.start)
    
    def __sub__(self, other):
        if isinstance(other, MalNumber):
            return MalNumber(str(self.numeric_value - other.numeric_value), self.start, other.end)
        raise MalSyntaxError(f"Cannot subtract {type(other)} from a number", self.start)
    
    def __mul__(self, other):
        if isinstance(other, MalNumber):
            return MalNumber(str(self.numeric_value * other.numeric_value), self.start, other.end)
        raise MalSyntaxError(f"Cannot multiply {type(other)} with a number", self.start)
    
    def __truediv__(self, other):
        if isinstance(other, MalNumber):
            if other.numeric_value == 0:
                raise MalSyntaxError("Division by zero", self.start)
            return MalNumber(str(self.numeric_value // other.numeric_value), self.start, other.end)
        raise MalSyntaxError(f"Cannot divide {type(other)} by a number", self.start)
    
    def __gt__(self, other):
        if isinstance(other, MalNumber):
            return MalBoolean(self.numeric_value > other.numeric_value, self.start, other.end)
        raise MalSyntaxError(f"Cannot compare {type(other)} with a number", self.start)
    
    def __ge__(self, other):
        if isinstance(other, MalNumber):
            return MalBoolean(self.numeric_value >= other.numeric_value, self.start, other.end)
        raise MalSyntaxError(f"Cannot compare {type(other)} with a number", self.start)
    
    def __lt__(self, other):
        if isinstance(other, MalNumber):
            return MalBoolean(self.numeric_value < other.numeric_value, self.start, other.end)
        raise MalSyntaxError(f"Cannot compare {type(other)} with a number", self.start)
    
    def __le__(self, other):
        if isinstance(other, MalNumber):
            return MalBoolean(self.numeric_value <= other.numeric_value, self.start, other.end)
        raise MalSyntaxError(f"Cannot compare {type(other)} with a number", self.start)
    
    def __eq__(self, other):
        if isinstance(other, MalNumber):
            return MalBoolean(self.numeric_value == other.numeric_value, self.start, other.end)
        return MalBoolean(False)


class MalCollection(MalToken, ABC):
    elements: list[MalToken]
    size: int
    start_token: str
    end_token: str

    def __init__(self, start_token: str, end_token: str, elements: list[MalToken], start: int, end: int):
        self.elements = elements
        self.size = len(elements)
        self.start_token = start_token
        self.end_token = end_token
        super().__init__("(", start, end)

    def __str__(self):
        return f"{self.start_token}{' '.join(str(e) for e in self.elements)}{self.end_token}"
    
    def str(self, print_readably: bool = False) -> str:
        return f"{self.start_token}{' '.join((e.str(print_readably)) for e in self.elements)}{self.end_token}"
    
    def __eq__(self, other):
        if isinstance(other, MalCollection):
            if self.size != other.size:
                return MalBoolean(False)
            return MalBoolean(all(a == b for a, b in zip(self.elements, other.elements)))
        return MalBoolean(False)

class MalList(MalCollection):
    def __init__(self, elements: list[MalToken], start: int = -1, end: int = -1):
        super().__init__("(", ")", elements, start, end)

class MalVector(MalCollection):
    def __init__(self, elements: list[MalToken], start: int, end: int):
        super().__init__("[", "]", elements, start, end)

class MalHashMap(MalToken):
    data: dict[str, MalToken]
    size: int

    def __init__(self, elements: list[MalToken], start: int, end: int):
        self.start = start
        self.end = end
        if len(elements) % 2 != 0:
            raise MalSyntaxError("Keys and values must have the same length", start)
        keys = elements[0::2]
        values = elements[1::2]
        self.data = dict(zip(map(str, keys), values))
        self.size = len(self.data)

    def __str__(self):
        return "{" + " ".join(f"{k} {v}" for k, v in self.data.items()) + "}"
    
    def str(self, print_readably: bool = False) -> str:
        return "{" + " ".join(f"{k} {v.str(print_readably)}" for k, v in self.data.items()) + "}"

    def __eq__(self, other):
        if isinstance(other, MalHashMap):
            if self.size != other.size:
                return MalBoolean(False)
            return MalBoolean(all(a == b for a, b in zip(self.data.items(), other.data.items())))
        return MalBoolean(False)

class MalBoolean(MalToken):
    boolean_value: bool

    def __init__(self, value: bool, start: int = -1, end: int = -1):
        super().__init__("true" if value else "false", start, end)
        self.boolean_value = value

    def __str__(self):
        return "true" if self.boolean_value else "false"
    
    def __eq__(self, other):
        if isinstance(other, MalBoolean):
            return MalBoolean(self.value == other.value, self.start, other.end)
        return MalBoolean(False)

class MalNil(MalToken):
    def __init__(self, value: str = "nil", start: int = -1, end: int = -1):
        super().__init__(value, start, end)

    def __str__(self):
        return "nil"
    
    def __eq__(self, other):
        if isinstance(other, MalNil):
            return MalBoolean(True)
        return MalBoolean(False)
    
class MalString(MalToken):
    def __str__(self, print_readably: bool = True):
        return f"\"{self.value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n")}\"" if print_readably else self.value
    
    def str(self, print_readably: bool = True):
        return f"\"{self.value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n")}\"" if print_readably else self.value
    

    def __eq__(self, other):
        if isinstance(other, MalString):
            return MalBoolean(self.value == other.value, self.start, other.end)
        return MalBoolean(False)
    
class MalKeyword(MalToken):
    def __str__(self):
        return f":{self.value}"
    
    def str(self, _: bool):
        return f":{self.value}"
    
    def __eq__(self, other):
        if isinstance(other, MalKeyword):
            return MalBoolean(self.value == other.value, self.start, other.end)
        return MalBoolean(False)