from typing import Any, Optional, Self
from mal_token import MalToken
from mal_error import MalSymbolNotFoundError


class MalEnv:
    def __init__(self, outer: Optional['MalEnv'] = None):
        self.outer = outer
        self.data: dict[str, Any] = {}

    def set(self, key: str, value: Any) -> None:
        self.data[key] = value

    def get(self, key:str) -> Any:
        if key in self.data:
            return self.data[key]
        if self.outer:
            return self.outer.get(key)
        raise MalSymbolNotFoundError(key)