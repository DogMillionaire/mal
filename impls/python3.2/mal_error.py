class MalError(Exception):
    pass

class MalEOFError(MalError):
    pass

class MalNoInputError(MalError):
    pass

class MalSymbolNotFoundError(MalError):
    symbol: str

    def __init__(self, symbol: str):
        super().__init__(f"Symbol {symbol} not found in the environment")
        self.symbol = symbol

class MalSyntaxError(MalError):
    start: int

    def __init__(self, message: str, start: int):
        super().__init__(message)
        self.start = start