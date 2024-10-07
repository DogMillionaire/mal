from ast import expr
from operator import concat
import re
import sys
from typing import Any, Callable
from reader import MalSyntaxError, read_form, read_str
from printer import pr_str
from mal_types import MalAtom, MalBoolean, MalCollection, MalHashMap, MalList, MalNil, MalString, MalSymbol, MalToken, MalVector
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
    while True:
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
                            if isinstance(value, MalFunction):
                                value.name = symbol.value
                                value.value = symbol.value
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
                            env = new_env
                            ast = l.elements[2]
                            continue
                        case "do":
                            if l.size == 1:
                                return MalNil()
                            evaluated_items = [EVAL(item, env) for item in l.elements[1:-1]]
                            ast = l.elements[-1]
                            continue
                        case "if":
                            if l.size != 3 and l.size != 4:
                                raise MalSyntaxError("if expects exactly 2 or 3 arguments", l.start)
                            condition = EVAL(l.elements[1], env)
                            if is_truthy(condition):
                                ast = l.elements[2]
                            else:
                                if l.size == 4:
                                    ast = l.elements[3]
                                else:
                                    return MalNil()
                            continue
                        case "fn*":
                            if l.size != 3:
                                raise MalSyntaxError("fn* expects exactly 2 arguments", l.start)
                            
                            binds = l.elements[1]

                            if not isinstance(binds, MalCollection):
                                raise MalSyntaxError("First argument to fn* must be a list of bindings", l.start)
            
                            ast = l.elements[2]
                            return MalFunction(l.value, l.start, l.end, env, binds, ast)
                        case "swap!":   
                            if l.size < 3:
                                raise MalSyntaxError("swap! expects at least 2 argument", l.start)
                            atom = EVAL(l.elements[1], env)
                            if not isinstance(atom, MalAtom):
                                raise MalSyntaxError("swap! expects an atom as the first argument", l.start)
                            function = EVAL(l.elements[2], env)
                            if not isinstance(function, MalFunction) and not isinstance(function, MalNativeFunction):
                                raise MalSyntaxError("swap! expects a function as the second argument", l.start)
                            function_call = [function] + [atom.deref()] +  l.elements[3:]
                            new_value = EVAL(MalList(function_call), env)
                            atom.reset(new_value)
                            return new_value
                        case "quote":
                            if l.size != 2:
                                raise MalSyntaxError("quote expects exactly 1 argument", l.start)
                            return l.elements[1]
                        case 'quasiquote':
                            if l.size != 2:
                                raise MalSyntaxError("quasiquote expects exactly 1 argument", l.start)
                            ast = quasiquote(l.elements[1])
                            continue
                evaluated_items = [EVAL(item, env) for item in l.elements]
                if isinstance(evaluated_items[0], MalNil):
                    return MalList(evaluated_items, l.start, l.end)
                match evaluated_items[0]:
                    case MalFunction() as f:
                        new_env = MalEnv(f.env, f.binds.elements, evaluated_items[1:])
                        ast = f.ast
                        env = new_env
                        if is_debug(env):
                            print(f"Function call: {f}")
                            print(f"\tEnv: {env}")
                    case MalNativeFunction() as nf:
                        if is_debug(env):
                            print(f"Function call: {nf}")
                            print(f"\tArgs: {evaluated_items[1:]}")
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

def quasiquote(ast: MalToken) -> MalToken:
    if isinstance(ast, MalList):
        if ast.size == 0:
            return ast
        if isinstance(ast.elements[0], MalSymbol) and ast.elements[0].value == "unquote":
            if ast.size < 2:
                raise MalSyntaxError("unquote expects at least 1 argument", ast.start)
            return ast.elements[1]
        results: list[MalToken] = []
        for elt in ast.elements.__reversed__():
            if isinstance(elt, MalCollection) and elt.size > 0 and isinstance(elt.elements[0], MalSymbol) and elt.elements[0].value == "splice-unquote":
                    if elt.size < 2:
                        raise MalSyntaxError("splice-unquote expects at least 1 argument", ast.start)
                    results = [MalSymbol("concat"), elt.elements[1], MalList(results)]
            else:
                results = [MalSymbol("cons"), quasiquote(elt), MalList(results)]
        return MalList(results)
    if isinstance(ast, MalSymbol) or isinstance(ast, MalHashMap):
        return MalList([MalSymbol("quote"), ast])
    if isinstance(ast, MalVector):
        result: list[MalToken] = [MalSymbol("vec")]
        result.append(quasiquote(MalList(ast.elements)))
        return MalList(result)
    return ast


def main() -> None:
    
    repl_env = MalEnv()

    for (symbol, value) in mal_core.ns.items():
        repl_env.set(symbol, value)
        
    eval_function: Callable[[MalToken], MalToken] = lambda x: EVAL(x, repl_env)
    repl_env.set("eval", MalNativeFunction("eval", eval_function))

    rep("(def! not (fn* (a) (if a false true)))", repl_env)
    rep("""(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))""", repl_env)
    rep("(def! DEBUG-EVAL true)", repl_env)

    args = sys.argv[1:]
    
    if args:
        file = args[0]
        repl_env.set("*ARGV*", MalList([MalString(arg) for arg in args[1:]]))
        rep(f"(load-file \"{file}\")", repl_env)
        return
    else:
        repl_env.set("*ARGV*", MalList([]))

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
