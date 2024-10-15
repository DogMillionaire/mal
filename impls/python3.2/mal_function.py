from mal_types import MalCollection
from mal_token import MalToken
from mal_env import MalEnv

class MalFunction(MalToken):
    def __init__(self, name: str, start: int, end: int, env: MalEnv, binds: MalCollection, ast: MalToken, is_macro: bool = False):
        super().__init__(name, start, end)
        self.env = env
        self.binds = binds
        self.ast = ast
        self.name = name
        self.is_macro = is_macro

    def __str__(self):
        return f"#<function:{self.name if self.name else "anonymous"}>"

class MalNativeFunction(MalToken):
    def __init__(self, name: str, func):
        super().__init__(f"#<function:native:{name}>", -1, -1)
        self.func = func
        self.name = name

    def __str__(self):
        return f"#<function:native:{self.name}>"