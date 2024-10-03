from mal_types import MalCollection
from mal_token import MalToken
from mal_env import MalEnv

class MalFunction(MalToken):
    def __init__(self, value: str, start: int, end: int, env: MalEnv, binds: MalCollection, ast: MalToken):
        super().__init__(value, start, end)
        self.env = env
        self.binds = binds
        self.ast = ast

    def __str__(self):
        return f"#<function>"

class MalNativeFunction(MalToken):
    def __init__(self, func):
        super().__init__("#<function>", -1, -1)
        self.func = func

    def __str__(self):
        return f"#<function>"