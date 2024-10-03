from typing import Any
from reader import MalSyntaxError, Reader, read_form, read_str
from printer import pr_str
from mal_types import MalHashMap, MalList, MalNil, MalSymbol, MalToken, MalVector
from mal_token import MalToken
from mal_error import MalError, MalNoInputError
import mal_readline

def READ(source: str) -> MalToken:
    reader = read_str(source)
    return read_form(reader)

def EVAL(ast: MalToken, env: dict[str, Any]) -> MalToken:
    match ast:
        case MalSymbol() as s:
            symbol = s.value
            if symbol not in env:
                raise MalSyntaxError(f"Symbol {symbol} not found in the environment", s.start)
            return env[symbol]
        case MalList() as l:
            if l.size == 0:
                return l
            evaluated_items = [EVAL(item, env) for item in l.elements]
            if isinstance(evaluated_items[0], MalNil):
                return MalList(evaluated_items, l.start, l.end)
            # TODO: Fix this when we can define MalFunctions
            return evaluated_items[0](*evaluated_items[1:]) # type: ignore
        case MalVector() as v:
            if v.size == 0:
                return v
            evaluated_items = [EVAL(item, env) for item in v.elements]
            return MalVector(evaluated_items, v.start, v.end)
        case MalHashMap() as h:
            if h.size == 0:
                return h
            evaluated_items = []
            
            for key, value in h.data.items():
                evaluated_value = EVAL(value, env)
                evaluated_items.append(key)
                evaluated_items.append(evaluated_value)
            return MalHashMap(evaluated_items, h.start, h.end)
        case _:
            return ast

def PRINT(form: MalToken) -> str:
    return pr_str(form, True)

def rep(source: str, env: dict[str, Any]) -> str:
    return PRINT(EVAL(READ(source), env))

def main() -> None:
    
    repl_env = {'+': lambda a,b: a+b,
            '-': lambda a,b: a-b,
            '*': lambda a,b: a*b,
            '/': lambda a,b: a/b}
    
    while True:
        try:
            
            print(rep(mal_readline.input_('user> '), repl_env))
        except MalNoInputError:
            continue
        except MalError as e:
            print(f"Error: {e}")
        except EOFError:
            break

if __name__ == '__main__':
    main()
