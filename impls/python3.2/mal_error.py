class MalError(Exception):
    pass

class MalEOFError(MalError):
    pass

class MalNoInputError(MalError):
    pass

class MalSyntaxError(MalError):
    start: int

    def __init__(self, message: str, start: int):
        super().__init__(message)
        self.start = start