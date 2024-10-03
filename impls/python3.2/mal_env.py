from typing import Any, Optional, Self
from mal_types import MalList
from mal_token import MalToken
from mal_error import MalSymbolNotFoundError, MalSyntaxError


class MalEnv:
    def __init__(self, outer: Optional['MalEnv'] = None, binds: Optional[list[MalToken]] = None, exprs: Optional[list[MalToken]] = None):
        self.outer = outer
        self.data: dict[str, MalToken] = {}

        if not exprs:
            exprs = []

        if binds:
            for i in range(len(binds)):
                if binds[i].value == "&":
                    self.set(str(binds[i+1]), MalList(exprs[i:]))
                    break
                if i > len(exprs):
                    raise MalSyntaxError("Mismatched number of symbols and expressions in environment creation", 0)
                self.set(str(binds[i]), exprs[i])

    def set(self, key: str, value: MalToken) -> None:
        self.data[key] = value

    def get(self, key:str) -> MalToken:
        if key in self.data:
            return self.data[key]
        if self.outer:
            return self.outer.get(key)
        raise MalSymbolNotFoundError(key)
    
    def __str__(self):
        return f"#<env: {self.data}>"