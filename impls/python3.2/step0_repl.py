import mal_readline

def READ(source: str) -> str:
    return source

def EVAL(ast: str) -> str:
    return ast

def PRINT(form: str) -> str:
    return form

def rep(source: str) -> str:
    return PRINT(EVAL(READ(source)))

def main() -> None:
    while True:
        try:
            
            print(rep(mal_readline.input_('user> ')))
        except EOFError:
            break

if __name__ == '__main__':
    main()
