from mal_token import MalToken


class MalError(Exception):
    pass

class MalEOFError(MalError):
    pass

class MalNoInputError(MalError):
    pass

class MalSymbolNotFoundError(MalError):
    symbol: str

    def __init__(self, symbol: str):
        super().__init__(f"'{symbol}' not found")
        self.symbol = symbol

class MalSyntaxError(MalError):
    start: int

    def __init__(self, message: str, start: int):
        super().__init__(message)
        self.start = start

class MalTokenException(MalError):
    token: MalToken

    def __init__(self, token: MalToken):
        super().__init__(str(token))
        self.token = token