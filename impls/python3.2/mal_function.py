from mal_types import MalCollection, MalNil
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
        self.meta:MalToken = MalNil()

    def __str__(self):
        return f"#<function:{self.name if self.name else "anonymous"}>"
    
    def clone(self):
        return MalFunction(self.name, self.start, self.end, self.env, self.binds, self.ast, self.is_macro)

class MalNativeFunction(MalToken):
    def __init__(self, name: str, func):
        super().__init__(f"#<function:native:{name}>", -1, -1)
        self.func = func
        self.name = name
        self.meta:MalToken = MalNil()

    def __str__(self):
        return f"#<function:native:{self.name}>"
    
    def clone(self):
        return MalNativeFunction(self.name, self.func)