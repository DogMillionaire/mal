from optparse import Option
from typing import Optional
from mal_token import MalToken
from mal_types import MalBoolean, MalCollection, MalList, MalNil, MalNumber, MalString
from mal_function import MalNativeFunction
from printer import pr_str

def prn(print_readable: bool, tokens: list[MalToken]) -> MalNil:
    if tokens:
            print(" ".join([(pr_str(token, print_readable)) for token in tokens]))
    else:
        print()
    return MalNil()

ns = {
    '+': MalNativeFunction(lambda a,b: a+b),
    '-': MalNativeFunction(lambda a,b: a-b),
    '*': MalNativeFunction(lambda a,b: a*b),
    '/': MalNativeFunction(lambda a,b: a/b),
    'list': MalNativeFunction(lambda *a: MalList([*a], -1, -1)),
    'list?': MalNativeFunction(lambda a: MalBoolean(isinstance(a, MalList))),
    'empty?': MalNativeFunction(lambda a: MalBoolean(len(a.elements) == 0)),
    'count': MalNativeFunction(lambda a: MalNumber(str(len(a.elements) if isinstance(a, MalCollection) else 0))),
    '=': MalNativeFunction(lambda a,b: a == b),
    '<': MalNativeFunction(lambda a,b: a < b),
    '<=': MalNativeFunction(lambda a,b: a <= b),
    '>': MalNativeFunction(lambda a,b: a > b),
    '>=': MalNativeFunction(lambda a,b: a >= b),
    'pr-str': MalNativeFunction(lambda *a: MalString(" ".join([pr_str(e, True) for e in a]), -1, -1)),
    'str': MalNativeFunction(lambda *a: MalString("".join([pr_str(e, False) for e in a]), -1, -1)),
    'prn': MalNativeFunction(lambda *a: prn(True, [*a])),
    'println': MalNativeFunction(lambda *a: prn(False, [*a])),
}