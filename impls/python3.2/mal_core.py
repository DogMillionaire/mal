from mal_token import MalToken
from mal_types import MalAtom, MalBoolean, MalCollection, MalList, MalNil, MalNumber, MalString
from mal_function import MalNativeFunction
from printer import pr_str
from reader import read_form, read_str

def prn(print_readable: bool, tokens: list[MalToken]) -> MalNil:
    if tokens:
            print(" ".join([(pr_str(token, print_readable)) for token in tokens]))
    else:
        print()
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
}