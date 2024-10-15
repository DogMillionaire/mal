import time
from mal_token import MalToken
from mal_types import MalAtom, MalBoolean, MalCollection, MalHashMap, MalKeyword, MalList, MalNil, MalNumber, MalString, MalSymbol, MalVector
from mal_function import MalFunction, MalNativeFunction
from printer import pr_str
from reader import read_form, read_str
from mal_error import MalSyntaxError, MalTokenException
from mal_readline import input_

def prn(print_readable: bool, tokens: list[MalToken]) -> MalNil:
    if tokens:
            print(" ".join([(pr_str(token, print_readable)) for token in tokens]))
    else:
        print()
    return MalNil()

def vec(list:MalCollection) -> MalVector:
    if not isinstance(list, MalCollection):
        raise MalSyntaxError(f"Cannot convert {type(list)} to a vector", list.start)
    if isinstance(list, MalVector):
        return list
    return MalVector(list.elements, list.start, list.end)

def first(collection:MalCollection) -> MalToken:
    if not isinstance(collection, MalCollection) or len(collection.elements) == 0:
        return MalNil()
    return collection.elements[0]

def rest(collection:MalCollection) -> MalToken:
    if not isinstance(collection, MalCollection) or len(collection.elements) == 0:
        return MalList([])
    return MalList(collection.elements[1:], collection.start, collection.end)

def throw(token:MalToken) -> None:
    raise MalTokenException(token)

def hash_map(elements: list[MalToken]) -> MalHashMap:
    if len(elements) % 2 != 0:
        raise MalSyntaxError("Odd number of elements in hash-map", 0)
    return MalHashMap(elements, -1, -1)

def assoc(map:MalHashMap, *elements:MalToken) -> MalHashMap:
    if len(elements) % 2 != 0:
        raise MalSyntaxError("Odd number of elements in assoc", 0)
    
    new_map = map.clone()
    
    for i in range(0, len(elements), 2):
        new_map.data[elements[i]] = elements[i+1]
    return new_map

def dissoc(map:MalHashMap, *elements:MalToken) -> MalHashMap:
    new_map = map.clone()
    
    for elt in [*elements]:
        new_map.data.pop(elt, None)
    return new_map

def conj(collection:MalCollection, elements:list[MalToken]) -> MalCollection:
    if isinstance(collection, MalList):
        return MalList([*elements.__reversed__(), *collection.elements], collection.start, collection.end)
    if isinstance(collection, MalVector):
        return MalVector([*collection.elements, *elements], collection.start, collection.end)
    raise MalSyntaxError(f"Cannot conj to {type(collection)}", collection.start)

def seq(form: MalToken) -> MalToken:
    match form:
        case MalList(_, _, _) as l:
            return l if l.size > 0 else MalNil()
        case MalVector(_, _, _) as v:
            return MalList(v.elements) if v.size > 0 else MalNil()
        case MalString(value, _, _):
            return MalList([MalString(c) for c in value]) if len(value) > 0 else MalNil()
        case MalNil() as n: 
            return n
    raise MalSyntaxError(f"Cannot convert {type(form)} to a sequence", form.start)

def meta(form: MalFunction | MalNativeFunction | MalList | MalVector | MalHashMap) -> MalToken:
    return form.meta

def with_meta(form: MalFunction | MalNativeFunction | MalList | MalVector | MalHashMap, meta: MalToken) -> MalToken:
    new_form = form.clone()
    new_form.meta = meta
    return new_form

def readline(prompt: MalString) -> MalToken:
    try:
        return MalString(input_(prompt.value))
    except EOFError:
        return MalNil()

ns = {
    '+': MalNativeFunction("+", lambda a,b: a+b),
    '-': MalNativeFunction("-", lambda a,b: a-b),
    '*': MalNativeFunction("*", lambda a,b: a*b),
    '/': MalNativeFunction("/", lambda a,b: a/b),
    'list': MalNativeFunction("list", lambda *a: MalList([*a], -1, -1)),
    'list?': MalNativeFunction("list?", lambda a: MalBoolean(isinstance(a, MalList))),
    'empty?': MalNativeFunction("empty?", lambda a: MalBoolean(len(a.elements) == 0)),
    'count': MalNativeFunction("count", lambda a: MalNumber(str(len(a.elements) if isinstance(a, MalCollection) else 0))),
    '=': MalNativeFunction("=", lambda a,b: MalBoolean(a == b)),
    '<': MalNativeFunction("<", lambda a,b: MalBoolean(a < b)),
    '<=': MalNativeFunction("<=", lambda a,b: MalBoolean(a <= b)),
    '>': MalNativeFunction(">", lambda a,b: MalBoolean(a > b)),
    '>=': MalNativeFunction(">=", lambda a,b: MalBoolean(a >= b)),
    'pr-str': MalNativeFunction("pr-str", lambda *a: MalString(" ".join([pr_str(e, True) for e in a]), -1, -1)),
    'str': MalNativeFunction("str", lambda *a: MalString("".join([pr_str(e, False) for e in a]), -1, -1)),
    'prn': MalNativeFunction("prn", lambda *a: prn(True, [*a])),
    'println': MalNativeFunction("println", lambda *a: prn(False, [*a])),
    'read-string': MalNativeFunction("read-string", lambda a: read_form(read_str(a.value))),
    'slurp': MalNativeFunction("slurp", lambda a: MalString(open(a.value).read())),
    'atom': MalNativeFunction("atom", lambda a: MalAtom(a)),
    'atom?': MalNativeFunction("atom?", lambda a: MalBoolean(isinstance(a, MalAtom))),
    'deref': MalNativeFunction("deref", lambda a: a.deref()),
    'reset!': MalNativeFunction("reset!", lambda a, b: a.reset(b)),
    'cons' : MalNativeFunction("cons", lambda a, b: MalList([a, *b.elements])),
    'concat': MalNativeFunction("concat", lambda *a: MalList([e for l in a for e in l.elements])),
    'vec': MalNativeFunction("vec", lambda a: vec(a)),
    'nth': MalNativeFunction("nth", lambda a, b:  a.elements[b.numeric_value]),
    'first': MalNativeFunction("first", lambda a: first(a)),
    'rest': MalNativeFunction("rest", lambda a: rest(a)),
    'macro?': MalNativeFunction("macro?", lambda a: MalBoolean(isinstance(a, MalFunction) and a.is_macro)),
    'throw': MalNativeFunction("throw", lambda a: throw(a)),
    'nil?': MalNativeFunction("nil?", lambda a: MalBoolean(isinstance(a, MalNil))),
    'true?': MalNativeFunction("true?", lambda a: MalBoolean(isinstance(a, MalBoolean) and a.boolean_value)),
    'false?': MalNativeFunction("false?", lambda a: MalBoolean(isinstance(a, MalBoolean) and not a.boolean_value)),
    'symbol?': MalNativeFunction("symbol?", lambda a: MalBoolean(isinstance(a, MalSymbol))),
    'symbol': MalNativeFunction("symbol", lambda a: MalSymbol(a.value)),
    'keyword': MalNativeFunction("keyword", lambda a: a if isinstance(a, MalKeyword) else MalKeyword(a.value)),
    'keyword?': MalNativeFunction("keyword?", lambda a: MalBoolean(isinstance(a, MalKeyword))),
    'vector': MalNativeFunction("vector", lambda *a: MalVector([*a], -1, -1)),
    'vector?': MalNativeFunction("vector?", lambda a: MalBoolean(isinstance(a, MalVector))),
    'sequential?': MalNativeFunction("sequential?", lambda a: MalBoolean(isinstance(a, MalCollection))),
    'hash-map': MalNativeFunction("hash-map", lambda *a: hash_map([*a])),
    'map?': MalNativeFunction("map?", lambda a: MalBoolean(isinstance(a, MalHashMap))),
    'assoc': MalNativeFunction("assoc", lambda a, *b: assoc(a, *b)),
    'dissoc': MalNativeFunction("dissoc", lambda a, *b: dissoc(a, *b)),
    'get': MalNativeFunction("get", lambda a, b: MalNil() if isinstance(a, MalNil) else a.data.get(b, MalNil())),
    'contains?': MalNativeFunction("contains?", lambda a, b: MalBoolean(b in a.data)),
    'keys': MalNativeFunction("keys", lambda a: MalList([k for k in a.data.keys()])),
    'vals': MalNativeFunction("vals", lambda a: MalList([v for v in a.data.values()])),
    'time-ms': MalNativeFunction("time-ms", lambda: MalNumber(str(int(time.time() * 1000)))),
    'fn?': MalNativeFunction("fn?", lambda a: MalBoolean((isinstance(a, MalFunction) and not a.is_macro) or isinstance(a, MalNativeFunction))),
    'macro?': MalNativeFunction("macro?", lambda a: MalBoolean(isinstance(a, MalFunction) and a.is_macro)),
    'string?': MalNativeFunction("string?", lambda a: MalBoolean(isinstance(a, MalString))),
    'number?': MalNativeFunction("number?", lambda a: MalBoolean(isinstance(a, MalNumber))),
    'conj': MalNativeFunction("conj", lambda a, *b: conj(a, [*b])),
    'seq': MalNativeFunction("seq", lambda a: seq(a)),
    'meta': MalNativeFunction("meta", lambda a: meta(a)),
    'with-meta': MalNativeFunction("with-meta", lambda a, b: with_meta(a, b)),
    'readline': MalNativeFunction("readline", lambda a: readline(a)),
}