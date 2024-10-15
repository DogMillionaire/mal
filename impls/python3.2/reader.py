from os import read
import re
from tracemalloc import start

from mal_types import MalBoolean, MalHashMap, MalKeyword, MalList, MalNil, MalNumber, MalString, MalSymbol, MalVector
from mal_token import MalToken
from mal_error import MalEOFError, MalNoInputError, MalSyntaxError

class Reader:
    tokens: list[MalToken]
    position: int

    def __init__(self, tokens: list[MalToken]):
        self.tokens = tokens
        self.position = 0

    def next(self) -> MalToken:
        if self.position >= len(self.tokens):
            raise MalEOFError("End of file reached")
        
        token = self.tokens[self.position]
        self.position += 1
        
        return token

    def peek(self) -> MalToken:
        if self.position >= len(self.tokens):
            raise MalEOFError("End of file reached")
        return self.tokens[self.position]

def read_string(reader: Reader) -> MalToken:
    token = reader.next()

    if token.value == '"':
        raise MalSyntaxError(f"EOF encountered while reading string starting at position {token.start}", token.start)

    value = ""

    escape = False
    for c in token.value[1:-1]:
        if escape:
            if c == "n":
                value += "\n"
            elif c == "\\":
                value += "\\"
            elif c == "\"":
                value += "\""
            else:
                raise MalSyntaxError(f"Invalid escape sequence \\{c} at position {token.start}", token.start)
            escape = False
        elif c == "\\":
            escape = True
        else:
            value += c

    if token.value[-1] != '"' or (token.value[-1] == '"' and escape):
        raise MalSyntaxError(f"EOF encountered while reading string starting at position {token.start}", token.start)

    return MalString(value, token.start, token.end)

def read_atom(reader: Reader) -> MalToken:
    token = reader.next()

    if token.value[0].isdigit() or (token.value[0] == "-" and len(token.value) > 1 and token.value[1].isdigit()):
        return MalNumber(token.value, token.start, token.end)

    match token.value:
        case "false" | "true":
            return MalBoolean(token.value == "true", token.start, token.end)
        case "nil":
            return MalNil(token.value, token.start, token.end)

    return MalSymbol(token.value, token.start, token.end)

def read_elements(reader: Reader, collection_type: type, end_token: str) -> tuple[list[MalToken], int]:
    start = reader.peek().start
    
    # Skip the opening parenthesis
    reader.next()

    # Create a list to store the elements of the list
    elements = []
    # Read elements until we reach the closing token
    try:
        while reader.peek().value != end_token:
            try:
                elements.append(read_form(reader))
            except MalNoInputError: 
                reader.next() # Skip comments
                continue
    except MalEOFError:
        raise MalSyntaxError(f"EOF encountered while reading {str(collection_type)} starting at position {start}", start)

    # Skip the closing token
    end = reader.peek().end
    reader.next()

    return (elements, end)

def read_list(reader: Reader) -> MalToken:
    start = reader.peek().start
    (elements, end) = read_elements(reader, MalList, ")")
    return MalList(elements, start, end)

def read_vector(reader: Reader) -> MalToken:
    start = reader.peek().start
    (elements, end) = read_elements(reader, MalVector, "]")
    return MalVector(elements, start, end)

def read_hash_map(reader: Reader) -> MalToken:
    start = reader.peek().start
    (elements, end) = read_elements(reader, MalHashMap, "}")
    return MalHashMap(elements, start, end)

def read_macro(reader: Reader, replacement: str, macro_len: int = 1) -> MalToken:
    start = reader.peek().start
    reader.next()
    list = [MalSymbol(replacement, start, start + macro_len), read_form(reader)]
    end = list[-1].end
    return MalList(list, start, end)

def read_form(reader: Reader) -> MalToken:
    token = reader.peek()

    if token.value == "~@":
        return read_macro(reader, "splice-unquote", 2)

    # Switch on the first character of the token
    match token.value[0]:
        case "(":
            return read_list(reader)
        case "[":
            return read_vector(reader)
        case "{":
            return read_hash_map(reader)
        case "\"":
            return read_string(reader)
        case ";":
            raise MalNoInputError("Comment encountered")
        case "'":
            return read_macro(reader, "quote")
        case "`":
            return read_macro(reader, "quasiquote")
        case "~":
            return read_macro(reader, "unquote")
        case "@":
            return read_macro(reader, "deref")
        case ":":
            reader.next()
            return MalKeyword(token.value[1:], token.start, token.end)
        case "^":
            reader.next()
            meta = read_form(reader)
            form = read_form(reader)
            return MalList([MalSymbol("with-meta"), form, meta], token.start, token.end)
        case _:
            return read_atom(reader)

def tokenize(input: str) -> list[MalToken]:
    token_re = re.compile(r"""[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)""")

    return [MalToken(match.group(1).strip(), match.start(), match.end()) for match in token_re.finditer(input) if match.start() != match.end()]


def read_str(input: str) -> Reader:
    tokens = tokenize(input)
    return Reader(tokens)

