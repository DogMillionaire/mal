from typing import Any
from reader import MalSyntaxError, read_form, read_str
from printer import pr_str
from mal_token import MalBoolean, MalCollection, MalHashMap, MalList, MalNil, MalSymbol, MalToken, MalVector
from mal_error import MalError, MalNoInputError, MalSymbolNotFoundError
from mal_env import MalEnv
import mal_readline

def READ(source: str) -> MalToken:
    reader = read_str(source)
    return read_form(reader)

def is_debug(env: MalEnv) -> bool:
    try:
        value = env.get("DEBUG-EVAL")
        match value:
            case MalNil():
                return False
            case MalBoolean() as b:
                return b.boolean_value
            case _:
                return True
        return value is not MalNil and value is not MalFalse
    except MalSymbolNotFoundError:
        return False

def EVAL(ast: MalToken, env: MalEnv) -> MalToken:
    if is_debug(env):
        print(f"EVAL: {PRINT(ast)}")
    match ast:
        case MalSymbol() as s:
            return env.get(s.value)
        case MalList() as l:
            if l.size == 0:
                return l
            
            first_symbol = l.elements[0]
            if isinstance(first_symbol, MalSymbol) and first_symbol.value == "def!":
                if l.size != 3:
                    raise MalSyntaxError("def! expects exactly 2 arguments", l.start)
                symbol = l.elements[1]
                value = EVAL(l.elements[2], env)
                env.set(symbol.value, value)
                return value
            
            if isinstance(first_symbol, MalSymbol) and first_symbol.value == "let*":
                if l.size != 3:
                    raise MalSyntaxError("let* expects exactly 2 arguments", l.start)
                bindings = l.elements[1]
                if not isinstance(bindings, MalCollection):
                    raise MalSyntaxError("let* expects a list of bindings", l.start)
                new_env = MalEnv(env)
                for i in range(0, bindings.size, 2):
                    symbol = bindings.elements[i]
                    value = EVAL(bindings.elements[i+1], new_env)
                    new_env.set(symbol.value, value)
                return EVAL(l.elements[2], new_env)

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
    
    repl_env = MalEnv()
    repl_env.set('+', lambda a,b: a+b)
    repl_env.set('-', lambda a,b: a-b)
    repl_env.set('*', lambda a,b: a*b)
    repl_env.set('/', lambda a,b: a/b)

    
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
