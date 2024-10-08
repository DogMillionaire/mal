from mal_token import MalToken
from mal_types import MalAtom, MalBoolean, MalCollection, MalList, MalNil, MalNumber, MalString, MalVector
from mal_function import MalFunction, MalNativeFunction
from printer import pr_str
from reader import read_form, read_str
from mal_error import MalSyntaxError

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

ns = {
    '+': MalNativeFunction("+", lambda a,b: a+b),
    '-': MalNativeFunction("-", lambda a,b: a-b),
    '*': MalNativeFunction("*", lambda a,b: a*b),
    '/': MalNativeFunction("/", lambda a,b: a/b),
    'list': MalNativeFunction("list", lambda *a: MalList([*a], -1, -1)),
    'list?': MalNativeFunction("list?", lambda a: MalBoolean(isinstance(a, MalList))),
    'empty?': MalNativeFunction("empty?", lambda a: MalBoolean(len(a.elements) == 0)),
    'count': MalNativeFunction("count", lambda a: MalNumber(str(len(a.elements) if isinstance(a, MalCollection) else 0))),
    '=': MalNativeFunction("=", lambda a,b: a == b),
    '<': MalNativeFunction("<", lambda a,b: a < b),
    '<=': MalNativeFunction("<=", lambda a,b: a <= b),
    '>': MalNativeFunction(">", lambda a,b: a > b),
    '>=': MalNativeFunction(">=", lambda a,b: a >= b),
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
}