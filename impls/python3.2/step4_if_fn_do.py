from ast import expr
from operator import concat
import re
from typing import Any
from reader import MalSyntaxError, read_form, read_str
from printer import pr_str
from mal_types import MalBoolean, MalCollection, MalHashMap, MalList, MalNil, MalSymbol, MalToken, MalVector
from mal_error import MalError, MalNoInputError, MalSymbolNotFoundError
from mal_env import MalEnv
from mal_function import MalFunction, MalNativeFunction
import mal_core
import mal_readline

def READ(source: str) -> MalToken:
    reader = read_str(source)
    return read_form(reader)

def is_truthy(value: MalToken) -> bool:
    if isinstance(value, MalNil):
        return False
    if isinstance(value, MalBoolean):
        return value.boolean_value
    return True

def is_debug(env: MalEnv) -> bool:
    try:
        value = env.get("DEBUG-EVAL")
        return is_truthy(value)
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
            if isinstance(first_symbol, MalSymbol):
                match first_symbol.value:
                    case "def!":
                        if l.size != 3:
                            raise MalSyntaxError("def! expects exactly 2 arguments", l.start)
                        symbol = l.elements[1]
                        value = EVAL(l.elements[2], env)
                        env.set(symbol.value, value)
                        return value
                    case "let*":
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
                    case "do":
                        if l.size == 1:
                            return MalNil()
                        evaluated_items = [EVAL(item, env) for item in l.elements[1:]]
                        return evaluated_items[-1]
                    case "if":
                        if l.size != 3 and l.size != 4:
                            raise MalSyntaxError("if expects exactly 2 or 3 arguments", l.start)
                        condition = EVAL(l.elements[1], env)
                        if is_truthy(condition):
                            return EVAL(l.elements[2], env)
                        else:
                            if l.size == 4:
                                return EVAL(l.elements[3], env)
                            return MalNil()
                    case "fn*":
                        if l.size != 3:
                            raise MalSyntaxError("fn* expects exactly 2 arguments", l.start)
                        
                        binds = l.elements[1]

                        if not isinstance(binds, MalCollection):
                            raise MalSyntaxError("First argument to fn* must be a list of bindings", l.start)
        
                        ast = l.elements[2]
                        return MalFunction(l.value, l.start, l.end, env, binds, ast)

            evaluated_items = [EVAL(item, env) for item in l.elements]
            if isinstance(evaluated_items[0], MalNil):
                return MalList(evaluated_items, l.start, l.end)
            match evaluated_items[0]:
                case MalFunction() as f:
                    new_env = MalEnv(f.env, f.binds.elements, evaluated_items[1:])
                    return EVAL(f.ast, new_env)
                case MalNativeFunction() as nf:
                    return nf.func(*evaluated_items[1:])
                case _:
                    raise MalSyntaxError(f"First element of list is not a function: {pr_str(evaluated_items[0], True)}", l.start)
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

def rep(source: str, env: MalEnv) -> str:
    return PRINT(EVAL(READ(source), env))

def main() -> None:
    
    repl_env = MalEnv()

    for (symbol, value) in mal_core.ns.items():
        repl_env.set(symbol, value)
    
    rep("(def! not (fn* (a) (if a false true)))", repl_env)
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
